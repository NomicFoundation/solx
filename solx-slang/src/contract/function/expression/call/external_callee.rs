//!
//! An externally dispatched callee: the definition its selector and symbol derive from.
//!

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionType;
use slang_solidity_v2::ast::StateVariableDefinition;

use solx_mlir::Function;

use crate::scope::source_unit::SourceUnitScope;

/// An externally dispatched callee, each variant carrying the ABI selector its admission
/// established.
pub enum ExternalCallee {
    /// A contract function.
    Function(FunctionDefinition, u32),
    /// A public state variable, dispatched through its getter.
    Getter(StateVariableDefinition, u32),
}

impl ExternalCallee {
    /// Classifies the definition a callee member resolves to, admitting each kind only with a
    /// selector.
    pub fn from_definition(definition: Definition) -> Option<Self> {
        match definition {
            Definition::Function(function_definition) => function_definition
                .compute_selector()
                .map(|selector| Self::Function(function_definition, selector)),
            Definition::StateVariable(state_variable) => state_variable
                .compute_selector()
                .map(|selector| Self::Getter(state_variable, selector)),
            _ => None,
        }
    }

    /// The callee's symbol and MLIR signature; `function_type` is the operand's.
    pub fn function<'context>(
        &self,
        function_type: &FunctionType,
        source_unit: &SourceUnitScope<'context>,
    ) -> Function<'context> {
        let symbol = match self {
            Self::Function(function_definition, _) => {
                SourceUnitScope::function_symbol(function_definition)
            }
            Self::Getter(state_variable, _) => SourceUnitScope::getter_symbol(state_variable),
        };
        Function::new(symbol, source_unit.function_type(function_type))
    }

    /// The ABI selector the callee dispatches on.
    pub fn selector(&self) -> u32 {
        let (Self::Function(_, selector) | Self::Getter(_, selector)) = self;
        *selector
    }
}
