//!
//! The deployable object a module emits: a contract or a library.
//!

use std::collections::HashMap;

use slang_solidity_v2::ast::ContractBase;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionMutability;
use slang_solidity_v2::ast::Identifier;
use slang_solidity_v2::ast::LibraryDefinition;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::NodeLocation;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::StateVariableMutability;
use slang_solidity_v2::compilation::FileId;

use solx_mlir::ContractKind;

use crate::contract::storage_slot::StorageSlot;

/// The deployable object a module emits, each variant carrying the definition its kind
/// dispatches from.
pub enum Object {
    /// A contract: storage, a constructor, selector dispatch.
    Contract(ContractDefinition),
    /// A library: no storage, no constructor, `DELEGATECALL` dispatch.
    Library(LibraryDefinition),
}

impl Object {
    /// Classifies the definition a name resolves to, admitting the two that deploy an object.
    pub fn from_definition(definition: Definition) -> Option<Self> {
        match definition {
            Definition::Contract(contract) => Some(Self::Contract(contract)),
            Definition::Library(library) => Some(Self::Library(library)),
            _ => None,
        }
    }

    /// The object's name.
    pub fn name(&self) -> Identifier {
        match self {
            Self::Contract(node) => node.name(),
            Self::Library(node) => node.name(),
        }
    }

    /// The file that declares the object.
    pub fn file_id(&self) -> &FileId {
        match self {
            Self::Contract(node) => node.get_file_id(),
            Self::Library(node) => node.get_file_id(),
        }
    }

    /// The byte offset of the definition's first byte in its file.
    pub fn offset(&self) -> usize {
        match self {
            Self::Contract(node) => node.calculate_text_range(),
            Self::Library(node) => node.calculate_text_range(),
        }
        .expect("slang locates every definition")
        .start
    }

    /// The contract's own constructor.
    pub fn constructor(&self) -> Option<FunctionDefinition> {
        match self {
            Self::Contract(node) => node.constructor(),
            Self::Library(_) => None,
        }
    }

    /// The byte offset of the own constructor's first byte, or the definition's when synthesized.
    pub fn constructor_offset(&self) -> usize {
        self.constructor().map_or(self.offset(), |constructor| {
            constructor
                .calculate_text_range()
                .expect("slang locates every constructor")
                .start
        })
    }

    /// Whether the own constructor accepts value; a synthesized one is non-payable.
    pub fn has_payable_constructor(&self) -> bool {
        self.constructor().is_some_and(|constructor| {
            matches!(
                constructor.attributes().mutability(),
                FunctionMutability::Payable
            )
        })
    }

    /// The object's identifier, qualified by its file: linking keys objects by it, and two files
    /// may declare the same name.
    pub fn identifier(&self) -> String {
        solx_utils::ContractName::full_path(self.file_id().as_str(), self.name().name())
    }

    /// The objects the deploy code may embed, its runtime object leading.
    pub fn deploy_dependencies(&self) -> solx_utils::Dependencies {
        let definitions = match self {
            Self::Contract(node) => node.creation_bytecode_dependencies(),
            Self::Library(_) => Vec::new(),
        };
        let deploy_identifier = self.identifier();
        let runtime_identifier =
            solx_utils::Dependencies::runtime_identifier(deploy_identifier.as_str());
        Self::dependencies(deploy_identifier, Some(runtime_identifier), definitions)
    }

    /// The objects the runtime code may embed.
    pub fn runtime_dependencies(&self) -> solx_utils::Dependencies {
        let definitions = match self {
            Self::Contract(node) => node.deployed_bytecode_dependencies(),
            Self::Library(node) => node.deployed_bytecode_dependencies(),
        };
        let runtime_identifier =
            solx_utils::Dependencies::runtime_identifier(self.identifier().as_str());
        Self::dependencies(runtime_identifier, None, definitions)
    }

    fn dependencies(
        code_identifier: String,
        runtime_identifier: Option<String>,
        definitions: Vec<Definition>,
    ) -> solx_utils::Dependencies {
        let dependencies = definitions.into_iter().map(|definition| {
            Self::from_definition(definition)
                .expect("Slang bytecode dependencies are contracts or libraries")
                .identifier()
        });

        solx_utils::Dependencies::new(code_identifier.as_str(), runtime_identifier, dependencies)
    }

