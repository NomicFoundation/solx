//!
//! Function call resolution metadata.
//!

use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::FlatSymbolRefAttribute;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::operation::OperationLike;
use melior::ir::r#type::FunctionType;
use melior::ir::r#type::IntegerType;

use crate::Builder;
use crate::FunctionKind;
use crate::StateMutability;
use crate::ods::sol::CallOperation;
use crate::ods::sol::FuncOperation;

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
        let operation = block.append_operation(mlir_op_build!(
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

    /// The `!sol.func_ref<…>` type of an internal pointer to this function,
    /// built from its declared signature.
    pub fn func_ref_type(&self, builder: &Builder<'context>) -> crate::Type<'context> {
        crate::Type::func_ref(builder.context, &self.parameter_types, &self.return_types)
    }

    /// `sol.func_constant` — the internal function pointer to this function, the
    /// value a bare function reference lowers to. Emitting the pointer is the
    /// function metadata's own behavior, beside `call` and `define`.
    pub fn pointer_constant<'block>(
        &self,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> crate::Value<'context, 'block> {
        crate::Value::function_constant(
            &self.mlir_name,
            self.func_ref_type(builder),
            builder,
            block,
        )
    }

    /// Emits this function's `sol.func` definition with an empty entry block and
    /// returns that block for appending the body. `selector` / `kind` / `id` are
    /// the optional dispatch attributes; a selector-bearing function, constructor,
    /// or fallback also carries `orig_fn_type` (the SolToYul fallback dispatcher
    /// reads it to recover the pre-conversion signature). Emitting a function's
    /// definition is its own behavior, so the op homes here alongside calling it.
    pub fn define<'block>(
        &self,
        selector: Option<u32>,
        state_mutability: StateMutability,
        kind: Option<FunctionKind>,
        id: Option<i64>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let function_type =
            FunctionType::new(builder.context, &self.parameter_types, &self.return_types);
        let body_region = Region::new();
        let entry_block = Block::new(
            &self
                .parameter_types
                .iter()
                .map(|parameter_type| (*parameter_type, builder.unknown_location))
                .collect::<Vec<_>>(),
        );
        body_region.append_block(entry_block);

        let mut operation_builder =
            FuncOperation::builder(builder.context, builder.unknown_location)
                .sym_name(StringAttribute::new(builder.context, &self.mlir_name))
                .function_type(TypeAttribute::new(function_type.into()))
                .state_mutability(state_mutability.attribute(builder.context))
                .body(body_region);
        if let Some(function_kind) = kind {
            operation_builder = operation_builder.kind(function_kind.attribute(builder.context));
        }
        if let Some(selector_value) = selector {
            operation_builder = operation_builder.selector(IntegerAttribute::new(
                IntegerType::new(builder.context, crate::Type::SELECTOR_BIT_WIDTH).into(),
                selector_value as i64,
            ));
        }
        // A referenceable function carries a unique `id` (its slang node id): the
        // `sol.func_constant` pointer lowers to that i256, and the `sol.icall`
        // dispatch switches over every same-signature function's `id`.
        if let Some(function_id) = id {
            operation_builder = operation_builder.id(IntegerAttribute::new(
                IntegerType::new(builder.context, 64).into(),
                function_id,
            ));
        }
        // A selector-bearing function, constructor, or fallback carries
        // `orig_fn_type`: the SolToYul fallback dispatcher reads it to recover the
        // pre-conversion Sol signature, else it dereferences a null type.
        if selector.is_some()
            || matches!(
                kind,
                Some(FunctionKind::Constructor | FunctionKind::Fallback)
            )
        {
            operation_builder =
                operation_builder.orig_fn_type(TypeAttribute::new(function_type.into()));
        }
        let operation = block.append_operation(operation_builder.build().into());
        operation
            .region(0)
            .expect("func has one region")
            .first_block()
            .expect("func body has entry block")
    }
}
