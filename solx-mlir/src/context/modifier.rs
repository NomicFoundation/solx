//!
//! Modifier definition emission.
//!

use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::Type as MlirType;
use melior::ir::Value as MlirValue;
use melior::ir::attribute::FlatSymbolRefAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::operation::OperationLike;
use melior::ir::r#type::FunctionType;

use crate::Context;
use crate::ods::sol::CallOperation;
use crate::ods::sol::ModifierOperation;

/// A `sol.modifier` definition: a `FunctionOpInterface` op like `sol.func`, but with no results.
///
/// Its body is the modifier's statements with each `_;` lowered to `sol.placeholder`, terminated by
/// `sol.return`. The downstream `sol.modifier_call_blk` references it by its `sym_name` symbol.
pub struct Modifier<'context> {
    /// The mangled MLIR symbol name, shared with the invoking `sol.call`.
    pub mlir_name: String,
    /// Parameter types, MLIR-interned, exact from the modifier signature.
    pub parameter_types: Vec<MlirType<'context>>,
}

impl<'context> Modifier<'context> {
    /// Records a modifier's mangled symbol and interned parameter types; `define` later materializes the op.
    pub fn new(mlir_name: String, parameter_types: Vec<MlirType<'context>>) -> Self {
        Self {
            mlir_name,
            parameter_types,
        }
    }

    /// Emits this `sol.modifier` definition with an empty entry block whose arguments are its
    /// parameters, returned for the body. A modifier has no results, so its `FunctionType` is
    /// `() -> ()` over its parameters.
    pub fn define<'block>(
        &self,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let function_type = FunctionType::new(context.mlir_context, &self.parameter_types, &[]);
        let body_region = Region::new();
        let entry_block = Block::new(
            &self
                .parameter_types
                .iter()
                .map(|parameter_type| (*parameter_type, context.location()))
                .collect::<Vec<_>>(),
        );
        body_region.append_block(entry_block);

        let operation = ModifierOperation::builder(context.mlir_context, context.location())
            .sym_name(StringAttribute::new(context.mlir_context, &self.mlir_name))
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

    /// Emits a `sol.call` to this modifier. A modifier has no results, so the call yields none.
    pub fn call<'block, B>(
        &self,
        operands: &[MlirValue<'context, 'block>],
        context: &Context<'context>,
        block: &B,
    ) where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(mlir_op_build!(
            context,
            CallOperation
                .callee(FlatSymbolRefAttribute::new(
                    context.mlir_context,
                    &self.mlir_name
                ))
                .outs(&[])
                .operands(operands)
        ));
    }
}
