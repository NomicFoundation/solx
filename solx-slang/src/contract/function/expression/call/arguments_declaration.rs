//!
//! The argument list of a call-shaped construct, emitted in its definition's parameter order.
//!

use std::collections::HashMap;

use slang_solidity_v2::ast::ArgumentsDeclaration as SlangArgumentsDeclaration;
use slang_solidity_v2::ast::Expression as SlangExpression;
use slang_solidity_v2::ast::Parameter as SlangParameter;
use slang_solidity_v2::ast::Parameters;

use solx_mlir::Value;

use crate::contract::function::expression::Expression;
use crate::contract::function::parameter::Parameter;
use crate::scope::FunctionScope;

codegen!(
    ArgumentsDeclaration {
        /// Emits each argument in the definition's parameter order, coerced to and paired with its
        /// parameter. Positional arguments are already in order; named arguments are matched to
        /// parameters by name, which slang has validated is total and unambiguous.
        pub fn emit_ordered<'context>(
            arguments: &SlangArgumentsDeclaration,
            parameters: &Parameters,
            scope: &mut FunctionScope<'_, '_, 'context>,
        ) -> Vec<(SlangParameter, Value<'context>)> {
            let ordered: Vec<SlangExpression> = match arguments {
                SlangArgumentsDeclaration::PositionalArguments(positional) => {
                    positional.iter().collect()
                }
                SlangArgumentsDeclaration::NamedArguments(named) => {
                    let mut by_name: HashMap<String, SlangExpression> = named
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
            };
            parameters
                .iter()
                .zip(ordered)
                .map(|(parameter, argument)| {
                    let parameter_type = Parameter::resolve(&parameter, scope);
                    let value =
                        Expression::emit(&argument, scope).coerce(parameter_type, scope);
                    (parameter, value)
                })
                .collect()
        }
    }
);
