//!
//! Internal / external library call lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::contract::function::expression::call::CallEmitter;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Emits an internal (`Library { external: false }`) library call — inlined
    /// like an ordinary internal function.
    pub fn emit_library_call(
        &self,
        access: &MemberAccessExpression,
        library_function: &FunctionDefinition,
        positional_arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let _ = (access, library_function, positional_arguments, block);
        unimplemented!("internal library call")
    }

    /// Emits an external (`Library { external: true }`) library call — a
    /// `delegatecall` to the deployed library.
    pub fn emit_library_external_call(
        &self,
        library_name: &str,
        function: &FunctionDefinition,
        arguments: &PositionalArguments,
        self_receiver: Option<&Expression>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let _ = (library_name, function, arguments, self_receiver, block);
        unimplemented!("external library delegatecall")
    }

    /// Re-raises a bubbled revert (`returndatacopy` + `revert`). Oracle free
    /// assoc fn → `&self` method (Rule-5).
    pub fn emit_bubble_revert(&self, block: &BlockRef<'context, 'block>) {
        let _ = block;
        unimplemented!("bubble revert")
    }
}
