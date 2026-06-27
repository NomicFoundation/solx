//!
//! Contract-local super/base and virtual dispatch maps.
//!

use std::collections::HashMap;

use slang_solidity_v2::ast::NodeId;

use crate::ast::analysis::walk::super_call::SuperDispatch;

/// Contract-local dispatch metadata computed from C3 linearisation.
#[derive(Default)]
pub struct ContractDispatch {
    /// `super` / qualified-base member access node ID to target function node ID.
    pub super_redirect: HashMap<NodeId, NodeId>,
    /// Shadowed base function node ID to most-derived override node ID.
    pub virtual_redirect: HashMap<NodeId, NodeId>,
}

impl ContractDispatch {
    /// Builds dispatch metadata from the super-dispatch precompute pass.
    pub fn from_super_dispatch(super_dispatch: &SuperDispatch) -> Self {
        Self {
            super_redirect: super_dispatch.redirect.clone(),
            virtual_redirect: super_dispatch.virtual_redirect.clone(),
        }
    }

    /// Resolves a super/base member access target.
    pub fn resolve_super(&self, access_id: NodeId) -> Option<NodeId> {
        self.super_redirect.get(&access_id).copied()
    }

    /// Resolves a virtual function target.
    pub fn resolve_virtual(&self, definition_id: NodeId) -> NodeId {
        self.virtual_redirect
            .get(&definition_id)
            .copied()
            .unwrap_or(definition_id)
    }
}
