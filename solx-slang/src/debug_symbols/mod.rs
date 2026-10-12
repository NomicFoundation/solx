//!
//! The per-source symbol table for debuggers and stack tracers, read off Slang's semantic model.
//!

mod spans;

use std::ops::Range;

use slang_solidity_v2::ast::ContractBase;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind as SlangFunctionKind;
use slang_solidity_v2::ast::FunctionType;
use slang_solidity_v2::ast::FunctionTypeVisibility;
use slang_solidity_v2::ast::FunctionVisibility;
use slang_solidity_v2::ast::SourceUnit;
use slang_solidity_v2::ast::SourceUnitMember;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::Type;

use solx_mlir::ContractKind;
use solx_standard_json::output::source::debug_symbols::DebugSymbols;
use solx_standard_json::output::source::debug_symbols::contract::Contract;
use solx_standard_json::output::source::debug_symbols::contract::base::Base;
use solx_standard_json::output::source::debug_symbols::function::Function;
use solx_standard_json::output::source::debug_symbols::function::kind::FunctionKind;
use solx_standard_json::output::source::debug_symbols::function::visibility::Visibility;

use crate::abi::AbiDefinition;
use crate::scope::source_unit::SourceUnitScope;

use self::spans::Spans;

/// The symbol table of a source unit, filled member by member in source order.
#[derive(Default)]
pub struct DebugSymbolsBuilder {
    /// The contracts, interfaces and libraries.
    contracts: Vec<Contract>,
    /// The free functions.
    free_functions: Vec<Function>,
}

impl DebugSymbolsBuilder {
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
        if let SourceUnitMember::FunctionDefinition(function) = member {
            self.free_functions
                .push(Self::function(function, FunctionKind::Free));
        }
        if let Some(definition) = AbiDefinition::from_member(member) {
            self.contracts.push(Self::contract(&definition));
        }
    }

    /// A contract, interface or library with its own members.
    fn contract(definition: &AbiDefinition) -> Contract {
        let name = definition.name().name().to_owned();
        match definition {
            AbiDefinition::Contract(node) => Contract {
                name,
                kind: ContractKind::Contract,
                r#abstract: node.is_abstract(),
                range: Self::range(node.get_text_range()),
                bases: Some(Self::bases(node.linearised_bases())),
                functions: Self::own_functions(node.members().iter()),
            },
            AbiDefinition::Interface(node) => Contract {
                name,
                kind: ContractKind::Interface,
                r#abstract: false,
                range: Self::range(node.get_text_range()),
                bases: Some(Self::bases(node.linearised_bases())),
                functions: Self::own_functions(node.members().iter()),
            },
            AbiDefinition::Library(node) => Contract {
                name,
                kind: ContractKind::Library,
                r#abstract: false,
                range: Self::range(node.get_text_range()),
                // Slang gives a library no linearisation.
                bases: None,
                functions: Self::own_functions(node.members().iter()),
            },
        }
    }

    /// A definition's `[offset, length]`.
    fn range(range: &Range<usize>) -> [usize; 2] {
        [range.start, range.len()]
    }

    fn bases(linearised_bases: Vec<ContractBase>) -> Vec<Base> {
        linearised_bases
            .iter()
            .map(|base| match base {
                ContractBase::Contract(base) => {
                    Self::base(base.get_file_id().as_str(), base.name().name())
                }
                ContractBase::Interface(base) => {
                    Self::base(base.get_file_id().as_str(), base.name().name())
                }
            })
            .collect()
    }

    fn base(path: &str, name: &str) -> Base {
        Base {
            path: path.to_owned(),
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
                ContractMember::StateVariableDefinition(state_variable)
                    if state_variable.is_externally_visible() =>
                {
                    let Some(Type::Function(getter)) = state_variable.getter_type() else {
                        unreachable!("slang types a public state variable's getter as a function");
                    };
                    Some(Self::getter(&state_variable, &getter))
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
            state_mutability: attributes.mutability().into(),
            selector: function
                .compute_selector()
                .map(|selector| format!("{selector:08x}")),
            range: Self::range(function.get_text_range()),
            implemented: function.body().is_some(),
        }
    }

    /// The getter of a public state variable, located at the variable's definition.
    fn getter(state_variable: &StateVariableDefinition, getter: &FunctionType) -> Function {
        Function {
            name: state_variable.name().name().to_owned(),
            kind: FunctionKind::Getter,
            visibility: match getter.visibility() {
                FunctionTypeVisibility::Public => Visibility::Public,
                FunctionTypeVisibility::External => Visibility::External,
                FunctionTypeVisibility::Internal => Visibility::Internal,
                FunctionTypeVisibility::Private => Visibility::Private,
            },
            state_mutability: getter.mutability().into(),
            selector: state_variable
                .compute_selector()
                .map(|selector| format!("{selector:08x}")),
            range: Self::range(state_variable.get_text_range()),
            implemented: true,
        }
    }
}
