//!
//! An expression evaluated in statement position for its effects.
//!

use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Expression as SlangExpression;

use crate::contract::function::expression::Expression;
use crate::contract::function::expression::literal::StringExpression;

codegen!(
    /// An expression evaluated for its effect. A `revert(..)` call expression is not an
    /// expression to the binder, so it is recognized by callee name and lowered to `sol.revert`
    /// directly.
    ExpressionStatement -> Effect |node, scope| {
        let expression = node.expression();
        if let SlangExpression::FunctionCallExpression(call) = &expression
            && let SlangExpression::Identifier(identifier) = call.operand()
            && identifier.name() == "revert"
        {
            let ArgumentsDeclaration::PositionalArguments(positional_arguments) =
                &call.arguments()
            else {
                unreachable!("revert call uses positional arguments");
            };
            let signature: String = match positional_arguments.iter().next() {
                None => String::new(),
                Some(SlangExpression::StringExpression(string_expression)) => {
                    let message = StringExpression::text(&string_expression);
                    if message.is_empty() {
                        unimplemented!(
                            "revert with an empty reason is not yet supported; use revert() for a no-data revert"
                        );
                    }
                    message
                }
                Some(_) => unreachable!("revert message is a string literal"),
            };
            scope
                .current_block()
                .revert(&signature, &[], scope);
            return;
        }
        Expression::emit_for_effect(&expression, scope);
    }
);
