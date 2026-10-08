//!
//! The deployable object a module emits: a contract or a library.
//!

use slang_solidity_v2::ast::ContractBase;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::Identifier;
use slang_solidity_v2::ast::LibraryDefinition;
use slang_solidity_v2::ast::StateVariableDefinition;

use solx_mlir::ContractKind;

use crate::abi::Abi;

/// The deployable object a module emits, each variant carrying the definition its kind
/// dispatches from and its ABI.
pub enum Object {
    /// A contract: storage, a constructor, selector dispatch.
    Contract(ContractDefinition, Abi),
    /// A library: no storage, no constructor, `DELEGATECALL` dispatch.
    Library(LibraryDefinition, Abi),
}

impl Object {
    /// The object's name.
    pub fn name(&self) -> Identifier {
        match self {
            Self::Contract(node, _) => node.name(),
            Self::Library(node, _) => node.name(),
        }
    }

    /// The object's identifier, qualified by its file: linking keys objects by it, and two files
    /// may declare the same name.
    pub fn identifier(&self) -> String {
        match self {
            Self::Contract(node, _) => Self::contract_identifier(node),
            Self::Library(node, _) => Self::library_identifier(node),
        }
    }

    /// The identifier of the object `contract` deploys.
    pub fn contract_identifier(contract: &ContractDefinition) -> String {
        solx_utils::ContractName::full_path(contract.get_file_id().as_str(), contract.name().name())
    }

    /// The identifier of the object `library` deploys.
    pub fn library_identifier(library: &LibraryDefinition) -> String {
        solx_utils::ContractName::full_path(library.get_file_id().as_str(), library.name().name())
    }

    /// The identifier of the object `definition` deploys, or `None` for a definition other than a
    /// contract or a library.
    pub fn definition_identifier(definition: &Definition) -> Option<String> {
        match definition {
            Definition::Contract(contract) => Some(Self::contract_identifier(contract)),
            Definition::Library(library) => Some(Self::library_identifier(library)),
            _ => None,
        }
    }

    /// The object's ABI.
    pub fn abi(&self) -> &Abi {
        match self {
            Self::Contract(_, abi) | Self::Library(_, abi) => abi,
        }
    }

    /// The objects the deploy code may embed, its runtime object leading.
    pub fn deploy_dependencies(&self) -> solx_utils::Dependencies {
        let definitions = match self {
            Self::Contract(node, _) => node.creation_bytecode_dependencies(),
            Self::Library(_, _) => Vec::new(),
        };
        let deploy_identifier = self.identifier();
        let runtime_identifier =
            solx_utils::Dependencies::runtime_identifier(deploy_identifier.as_str());
        Self::dependencies(deploy_identifier, Some(runtime_identifier), definitions)
    }

    /// The objects the runtime code may embed.
    pub fn runtime_dependencies(&self) -> solx_utils::Dependencies {
        let definitions = match self {
            Self::Contract(node, _) => node.deployed_bytecode_dependencies(),
            Self::Library(node, _) => node.deployed_bytecode_dependencies(),
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
            Self::definition_identifier(&definition)
                .expect("Slang bytecode dependencies are contracts or libraries")
        });

        solx_utils::Dependencies::new(code_identifier.as_str(), runtime_identifier, dependencies)
    }

    /// The kind the object's `sol.contract` declares.
    pub fn kind(&self) -> ContractKind {
        match self {
            Self::Contract(_, _) => ContractKind::Contract,
            Self::Library(_, _) => ContractKind::Library,
        }
    }

    /// Whether the object accepts value: a library is reached by `DELEGATECALL` alone, which
    /// carries the caller's.
    pub fn is_payable(&self) -> bool {
        match self {
            Self::Contract(node, _) => node.is_payable(),
            Self::Library(_, _) => false,
        }
    }

    /// The contracts of the object's linearisation, itself first: an interface base declares
    /// nothing an object runs, and a library is in no linearisation.
    pub fn contracts(&self) -> Vec<ContractDefinition> {
        match self {
            Self::Contract(node, _) => node
                .linearised_bases()
                .into_iter()
                .filter_map(|base| match base {
                    ContractBase::Contract(base) => Some(base),
                    ContractBase::Interface(_) => None,
                })
                .collect(),
            Self::Library(_, _) => Vec::new(),
        }
    }

    /// A contract's functions over its hierarchy after resolving overrides and getter shadowing;
    /// a library's own functions.
    pub fn functions(&self) -> Vec<FunctionDefinition> {
        match self {
            Self::Contract(node, _) => node.linearised_functions(),
            Self::Library(node, _) => node.functions(),
        }
    }

    /// The state variables the object declares over its hierarchy, in storage order.
    pub fn state_variables(&self) -> Vec<StateVariableDefinition> {
        match self {
            Self::Contract(node, _) => node.linearised_state_variables(),
            Self::Library(node, _) => node.state_variables(),
        }
    }
}
