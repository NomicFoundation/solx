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

    /// The object's identifier, qualified by its file: linking keys objects by it, and two files
    /// may declare the same name.
    pub fn identifier(&self) -> String {
        let file_id = match self {
            Self::Contract(node) => node.get_file_id(),
            Self::Library(node) => node.get_file_id(),
        };
        solx_utils::ContractName::full_path(file_id.as_str(), self.name().name())
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

    /// A contract's functions over its hierarchy after resolving overrides and getter shadowing;
    /// a library's own functions.
    pub fn functions(&self) -> Vec<FunctionDefinition> {
        match self {
            Self::Contract(node) => node.linearised_functions(),
            Self::Library(node) => node.functions(),
        }
    }

    /// The functions the runtime code dispatches to: the externally visible ones, `receive` and
    /// `fallback` among them.
    pub fn entry_points(&self) -> Vec<FunctionDefinition> {
        self.functions()
            .into_iter()
            .filter(FunctionDefinition::is_externally_visible)
            .collect()
    }

    /// The state variables the object declares over its hierarchy, in storage order.
    pub fn state_variables(&self) -> Vec<StateVariableDefinition> {
        match self {
            Self::Contract(node) => node.linearised_state_variables(),
            Self::Library(node) => node.state_variables(),
        }
    }
}
