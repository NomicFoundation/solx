//!
//! Pure transformations on Slang's [`ArgumentsDeclaration`] AST node.
//!

use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;

use crate::ast::named_arguments_ext::NamedArgumentsExt;

/// Extension methods on Slang's [`ArgumentsDeclaration`] AST node.
///
/// An extension trait (NOT a slang API); a `pub trait` per the visibility rule
/// (no `pub(crate)`).
pub trait ArgumentsDeclarationExt {
    /// Orders a call / emit / revert argument list into the callee's
    /// parameter-declaration order, keyed by `parameter_ids` (NodeId identity,
    /// never name text — Rule-7). Positional arguments are already in order;
    /// named arguments are reordered via [`NamedArgumentsExt::ordered_by`].
    fn ordered_by(&self, parameter_ids: &[NodeId]) -> Vec<Expression>;
}

impl ArgumentsDeclarationExt for ArgumentsDeclaration {
    fn ordered_by(&self, parameter_ids: &[NodeId]) -> Vec<Expression> {
        match self {
            ArgumentsDeclaration::PositionalArguments(positional) => positional.iter().collect(),
            ArgumentsDeclaration::NamedArguments(named) => named.ordered_by(parameter_ids),
        }
    }
}
