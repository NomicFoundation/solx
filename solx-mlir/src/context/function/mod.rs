//!
//! Function call resolution metadata.
//!

pub mod dispatch;

use melior::ir::Block as MlirBlock;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::operation::OperationLike;
use melior::ir::r#type::IntegerType;

use crate::Block;
use crate::Context;
use crate::FunctionDispatch;
use crate::FunctionKind;
use crate::FunctionType;
use crate::StateMutability;
use crate::Type;
use crate::Value;
use crate::ods::sol::FuncOperation;

/// A function a call site can name: its mangled symbol and MLIR-interned parameter and return
/// types, so the site emits its call without re-resolving the signature. Functions this unit
/// lowers are registered once; a callee in another contract is named where it is called.
#[derive(Clone)]
pub struct Function<'context> {
    /// The mangled MLIR function name.
    pub mlir_name: String,
    /// Parameter and result types, MLIR-interned, exact from the function signature.
    pub function_type: FunctionType<'context>,
}

impl<'context> Function<'context> {
    /// The `mlir_name` of a synthesized parameterless constructor.
    pub const CONSTRUCTOR_NAME: &'static str = "@constructor()";

    /// Records a function's mangled name and interned signature.
    pub fn new(mlir_name: String, function_type: FunctionType<'context>) -> Self {
        Self {
            mlir_name,
            function_type,
        }
    }

    /// The signature of a synthesized parameterless constructor.
    pub fn constructor() -> Self {
        Self::new(Self::CONSTRUCTOR_NAME.to_owned(), FunctionType::default())
    }

    /// Emits this function's `sol.func` definition with an entry block whose arguments carry the
    /// parameter types, returned for the body. An original function type is attached for
    /// selector-dispatched and constructor functions.
    pub fn define(
        &self,
        selector: Option<u32>,
        dispatch: FunctionDispatch,
        state_mutability: StateMutability,
        context: &Context<'context>,
        contract_body: Block<'context>,
    ) -> Block<'context> {
        let parameters = self
            .function_type
            .parameters
            .iter()
            .map(|parameter| parameter.into_mlir())
            .collect::<Vec<_>>();
        let function_type = self.function_type.to_mlir(context.melior);
        let body_region = Region::new();
        let entry_block = MlirBlock::new(
            &parameters
                .iter()
                .map(|parameter| (*parameter, context.location()))
                .collect::<Vec<_>>(),
        );
        body_region.append_block(entry_block);

        let mut operation_builder = FuncOperation::builder(context.melior, context.location())
            .sym_name(StringAttribute::new(context.melior, &self.mlir_name))
            .function_type(TypeAttribute::new(function_type.into()))
            .state_mutability(state_mutability.attribute(context.melior))
            .body(body_region);
        operation_builder = match dispatch {
            FunctionDispatch::Identifier(identifier) => {
                operation_builder.id(IntegerAttribute::new(
                    IntegerType::new(context.melior, solx_utils::BIT_LENGTH_X64 as u32).into(),
                    usize::from(identifier) as i64,
                ))
            }
            FunctionDispatch::Kind(function_kind) => {
                operation_builder.kind(function_kind.attribute(context.melior))
            }
            FunctionDispatch::Getter => operation_builder,
        };
        if let Some(selector_value) = selector {
            operation_builder = operation_builder
                .selector(Type::selector_attribute(selector_value, context.melior));
        }
        if selector.is_some()
            || matches!(dispatch, FunctionDispatch::Kind(FunctionKind::Constructor))
        {
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

    /// Emits the internal function pointer to this function (`sol.func_constant`).
    pub fn pointer_constant(&self, context: &Context<'context>) -> Value<'context> {
        Value::function_constant(
            &self.mlir_name,
            self.function_type
                .reference(context.melior, solx_utils::FunctionReferenceKind::Internal),
            context,
        )
    }
}
