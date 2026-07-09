//!
//! The Contract declaration entity: emits `sol.contract` and its `sol.state_var` members.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::r#type::IntegerType;
use ruint::aliases::U256;

use crate::Context;
use crate::ContractKind;
use crate::Type;
use crate::ods::sol::ContractOperation;
use crate::ods::sol::StateVarOperation;

/// A `sol.contract` declaration and the insertion point for its `sol.state_var` members.
#[derive(Clone, Copy)]
pub struct Contract<'context, 'block> {
    /// The contract's body region entry block, where its members and functions are emitted.
    pub body: BlockRef<'context, 'block>,
}

impl<'context, 'block> Contract<'context, 'block> {
    /// Emits `sol.contract @name` of `kind` into `module_body`, returning it wrapping the body region.
    pub fn define<B>(
        name: &str,
        kind: ContractKind,
        context: &Context<'context>,
        module_body: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let body = mlir_region_op!(
            context, module_body,
            ContractOperation
                .sym_name(StringAttribute::new(context.melior, name))
                .kind(kind.attribute(context.melior));
            body_region
        );
        Self { body }
    }

    /// Emits a `sol.state_var @name` member of `element_type` at storage `slot`/`byte_offset`.
    pub fn declare_state_var(
        self,
        name: &str,
        element_type: Type<'context>,
        slot: U256,
        byte_offset: u32,
        context: &Context<'context>,
    ) {
        mlir_op_void!(
            context,
            &self.body,
            StateVarOperation
                .sym_name(StringAttribute::new(context.melior, name))
                .r#type(TypeAttribute::new(element_type.into_mlir()))
                .slot(IntegerAttribute::from_words(
                    IntegerType::new(context.melior, solx_utils::BIT_LENGTH_FIELD as u32).into(),
                    slot.as_limbs(),
                ))
                .byte_offset(IntegerAttribute::new(
                    IntegerType::new(context.melior, solx_utils::BIT_LENGTH_X32 as u32).into(),
                    byte_offset.into(),
                ))
        );
    }
}
