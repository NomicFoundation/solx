//!
//! One bound modifier parameter for a modifier stage.
//!

use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::NodeId;

/// One bound modifier parameter for a stage: the parameter declaration's node id
/// (keys the [`NodeId`]-keyed environment), the stack slot holding its evaluated
/// argument, and the slot's element type. Replaces a `(NodeId, Value, Type)`
/// tuple.
#[derive(Clone, Copy)]
pub struct ModifierParameterBinding<'context, 'env> {
    /// The parameter declaration's node id.
    pub declaration: NodeId,
    /// The stack slot holding the evaluated argument.
    pub pointer: Value<'context, 'env>,
    /// The slot's element type.
    pub element_type: Type<'context>,
}
