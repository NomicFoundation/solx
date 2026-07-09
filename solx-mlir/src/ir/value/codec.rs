//!
//! ABI codec value producers: `abi.encode*` and `abi.decode`.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;

use crate::Context;
use crate::Type;
use crate::Value;
use crate::ods::sol::DecodeOperation;
use crate::ods::sol::EncodeOperation;

impl<'context, 'block> Value<'context, 'block> {
    /// Emits `sol.encode` producing a `bytes memory` payload from `inputs`.
    ///
    /// A `selector`, when present, is prepended as the first four bytes and must already be
    /// `!sol.fixed_bytes<4>`. `packed` selects the ABI-packed encoding (no per-element padding).
    pub fn encode<B>(
        inputs: &[Self],
        selector: Option<Self>,
        packed: bool,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let inputs = inputs
            .iter()
            .map(|element| element.into_mlir())
            .collect::<Vec<_>>();
        let memory = Type::string(context.melior, solx_utils::DataLocation::Memory).into_mlir();
        let mut builder = EncodeOperation::builder(context.melior, context.location())
            .ins(inputs.as_slice())
            .res(memory);
        if let Some(selector) = selector {
            builder = builder.selector(selector.into_mlir());
        }
        if packed {
            builder = builder.packed(Attribute::unit(context.melior));
        }
        Self::new(
            block
                .append_operation(builder.build().into())
                .result(0)
                .expect("sol.encode always produces one result")
                .into(),
        )
    }

    /// Emits `sol.decode` recovering a single value of `result_type` from the `payload` bytes.
    pub fn decode<B>(
        payload: Self,
        result_type: Type<'context>,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(mlir_op!(
            context,
            block,
            DecodeOperation
                .addr(payload)
                .outs(&[result_type.into_mlir()])
        ))
    }
}
