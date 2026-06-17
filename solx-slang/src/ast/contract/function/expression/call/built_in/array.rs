//!
//! The dynamic-array / `bytes` push-slot primitive shared by `arr.push` emission
//! and the `arr.push() = v` lvalue.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;
use solx_mlir::ods::sol::PushOperation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::LocationPolicy;
use crate::ast::Type as AstType;
use crate::ast::contract::function::expression::ExpressionContext;

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Appends a default element to a dynamic array (or `bytes`) and returns the
    /// freshly-allocated slot reference together with its element MLIR type.
    /// Shared by `arr.push(x)` (which then stores `x` into the slot) and the
    /// `arr.push() = v` lvalue (which copies the right-hand side into it).
    pub fn emit_push_slot(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> (
        Value<'context, 'block>,
        Type<'context>,
        BlockRef<'context, 'block>,
    ) {
        let base = access.operand();
        let base_slang_type = base.get_type().expect("slang validated");
        let builder = &self.state.builder;
        let (element_type, slang_location) = match &base_slang_type {
            SlangType::Array(array_type) => (
                AstType::resolve(
                    &array_type.element_type(),
                    LocationPolicy::Declared(None),
                    builder,
                ),
                array_type.location(),
            ),
            SlangType::Bytes(bytes_type) => (
                AstType::fixed_bytes(builder.context, 1).into_mlir(),
                bytes_type.location(),
            ),
            other => unreachable!(
                "Solidity's .push is a member of dynamic arrays and bytes only; got {:?}",
                std::mem::discriminant(other)
            ),
        };
        let base_location = match slang_location {
            SlangDataLocation::Inherited => {
                unreachable!("slang's binder should not surface Inherited at an array push base")
            }
            other => solx_utils::DataLocation::from_slang(other, None),
        };
        let BlockAnd {
            value: array_value,
            block,
        } = base.emit(self, block);
        // solc's `sol.push` yields the new element's reference type directly when
        // the element is a reference type (nested array / struct / string) — the
        // slot is then copied into via `sol.copy` — and a `!sol.ptr` to the
        // element when it is a value type, stored into via `sol.store`. Mirror
        // that: a reference element pushed to a pointer would force a
        // memory→storage data-location cast the backend cannot lower.
        let push_result_type = if AstType::new(element_type).is_reference() {
            element_type
        } else {
            AstType::pointer(builder.context, element_type, base_location).into_mlir()
        };
        let new_slot = mlir_op!(
            builder,
            &block,
            PushOperation.inp(array_value).addr(push_result_type)
        );
        (new_slot, element_type, block)
    }
}
