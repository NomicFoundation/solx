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

use crate::Context;
use crate::FunctionKind;
use crate::StateMutability;
use crate::ods::sol::CallOperation;
use crate::ods::sol::FuncOperation;
use crate::ods::sol::ModifierOperation;

/// Function call resolution metadata for the MLIR context.
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

    /// Emits a `sol.call` to this function (an internal call by symbol), returning its results in order.
    pub fn call<'block, B>(
        &self,
        operands: &[Value<'context, 'block>],
        context: &Context<'context>,
        block: &B,
    ) -> Vec<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let operation = block.append_operation(mlir_op_build!(
            context,
            CallOperation
                .callee(FlatSymbolRefAttribute::new(context.mlir(), &self.mlir_name))
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
    pub fn func_ref_type(&self, context: &Context<'context>) -> crate::Type<'context> {
        crate::Type::func_ref(context.mlir(), &self.parameter_types, &self.return_types)
    }

    /// `sol.func_constant` — the internal function pointer to this function.
    pub fn pointer_constant<'block>(
        &self,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> crate::Value<'context, 'block> {
        crate::Value::function_constant(
            &self.mlir_name,
            self.func_ref_type(context),
            context,
            block,
        )
    }

    /// Emits this function's `sol.func` definition with an empty entry block, returned for the body.
    /// `selector` / `kind` / `id` are the optional dispatch attributes.
    pub fn define<'block>(
        &self,
        selector: Option<u32>,
        state_mutability: StateMutability,
        kind: Option<FunctionKind>,
        id: Option<i64>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let function_type =
            FunctionType::new(context.mlir(), &self.parameter_types, &self.return_types);
        let body_region = Region::new();
        let entry_block = Block::new(
            &self
                .parameter_types
                .iter()
                .map(|parameter_type| (*parameter_type, context.location()))
                .collect::<Vec<_>>(),
        );
        body_region.append_block(entry_block);

        let mut operation_builder = FuncOperation::builder(context.mlir(), context.location())
            .sym_name(StringAttribute::new(context.mlir(), &self.mlir_name))
            .function_type(TypeAttribute::new(function_type.into()))
            .state_mutability(state_mutability.attribute(context.mlir()))
            .body(body_region);
        if let Some(function_kind) = kind {
            operation_builder = operation_builder.kind(function_kind.attribute(context.mlir()));
        }
        if let Some(selector_value) = selector {
            operation_builder = operation_builder.selector(IntegerAttribute::new(
                IntegerType::new(context.mlir(), crate::Type::SELECTOR_BIT_WIDTH).into(),
                selector_value as i64,
            ));
        }
        // A referenceable function carries a unique `id`: `sol.icall` dispatch switches over it.
        if let Some(function_id) = id {
            operation_builder = operation_builder.id(IntegerAttribute::new(
                IntegerType::new(context.mlir(), 64).into(),
                function_id,
            ));
        }
        // A selector-bearing function or constructor carries `orig_fn_type`: the
        // SolToYul interface/constructor dispatchers read it to recover the
        // pre-conversion Sol signature. Fallbacks no longer need it — their
        // dispatcher derives everything from the live `function_type` arity.
        if selector.is_some() || matches!(kind, Some(FunctionKind::Constructor)) {
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

/// A `sol.modifier` definition: a `FunctionOpInterface` op like `sol.func`, but with no results.
///
/// Its body is the modifier's statements with each `_;` lowered to `sol.placeholder`, terminated by
/// `sol.return`. The downstream `sol.modifier_call_blk` references it by its `sym_name` symbol.
pub struct Modifier<'context> {
    /// The mangled MLIR symbol name (shared with the invoking `sol.call`).
    pub mlir_name: String,
    /// Parameter types (MLIR-interned, exact types from the modifier signature).
    pub parameter_types: Vec<Type<'context>>,
}

impl<'context> Modifier<'context> {
    /// Creates a new modifier definition descriptor.
    pub fn new(mlir_name: String, parameter_types: Vec<Type<'context>>) -> Self {
        Self {
            mlir_name,
            parameter_types,
        }
    }

    /// Emits this `sol.modifier` definition with an empty entry block (its parameters as block
    /// arguments), returned for the body. A modifier has no results, so its `FunctionType` is `() -> ()`
    /// over its parameters.
    pub fn define<'block>(
        &self,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let function_type = FunctionType::new(context.mlir(), &self.parameter_types, &[]);
        let body_region = Region::new();
        let entry_block = Block::new(
            &self
                .parameter_types
                .iter()
                .map(|parameter_type| (*parameter_type, context.location()))
                .collect::<Vec<_>>(),
        );
        body_region.append_block(entry_block);

        let operation = ModifierOperation::builder(context.mlir(), context.location())
            .sym_name(StringAttribute::new(context.mlir(), &self.mlir_name))
            .function_type(TypeAttribute::new(function_type.into()))
            .body(body_region)
            .build();
        let operation = block.append_operation(operation.into());
        operation
            .region(0)
            .expect("modifier has one region")
            .first_block()
            .expect("modifier body has entry block")
    }
}
