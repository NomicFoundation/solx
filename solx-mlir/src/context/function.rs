//!
//! Function call resolution metadata.
//!

use melior::ir::BlockLike;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::FlatSymbolRefAttribute;

use crate::Builder;
use crate::ods::sol::CallOperation;

/// Function call resolution metadata for the MLIR builder.
#[derive(Clone)]
pub struct Function<'context> {
    /// The mangled MLIR function name.
    pub mlir_name: String,
    /// Parameter types (MLIR-interned, exact types from the function signature).
    pub parameter_types: Vec<Type<'context>>,
    /// Return types (MLIR-interned, exact types from the function signature).
    pub return_types: Vec<Type<'context>>,
}

impl<'context> Function<'context> {
    /// Creates a new function metadata entry.
    pub fn new(
        mlir_name: String,
        parameter_types: Vec<Type<'context>>,
        return_types: Vec<Type<'context>>,
    ) -> Self {
        Self {
            mlir_name,
            parameter_types,
            return_types,
        }
    }

    /// Emits a `sol.call` to this function — an internal call by symbol — and
    /// returns its result values in declaration order. Calling the function is
    /// the resolution metadata's own behavior, so the op homes here rather than
    /// on a builder method.
    pub fn call<'block, B>(
        &self,
        operands: &[Value<'context, 'block>],
        builder: &Builder<'context>,
        block: &B,
    ) -> Vec<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let operation = block.append_operation(sol_op_build!(
            builder,
            CallOperation
                .callee(FlatSymbolRefAttribute::new(
                    builder.context,
                    &self.mlir_name
                ))
                .outs(&self.return_types)
                .operands(operands)
        ));
        (0..self.return_types.len())
            .map(|index| {
                operation
                    .result(index)
                    .expect("sol.call produces its declared result count")
                    .into()
            })
            .collect()
    }
}
