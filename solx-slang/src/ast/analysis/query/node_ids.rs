//!
//! Parameter node-id query for a parameter list (pure-Slang).
//!

use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Parameters;

/// The node ids of a parameter list, in declaration order.
pub trait ParameterNodeIds {
    /// Each parameter's node id, in declaration order.
    fn node_ids(&self) -> Vec<NodeId>;
}

impl ParameterNodeIds for Parameters {
    fn node_ids(&self) -> Vec<NodeId> {
        self.iter().map(|parameter| parameter.node_id()).collect()
    }
}
