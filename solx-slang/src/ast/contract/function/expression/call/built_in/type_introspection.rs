//!
//! `type(T).min`/`max`/`interfaceId`/`code`/`name` lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::MemberAccessExpression;

use crate::ast::contract::function::expression::call::CallEmitter;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Emits `type(E).min` / `type(E).max` for an enum.
    pub fn emit_type_enum_min_max(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let _ = (access, block);
        unimplemented!("type(E).min/max")
    }

    /// Emits `type(T).min` / `type(T).max` for an integer type.
    pub fn emit_type_min_max(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let _ = (access, block);
        unimplemented!("type(T).min/max")
    }

    /// Emits `type(I).interfaceId` (EIP-165).
    pub fn emit_type_interface_id(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let _ = (access, block);
        unimplemented!("type(I).interfaceId")
    }

    /// Emits `type(C).creationCode` / `type(C).runtimeCode` (+ `add_dependency`).
    pub fn emit_type_code(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let _ = (access, block);
        unimplemented!("type(C).creationCode/runtimeCode")
    }

    /// Emits `type(C).name` (`sol.string_lit`).
    pub fn emit_type_name(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let _ = (access, block);
        unimplemented!("type(C).name")
    }
}
