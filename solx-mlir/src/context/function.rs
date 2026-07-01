//!
//! Function call resolution metadata, and the `sol.func` / `sol.call` it emits.
//!

use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::Type as MlirType;
use melior::ir::Value as MlirValue;
use melior::ir::attribute::FlatSymbolRefAttribute;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::operation::OperationLike;
use melior::ir::r#type::FunctionType;
use melior::ir::r#type::IntegerType;

use crate::Context;
use crate::FunctionKind;
use crate::StateMutability;
use crate::Type;
use crate::ods::sol::CallOperation;
use crate::ods::sol::FuncOperation;

/// Cached signature of a lowered function: its mangled symbol and MLIR-interned parameter and
/// return types, so a call site emits `sol.call` without re-resolving the signature.
#[derive(Clone)]
pub struct Function<'context> {
    /// The mangled MLIR function name.
    pub mlir_name: String,
    /// Parameter types, MLIR-interned, exact from the function signature.
    pub parameter_types: Vec<MlirType<'context>>,
    /// Return types, MLIR-interned, exact from the function signature.
    pub return_types: Vec<MlirType<'context>>,
}

impl<'context> Function<'context> {
    /// Records a function's mangled name and interned signature.
    pub fn new(
        mlir_name: String,
        parameter_types: Vec<MlirType<'context>>,
        return_types: Vec<MlirType<'context>>,
    ) -> Self {
        Self {
            mlir_name,
            parameter_types,
            return_types,
        }
    }

    /// Emits this function's `sol.func` definition with an entry block whose arguments carry the
    /// parameter types, returned for the body. `selector` / `kind` are the optional dispatch
    /// attributes; `dispatch_identifier` is the internal-function-pointer dispatch tag; an original
    /// function type is attached for selector-dispatched, constructor, and fallback functions.
    pub fn define<'block>(
        &self,
        selector: Option<u32>,
        state_mutability: StateMutability,
        kind: Option<FunctionKind>,
        dispatch_identifier: Option<i64>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let function_type = FunctionType::new(
            context.mlir_context,
            &self.parameter_types,
            &self.return_types,
        );
        let body_region = Region::new();
        let entry_block = Block::new(
            &self
                .parameter_types
                .iter()
                .map(|parameter_type| (*parameter_type, context.location()))
                .collect::<Vec<_>>(),
        );
        body_region.append_block(entry_block);

        let mut operation_builder =
            FuncOperation::builder(context.mlir_context, context.location())
                .sym_name(StringAttribute::new(context.mlir_context, &self.mlir_name))
                .function_type(TypeAttribute::new(function_type.into()))
                .state_mutability(state_mutability.attribute(context.mlir_context))
                .body(body_region);
        if let Some(function_kind) = kind {
            operation_builder =
                operation_builder.kind(function_kind.attribute(context.mlir_context));
        }
        if let Some(selector_value) = selector {
            operation_builder = operation_builder.selector(IntegerAttribute::new(
                IntegerType::new(context.mlir_context, Type::SELECTOR_BIT_WIDTH).into(),
                selector_value as i64,
            ));
        }
        if let Some(function_id) = dispatch_identifier {
            operation_builder = operation_builder.id(IntegerAttribute::new(
                IntegerType::new(context.mlir_context, solx_utils::BIT_LENGTH_X64 as u32).into(),
                function_id,
            ));
        }
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

    /// Emits a `sol.call` to `callee` by symbol, returning its results in declaration order.
    pub fn call<'block, B>(
        callee: &str,
        operands: &[MlirValue<'context, 'block>],
        result_types: &[MlirType<'context>],
        context: &Context<'context>,
        block: &B,
    ) -> anyhow::Result<Vec<MlirValue<'context, 'block>>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let operation = block.append_operation(mlir_op_build!(
            context,
            CallOperation
                .callee(FlatSymbolRefAttribute::new(context.mlir_context, callee))
                .outs(result_types)
                .operands(operands)
        ));
        let mut results = Vec::with_capacity(result_types.len());
        for index in 0..result_types.len() {
            results.push(operation.result(index)?.into());
        }
        Ok(results)
    }

    /// The `!sol.func_ref<...>` type of an internal pointer to this function, built from its
    /// declared signature.
    pub fn func_ref_type(&self, context: &Context<'context>) -> Type<'context> {
        Type::func_ref(
            context.mlir_context,
            &self.parameter_types,
            &self.return_types,
        )
    }

    /// `sol.func_constant`: the internal function pointer to this function.
    pub fn pointer_constant<'block>(
        &self,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> crate::Value<'context, 'block> {
        crate::Value::function_constant(&self.mlir_name, self.func_ref_type(context), context, block)
    }
}
