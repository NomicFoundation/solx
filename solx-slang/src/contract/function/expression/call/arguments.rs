//!
//! The argument lists of call-shaped constructs, emitted in definition parameter order.
//!

use std::collections::HashMap;

use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NamedArguments;
use slang_solidity_v2::ast::Parameter;
use slang_solidity_v2::ast::Parameters;
use slang_solidity_v2::ast::PositionalArguments;

use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// Emits each argument in the definition's parameter order, coerced to and paired with its
    /// parameter.
    pub fn arguments_declaration(
        &mut self,
        arguments: &ArgumentsDeclaration,
        parameters: &Parameters,
    ) -> Vec<(Parameter, Value<'context>)> {
        let ordered: Vec<Expression> = match arguments {
            ArgumentsDeclaration::PositionalArguments(positional) => positional.iter().collect(),
            ArgumentsDeclaration::NamedArguments(named) => Self::named_arguments(named, parameters),
        };
        parameters
            .iter()
            .zip(ordered)
            .map(|(parameter, argument)| {
                let parameter_type = self.typing(parameter.get_type());
                let value = self.coerced(&argument, parameter_type);
                (parameter, value)
            })
            .collect()
    }

    /// The positional argument list of a call, each argument evaluated in order.
    pub fn positional_arguments(&mut self, node: &PositionalArguments) -> Vec<Value<'context>> {
        node.iter()
            .map(|argument| self.expression(&argument))
            .collect()
    }

    /// The named argument list of a call, reordered into the definition's parameter order by the
    /// name matching slang has validated is total and unambiguous.
    fn named_arguments(named: &NamedArguments, parameters: &Parameters) -> Vec<Expression> {
        let mut by_name: HashMap<String, Expression> = named
            .iter()
            .map(|argument| (argument.name().name(), argument.value()))
            .collect();
        parameters
            .iter()
            .map(|parameter| {
                let name = parameter
                    .name()
                    .expect("slang validates a named argument targets a named parameter")
                    .name();
                by_name
                    .remove(&name)
                    .expect("slang validates every parameter receives a named argument")
            })
            .collect()
    }
}
