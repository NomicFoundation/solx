//!
//! Function call resolution metadata, and the `sol.func` / `sol.call` it emits.
//!

use melior::ir::Block as MlirBlock;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::attribute::FlatSymbolRefAttribute;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::operation::OperationLike;
use melior::ir::r#type::FunctionType;
use melior::ir::r#type::IntegerType;

use crate::Block;
use crate::Context;
use crate::FunctionKind;
use crate::StateMutability;
use crate::Type;
use crate::Value;
use crate::ods::sol::CallOperation;
use crate::ods::sol::FuncOperation;

/// Cached signature of a lowered function: its mangled symbol and MLIR-interned parameter and
/// return types, so a call site emits `sol.call` without re-resolving the signature.
#[derive(Clone)]
pub struct Function<'context> {
    /// The mangled MLIR function name.
    pub mlir_name: String,
    /// Parameter types, MLIR-interned, exact from the function signature.
    pub parameter_types: Vec<Type<'context>>,
    /// Return types, MLIR-interned, exact from the function signature.
    pub return_types: Vec<Type<'context>>,
}

impl<'context> Function<'context> {
    /// The `mlir_name` of a synthesized parameterless constructor.
    pub const CONSTRUCTOR_NAME: &'static str = "@constructor()";

    /// Records a function's mangled name and interned signature.
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

    /// The signature of a synthesized parameterless constructor.
    pub fn constructor() -> Self {
        Self::new(Self::CONSTRUCTOR_NAME.to_owned(), Vec::new(), Vec::new())
    }

    /// Emits this function's `sol.func` definition with an entry block whose arguments carry the
    /// parameter types, returned for the body. `selector` / `kind` are the optional dispatch
    /// attributes; an original function type is attached for selector-dispatched and constructor
    /// functions.
    pub fn define(
        &self,
        selector: Option<u32>,
        state_mutability: StateMutability,
        kind: Option<FunctionKind>,
        context: &Context<'context>,
        contract_body: Block<'context>,
    ) -> Block<'context> {
        let parameter_types = self
            .parameter_types
            .iter()
            .map(|parameter_type| parameter_type.into_mlir())
            .collect::<Vec<_>>();
        let return_types = self
            .return_types
            .iter()
            .map(|return_type| return_type.into_mlir())
            .collect::<Vec<_>>();
        let function_type = FunctionType::new(context.melior, &parameter_types, &return_types);
        let body_region = Region::new();
        let entry_block = MlirBlock::new(
            &parameter_types
                .iter()
                .map(|parameter_type| (*parameter_type, context.location()))
                .collect::<Vec<_>>(),
        );
        body_region.append_block(entry_block);

        let mut operation_builder = FuncOperation::builder(context.melior, context.location())
            .sym_name(StringAttribute::new(context.melior, &self.mlir_name))
            .function_type(TypeAttribute::new(function_type.into()))
            .state_mutability(state_mutability.attribute(context.melior))
            .body(body_region);
        if let Some(function_kind) = kind {
            operation_builder = operation_builder.kind(function_kind.attribute(context.melior));
        }
        if let Some(selector_value) = selector {
            operation_builder = operation_builder.selector(IntegerAttribute::new(
                IntegerType::new(context.melior, Type::SELECTOR_BIT_WIDTH).into(),
                selector_value as i64,
            ));
        }
        if selector.is_some() || matches!(kind, Some(FunctionKind::Constructor)) {
            operation_builder =
                operation_builder.orig_fn_type(TypeAttribute::new(function_type.into()));
        }
        let operation = contract_body.append_operation(operation_builder.build().into());
        Block::from(
            operation
                .region(0)
                .expect("func has one region")
                .first_block()
                .expect("func body has entry block"),
        )
    }

    /// Emits a `sol.call` to `callee` by symbol, returning its results in declaration order.
    pub fn call(
        callee: &str,
        operands: &[Value<'context>],
        result_types: &[Type<'context>],
        context: &Context<'context>,
    ) -> anyhow::Result<Vec<Value<'context>>> {
        let operands = operands
            .iter()
            .map(|operand| operand.into_mlir())
            .collect::<Vec<_>>();
        let result_types = result_types
            .iter()
            .map(|result_type| result_type.into_mlir())
            .collect::<Vec<_>>();
        let operation = context.current_block().append_operation(mlir_op_build!(
            context,
            CallOperation
                .callee(FlatSymbolRefAttribute::new(context.melior, callee))
                .outs(result_types.as_slice())
                .operands(operands.as_slice())
        ));
        let mut results = Vec::with_capacity(result_types.len());
        for index in 0..result_types.len() {
            results.push(Value::from(operation.result(index)?));
        }
        Ok(results)
    }
}
