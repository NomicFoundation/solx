//!
//! The per-source symbol table for debuggers and stack tracers, read off Slang's semantic model.
//!

mod spans;

use std::ops::Range;

use slang_solidity_v2::ast::ContractBase;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind as SlangFunctionKind;
use slang_solidity_v2::ast::FunctionMutability;
use slang_solidity_v2::ast::FunctionVisibility;
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

/// The symbol table of `unit`: its contracts, interfaces and libraries with their own members,
/// its free functions, and the spans of its nonterminal nodes.
pub fn symbol_table(unit: &SourceUnit) -> DebugSymbols {
    let mut contracts = Vec::new();
    let mut free_functions = Vec::new();
    for member in unit.members().iter() {
        match member {
            SourceUnitMember::ContractDefinition(contract) => contracts.push(Contract {
                name: contract.name().name().to_owned(),
                kind: ContractKind::Contract,
                is_abstract: contract.is_abstract(),
                range: offset_length(contract.get_text_range()),
                bases: Some(
                    contract
                        .linearised_bases()
                        .iter()
                        .map(|base| match base {
                            ContractBase::Contract(base) => {
                                self::base(base.get_file_id().as_str(), base.name().name())
                            }
                            ContractBase::Interface(base) => {
                                self::base(base.get_file_id().as_str(), base.name().name())
                            }
                        })
                        .collect(),
                ),
                functions: own_functions(contract.members().iter()),
            }),
            SourceUnitMember::InterfaceDefinition(interface) => contracts.push(Contract {
                name: interface.name().name().to_owned(),
                kind: ContractKind::Interface,
                is_abstract: false,
                range: offset_length(interface.get_text_range()),
                bases: None,
                functions: own_functions(interface.members().iter()),
            }),
            SourceUnitMember::LibraryDefinition(library) => contracts.push(Contract {
                name: library.name().name().to_owned(),
                kind: ContractKind::Library,
                is_abstract: false,
                range: offset_length(library.get_text_range()),
                bases: Some(vec![self::base(
                    library.get_file_id().as_str(),
                    library.name().name(),
                )]),
                functions: own_functions(library.members().iter()),
            }),
            SourceUnitMember::FunctionDefinition(function) => {
                free_functions.push(self::function(&function, FunctionKind::Free));
            }
            _ => {}
        }
    }
    DebugSymbols {
        contracts,
        free_functions,
        spans: spans::spans(unit),
    }
}

/// A Slang text range as `[offset, length]`.
fn offset_length(range: &Range<usize>) -> [usize; 2] {
    [range.start, range.end - range.start]
}

/// A linearisation entry.
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
                Some(self::function(&function, kind))
            }
            ContractMember::StateVariableDefinition(variable)
                if variable.is_externally_visible() =>
            {
                Some(getter(&variable))
            }
            _ => None,
        })
        .collect()
}

/// A declared function or modifier. Its selector is computed as `evm.methodIdentifiers` computes
/// it.
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
        selector: matches!(kind, FunctionKind::Function)
            .then(|| function.compute_selector())
            .flatten()
            .map(|selector| format!("{selector:08x}")),
        range: offset_length(function.get_text_range()),
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
            .map(|selector| format!("{selector:08x}")),
        range: offset_length(variable.get_text_range()),
        implemented: true,
    }
}
