//!
//! `NamedArgumentsExt::ordered_by` extension trait.
//!

use std::collections::HashMap;

use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NamedArguments;
use slang_solidity_v2::ast::NodeId;

/// Extension methods on Slang's [`NamedArguments`] AST node.
pub trait NamedArgumentsExt {
    /// Reorders the named arguments into the positional order of
    /// `declaration_ids` (the callee's parameter / struct-field node ids),
    /// yielding each argument's value expression. Slang has already bound every
    /// declared name to exactly one argument.
    fn ordered_by(&self, declaration_ids: &[NodeId]) -> Vec<Expression>;
}

impl NamedArgumentsExt for NamedArguments {
    fn ordered_by(&self, declaration_ids: &[NodeId]) -> Vec<Expression> {
        let mut by_definition: HashMap<NodeId, Expression> = HashMap::new();
        for argument in self.iter() {
            let definition = argument
                .name()
                .resolve_to_definition()
                .expect("slang resolves every named argument to its target definition");
            by_definition.insert(definition.node_id(), argument.value());
        }
        declaration_ids
            .iter()
            .map(|declaration_id| {
                by_definition
                    .remove(declaration_id)
                    .expect("slang binds a named argument for every declared name")
            })
            .collect()
    }
}
