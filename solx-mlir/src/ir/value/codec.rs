//!
//! ABI codec value producers: `abi.encode*` and `abi.decode`.
//!

use melior::ir::Attribute;

use crate::Context;
use crate::Type;
use crate::Value;
use crate::ods::sol::DecodeOperation;
use crate::ods::sol::EncodeOperation;

impl<'context> Value<'context> {
    /// Emits `sol.encode` producing a `bytes memory` payload from `inputs`.
    ///
    /// A `selector`, when present, is prepended as the first four bytes and must already be
    /// `!sol.fixed_bytes<4>`. `packed` selects the ABI-packed encoding (no per-element padding).
    pub fn encode(
        inputs: &[Self],
        selector: Option<Self>,
        packed: bool,
        context: &Context<'context>,
    ) -> Self {
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
        Self::from(
            context
                .current_block()
                .append_operation(builder.build().into())
                .result(0)
                .expect("sol.encode always produces one result"),
        )
    }

    /// Emits `sol.decode` recovering a single value of `result_type` from the `payload` bytes.
    pub fn decode(payload: Self, result_type: Type<'context>, context: &Context<'context>) -> Self {
        Self::from(mlir_op!(
            context,
            DecodeOperation
                .addr(payload)
                .outs(&[result_type.into_mlir()])
        ))
    }
}
