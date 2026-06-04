//!
//! Dynamic-array and `bytes` member built-ins: `arr.push(x)`, `arr.push()`,
//! and `arr.pop()`.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::Type as SlangType;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Lowers `arr.pop()` / `bytes.pop()` to `sol.pop`, removing the last
    /// element in place.
    pub fn emit_array_pop(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (array, block) = self
            .expression_emitter
            .emit_value(&access.operand(), block)?;
        self.expression_emitter
            .state
            .builder
            .emit_sol_pop(array, &block);
        Ok((None, block))
    }

    /// Lowers `arr.push(x)` / `arr.push()` / `bytes.push(x)`.
    ///
    /// `bytes.push(x)` uses the dedicated `sol.push_string` lowering, which
    /// handles the in-place to out-of-place storage transition at the 31-byte
    /// boundary. Every other case appends a slot with `sol.push`, then either
    /// stores the cast argument into it or — for the no-argument form — yields
    /// the freshly appended element.
    pub fn emit_array_push(
        &self,
        access: &MemberAccessExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let base = access.operand();
        let base_type = base
            .get_type()
            .expect("the binder types every array-push base");
        let value_argument = arguments.iter().next();

        if let (SlangType::Bytes(_), Some(value_expression)) = (&base_type, &value_argument) {
            let builder = &self.expression_emitter.state.builder;
            let byte_type = builder.types.fixed_bytes(1);
            let (array, block) = self.expression_emitter.emit_value(&base, block)?;
            let (value, block) = self
                .expression_emitter
                .emit_value(value_expression, block)?;
            let byte =
                TypeConversion::from_target_type(byte_type, builder).emit(value, builder, &block);
            builder.emit_sol_push_string(array, byte, &block);
            return Ok((None, block));
        }

        let (slot, element_type, block) = self.emit_push_slot(access, block)?;
        let Some(value_expression) = value_argument else {
            let value =
                self.expression_emitter
                    .state
                    .builder
                    .emit_sol_load(slot, element_type, &block)?;
            return Ok((Some(value), block));
        };
        let (value, block) = self
            .expression_emitter
            .emit_value(&value_expression, block)?;
        let builder = &self.expression_emitter.state.builder;
        let value =
            TypeConversion::from_target_type(element_type, builder).emit(value, builder, &block);
        builder.emit_sol_store(value, slot, &block);
        Ok((None, block))
    }

    /// Emits `sol.push` for `arr.push()` / `bytes.push()`, returning the new
    /// element's reference, its element type, and the continuation block.
    fn emit_push_slot(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        Value<'context, 'block>,
        Type<'context>,
        BlockRef<'context, 'block>,
    )> {
        let base = access.operand();
        let base_type = base
            .get_type()
            .expect("the binder types every array-push base");
        let builder = &self.expression_emitter.state.builder;
        let element_type = match &base_type {
            SlangType::Array(array_type) => {
                TypeConversion::resolve_slang_type(&array_type.element_type(), None, builder)
            }
            SlangType::Bytes(_) => builder.types.fixed_bytes(1),
            other => unreachable!(
                "`.push` is a member of dynamic arrays and `bytes` only; got {:?}",
                std::mem::discriminant(other)
            ),
        };
        let location = match base_type.data_location() {
            Some(SlangDataLocation::Inherited) => {
                unreachable!("an array-push base never carries an Inherited location")
            }
            Some(location) => DataLocation::from_slang(location, None),
            None => unreachable!("an array-push base is a reference type with a data location"),
        };
        let (array, block) = self.expression_emitter.emit_value(&base, block)?;
        let address_type = builder.types.pointer(element_type, location);
        let slot = builder.emit_sol_push(array, address_type, &block);
        Ok((slot, element_type, block))
    }
}
