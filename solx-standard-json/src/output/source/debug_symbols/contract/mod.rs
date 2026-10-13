//!
//! A contract, interface or library in the debug symbol table.
//!

pub mod base;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::InterfaceDefinition;
use slang_solidity_v2::ast::LibraryDefinition;

use solx_mlir::ContractKind;

use super::function::Function;

use self::base::Base;

///
/// A contract, interface or library in the debug symbol table.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Contract {
    /// The declared name.
    pub name: String,
    /// Whether the definition is a contract, an interface or a library.
    pub kind: solx_mlir::ContractKind,
    /// Whether the contract is declared `abstract`.
    pub r#abstract: bool,
    /// The `[byteOffset, byteLength]` of the whole definition.
    pub range: [usize; 2],
    /// The C3 linearisation, the definition itself first. Present for contracts and interfaces.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bases: Option<Vec<Base>>,
    /// The members the definition itself declares, in declaration order.
    pub functions: Vec<Function>,
}

impl From<&ContractDefinition> for Contract {
    /// Includes the contract's own members and Slang's C3 linearisation.
    fn from(contract: &ContractDefinition) -> Self {
        let range = contract.get_text_range();
        Self {
            name: contract.name().name().to_owned(),
            kind: ContractKind::Contract,
            r#abstract: contract.is_abstract(),
            range: [range.start, range.len()],
            bases: Some(contract.linearised_bases().iter().map(Base::from).collect()),
            functions: Self::own_functions(contract.members().iter()),
        }
    }
}

impl From<&InterfaceDefinition> for Contract {
    /// Includes the interface's own members and Slang's C3 linearisation.
    fn from(interface: &InterfaceDefinition) -> Self {
        let range = interface.get_text_range();
        Self {
            name: interface.name().name().to_owned(),
            kind: ContractKind::Interface,
            r#abstract: false,
            range: [range.start, range.len()],
            bases: Some(
                interface
                    .linearised_bases()
                    .iter()
                    .map(Base::from)
                    .collect(),
            ),
            functions: Self::own_functions(interface.members().iter()),
        }
    }
}

impl From<&LibraryDefinition> for Contract {
    /// Includes the library's own members.
    fn from(library: &LibraryDefinition) -> Self {
        let range = library.get_text_range();
        Self {
            name: library.name().name().to_owned(),
            kind: ContractKind::Library,
            r#abstract: false,
            range: [range.start, range.len()],
            bases: None,
            functions: Self::own_functions(library.members().iter()),
        }
    }
}

impl Contract {
    /// Preserves declaration order across functions, modifiers and public variable getters.
    fn own_functions(members: impl Iterator<Item = ContractMember>) -> Vec<Function> {
        members
            .filter_map(|member| match member {
                ContractMember::FunctionDefinition(function) => Some(Function::from(&function)),
                ContractMember::StateVariableDefinition(state_variable)
                    if state_variable.is_externally_visible() =>
                {
                    Some(Function::from(&state_variable))
                }
                _ => None,
            })
            .collect()
    }
}
