//!
//! The argument lists of call-shaped constructs, emitted in definition parameter order.
//!

use std::collections::HashMap;

use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NamedArguments;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Parameter;
use slang_solidity_v2::ast::Parameters;

use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// Emits each argument in the definition's parameter order, converted to and paired with its
    /// parameter.
    pub fn arguments_declaration(
        &mut self,
        arguments: &ArgumentsDeclaration,
        parameters: &Parameters,
    ) -> Vec<(Parameter, Value<'context>)> {
        let ordered: Vec<Expression> = match arguments {
            ArgumentsDeclaration::PositionalArguments(positional) => positional.iter().collect(),
            ArgumentsDeclaration::NamedArguments(named) => Self::named_arguments(
                named,
                parameters.iter().map(|parameter| parameter.node_id()),
            ),
        };
        parameters
            .iter()
            .zip(ordered)
            .map(|(parameter, argument)| {
                let parameter_type = self.typing(parameter.get_type());
                let value = self.converted(&argument, parameter_type);
                (parameter, value)
            })
            .collect()
    }

    /// The positional argument list of a call, each argument evaluated in order.
    pub fn positional_arguments(&mut self, arguments: &[Expression]) -> Vec<Value<'context>> {
        arguments
            .iter()
            .map(|argument| self.expression(argument))
            .collect()
    }

    /// The named argument list of a call, reordered into the targets' declaration order through
    /// the label bindings slang resolves.
    pub fn named_arguments(
        named: &NamedArguments,
        targets: impl Iterator<Item = NodeId>,
    ) -> Vec<Expression> {
        let mut by_target: HashMap<NodeId, Expression> = named
            .iter()
            .map(|argument| {
                (
                    argument
                        .name()
                        .resolve_to_definition()
                        .expect("slang binds every named-argument label")
                        .node_id(),
                    argument.value(),
                )
            })
            .collect();
        targets
            .map(|target| {
                by_target
                    .remove(&target)
                    .expect("slang validates every parameter receives a named argument")
            })
            .collect()
    }
}