    /// The kind the object's `sol.contract` declares.
    pub fn kind(&self) -> ContractKind {
        match self {
            Self::Contract(_) => ContractKind::Contract,
            Self::Library(_) => ContractKind::Library,
        }
    }

    /// Whether the object accepts value: a library is reached by `DELEGATECALL` alone, which
    /// carries the caller's.
    pub fn is_payable(&self) -> bool {
        match self {
            Self::Contract(node) => node.is_payable(),
            Self::Library(_) => false,
        }
    }

    /// The contracts of the object's linearisation, itself first: an interface base declares
    /// nothing an object runs, and a library is in no linearisation.
    pub fn contracts(&self) -> Vec<ContractDefinition> {
        match self {
            Self::Contract(node) => node
                .linearised_bases()
                .into_iter()
                .filter_map(|base| match base {
                    ContractBase::Contract(base) => Some(base),
                    ContractBase::Interface(_) => None,
                })
                .collect(),
            Self::Library(_) => Vec::new(),
        }
    }

    /// The functions the runtime code dispatches to, `receive` and `fallback` among them: a
    /// contract's over its hierarchy after resolving overrides and getter shadowing, a library's
    /// own.
    pub fn externally_visible_functions(&self) -> impl Iterator<Item = FunctionDefinition> + use<> {
        match self {
            Self::Contract(node) => node.linearised_functions(),
            Self::Library(node) => node.functions(),
        }
        .into_iter()
        .filter(FunctionDefinition::is_externally_visible)
    }

    /// The source bytes of the externally visible functions, by which the objects are scheduled
    /// for lowering: the runtime module defines what they reach, so their size tracks the work.
    pub fn estimated_lowering_cost(&self) -> usize {
        self.externally_visible_functions()
            .map(|function| function.get_text_range().len())
            .sum()
    }

    /// The state variables the object declares over its hierarchy, in storage order.
    pub fn state_variables(&self) -> Vec<StateVariableDefinition> {
        match self {
            Self::Contract(node) => node.linearised_state_variables(),
            Self::Library(node) => node.state_variables(),
        }
    }

    /// The state variables with an inline initializer, in storage order over the hierarchy.
    pub fn initialized_state_variables(&self) -> Vec<(StateVariableDefinition, Expression)> {
        self.state_variables()
            .into_iter()
            .filter(|state_variable| {
                matches!(
                    state_variable.attributes().mutability(),
                    StateVariableMutability::Mutable | StateVariableMutability::Immutable
                )
            })
            .filter_map(|state_variable| {
                let initializer = state_variable.value()?;
                Some((state_variable, initializer))
            })
            .collect()
    }

    /// Whether the constructor chain does nothing, so the deploy code is the stub.
    pub fn has_stub_creation_code(&self) -> bool {
        match self {
            Self::Contract(_) => {
                self.contracts().iter().all(|contract| {
                    contract.constructor().is_none_or(|constructor| {
                        constructor.parameters().is_empty()
                            && constructor.attributes().modifier_invocations().is_empty()
                            && constructor
                                .body()
                                .is_some_and(|body| body.statements().is_empty())
                    })
                }) && self.state_variables().iter().all(|state_variable| {
                    !matches!(
                        state_variable.attributes().mutability(),
                        StateVariableMutability::Immutable
                    )
                }) && self.initialized_state_variables().is_empty()
            }
            Self::Library(_) => false,
        }
    }

    /// The storage slot of each state variable the object stores, persistent and transient in
    /// one map keyed by definition id, or `None` when Slang cannot lay out its storage.
    pub fn storage_layout(&self) -> Option<HashMap<NodeId, StorageSlot>> {
        match self {
            Self::Contract(node) => Some(
                node.compute_storage_items()?
                    .into_iter()
                    .chain(node.compute_transient_storage_items()?)
                    .map(|item| (item.node_id(), StorageSlot::from(&item)))
                    .collect(),
            ),
            Self::Library(_) => Some(HashMap::new()),
        }
    }
}
