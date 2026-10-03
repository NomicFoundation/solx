//!
//! The per-source symbol table for debuggers and stack tracers, read off Slang's semantic model.
//!

mod spans;

use slang_solidity_v2::ast::ContractBase;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind as SlangFunctionKind;
use slang_solidity_v2::ast::FunctionMutability;
use slang_solidity_v2::ast::FunctionVisibility;
use slang_solidity_v2::ast::InterfaceDefinition;
use slang_solidity_v2::ast::LibraryDefinition;
use slang_solidity_v2::ast::NodeLocation;
use slang_solidity_v2::ast::SourceUnit;
use slang_solidity_v2::ast::SourceUnitMember;
use slang_solidity_v2::ast::StateVariableDefinition;

use solx_standard_json::output::source::debug_symbols::DebugSymbols;
use solx_standard_json::output::source::debug_symbols::contract::Base;
use solx_standard_json::output::source::debug_symbols::contract::Contract;
use solx_standard_json::output::source::debug_symbols::contract::ContractKind;
use solx_standard_json::output::source::debug_symbols::function::Function;
use solx_standard_json::output::source::debug_symbols::function::FunctionKind;
use solx_standard_json::output::source::debug_symbols::function::Mutability;
use solx_standard_json::output::source::debug_symbols::function::Visibility;

use crate::scope::source_unit::SourceUnitScope;

use self::spans::Spans;

/// The symbol table of a source unit, filled member by member in source order.
#[derive(Default)]
pub struct SymbolTable {
    /// The contracts, interfaces and libraries.
    contracts: Vec<Contract>,
    /// The free functions.
    free_functions: Vec<Function>,
}

impl SymbolTable {
    /// The symbol table of `unit`: its contracts, interfaces and libraries with their own members,
    /// its free functions, and the spans of its nonterminal nodes.
    pub fn build(unit: &SourceUnit) -> DebugSymbols {
        let mut table = Self::default();
        for member in unit.members().iter() {
            table.member(&member);
        }
        DebugSymbols {
            contracts: table.contracts,
            free_functions: table.free_functions,
            spans: Spans::collect(unit),
        }
    }

    /// Adds a source unit member, if it is a definition the table lists.
    fn member(&mut self, member: &SourceUnitMember) {
        match member {
            SourceUnitMember::ContractDefinition(contract) => {
                self.contracts.push(Self::contract(contract));
            }
            SourceUnitMember::InterfaceDefinition(interface) => {
                self.contracts.push(Self::interface(interface));
            }
            SourceUnitMember::LibraryDefinition(library) => {
                self.contracts.push(Self::library(library));
            }
            SourceUnitMember::FunctionDefinition(function) => {
                self.free_functions
                    .push(Self::function(function, FunctionKind::Free));
            }
            _ => {}
        }
    }

    /// A contract with its C3 linearisation.
    fn contract(contract: &ContractDefinition) -> Contract {
        Contract {
            name: contract.name().name().to_owned(),
            kind: ContractKind::Contract,
            is_abstract: contract.is_abstract(),
            range: Self::range(contract),
            bases: Some(
                contract
                    .linearised_bases()
                    .iter()
                    .map(|base| match base {
                        ContractBase::Contract(base) => {
                            Self::base(base.get_file_id().as_str(), base.name().name())
                        }
                        ContractBase::Interface(base) => {
                            Self::base(base.get_file_id().as_str(), base.name().name())
                        }
                    })
                    .collect(),
            ),
            functions: Self::own_functions(contract.members().iter()),
        }
    }

    /// An interface, without a linearisation.
    fn interface(interface: &InterfaceDefinition) -> Contract {
        Contract {
            name: interface.name().name().to_owned(),
            kind: ContractKind::Interface,
            is_abstract: false,
            range: Self::range(interface),
            bases: None,
            functions: Self::own_functions(interface.members().iter()),
        }
    }

    /// A library. A library cannot inherit, so its linearisation is itself.
    fn library(library: &LibraryDefinition) -> Contract {
        Contract {
            name: library.name().name().to_owned(),
            kind: ContractKind::Library,
            is_abstract: false,
            range: Self::range(library),
            bases: Some(vec![Self::base(
                library.get_file_id().as_str(),
                library.name().name(),
            )]),
            functions: Self::own_functions(library.members().iter()),
        }
    }

    /// A definition's `[offset, length]`.
    fn range(node: &impl NodeLocation) -> [usize; 2] {
        let range = node
            .calculate_text_range()
            .expect("every definition covers source text");
        [range.start, range.len()]
    }

    fn base(file: &str, name: &str) -> Base {
        Base {
            file: file.to_owned(),
            name: name.to_owned(),
        }
    }

    /// The functions, modifiers and public state variable getters a definition itself declares, in
    /// declaration order.
    fn own_functions(members: impl Iterator<Item = ContractMember>) -> Vec<Function> {
        members
            .filter_map(|member| match member {
                ContractMember::FunctionDefinition(function) => {
                    let kind = match function.kind() {
                        SlangFunctionKind::Regular => FunctionKind::Function,
                        SlangFunctionKind::Constructor => FunctionKind::Constructor,
                        SlangFunctionKind::Fallback => FunctionKind::Fallback,
                        SlangFunctionKind::Receive => FunctionKind::Receive,
                        SlangFunctionKind::Modifier => FunctionKind::Modifier,
                    };
                    Some(Self::function(&function, kind))
                }
                ContractMember::StateVariableDefinition(variable)
                    if variable.is_externally_visible() =>
                {
                    Some(Self::getter(&variable))
                }
                _ => None,
            })
            .collect()
    }

    /// A declared function or modifier.
    fn function(function: &FunctionDefinition, kind: FunctionKind) -> Function {
        let attributes = function.attributes();
        Function {
            name: SourceUnitScope::function_name(function),
            kind,
            visibility: match attributes.visibility() {
                FunctionVisibility::Public => Visibility::Public,
                FunctionVisibility::External => Visibility::External,
                FunctionVisibility::Internal => Visibility::Internal,
                FunctionVisibility::Private => Visibility::Private,
            },
            mutability: match attributes.mutability() {
                FunctionMutability::Pure => Mutability::Pure,
                FunctionMutability::View => Mutability::View,
                FunctionMutability::NonPayable => Mutability::NonPayable,
                FunctionMutability::Payable => Mutability::Payable,
            },
            selector: function
                .compute_selector()
                .map(SourceUnitScope::selector_hex),
            range: Self::range(function),
            implemented: function.body().is_some(),
        }
    }

    /// The getter of a public state variable, located at the variable's definition.
    fn getter(variable: &StateVariableDefinition) -> Function {
        Function {
            name: variable.name().name().to_owned(),
            kind: FunctionKind::Getter,
            visibility: Visibility::Public,
            mutability: Mutability::View,
            selector: variable
                .compute_selector()
                .map(SourceUnitScope::selector_hex),
            range: Self::range(variable),
            implemented: true,
        }
    }
}
