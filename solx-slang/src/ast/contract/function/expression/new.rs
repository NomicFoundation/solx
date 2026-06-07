//!
//! `new` expression lowering: dynamic-aggregate allocation (`new T[](n)`,
//! `new bytes(n)`, `new string(n)`) and contract creation (`new C(args)`).
//!
//! An [`ExpressionEmitter`] method: `new.rs` lives in the expression module
//! subtree, so it lowers through the expression emitter directly rather than
//! the call emitter (the oracle's `built_in/new.rs` placement).
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Operation;
use melior::ir::Value;
use melior::ir::attribute::DenseI32ArrayAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::operation::OperationMutLike;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::Type as SlangType;
use slang_solidity_v2::ast::TypeName as SlangTypeName;

use solx_mlir::ods::sol::NewOperation;
use solx_utils::DataLocation;

use crate::ast::contract::ContractEmitter;
use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits a `new` expression: dynamic-aggregate allocation (`new T[](n)`,
    /// `new bytes(n)`) or contract creation (`new C(args)`).
    pub fn emit_new(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let slang_type = call.get_type();
        // `new T[](n)` / `new bytes(n)` / `new string(n)` allocate a dynamic
        // memory aggregate of `n` elements/bytes via a zeroed `sol.malloc`, the
        // count driving the length slot. slang resolves the array forms' call
        // type, but `new bytes` / `new string` surface no call type, so fall back
        // to the syntactic elementary type name (both lower to a memory string).
        let dynamic_result_type = match &slang_type {
            Some(inner @ (SlangType::Array(_) | SlangType::Bytes(_) | SlangType::String(_))) => {
                Some(TypeConversion::resolve_slang_type(
                    inner,
                    Some(DataLocation::Memory),
                    &self.state.builder,
                ))
            }
            None if matches!(
                call.operand(),
                Expression::NewExpression(new_expression)
                    if matches!(new_expression.type_name(), SlangTypeName::ElementaryType(_))
            ) =>
            {
                Some(self.state.builder.types.string(DataLocation::Memory))
            }
            _ => None,
        };
        if let Some(result_type) = dynamic_result_type {
            let (values, current_block) = self.emit_argument_values(arguments, block)?;
            let builder = &self.state.builder;
            let address =
                match values.first() {
                    Some(&size_value) => {
                        let size = TypeConversion::from_target_type(builder.types.ui256, builder)
                            .emit(size_value, builder, &current_block);
                        builder.emit_sol_malloc_sized_zeroed(result_type, size, &current_block)
                    }
                    None => builder.emit_sol_malloc_zeroed(result_type, &current_block),
                };
            return Ok((Some(address), current_block));
        }

        // Contract creation: `new C(args)` lowers to `sol.new`, which embeds
        // `C`'s deploy bytecode. Record the dependency so the linker pulls the
        // object in. Value transfer (`new C{value: x}()`) and CREATE2 salt go
        // through call options and are not handled here.
        let Some(SlangType::Contract(contract_type)) = slang_type else {
            unimplemented!("new expression has no resolved type or unsupported new target");
        };
        let Definition::Contract(contract_definition) = contract_type.definition() else {
            unreachable!("Slang ContractType always references a Contract definition");
        };
        let contract_name = contract_definition.name().name();
        let payable = ContractEmitter::is_contract_payable(&contract_definition);
        self.state.add_dependency(contract_name.clone());

        let (ctor_args, block) = self.emit_argument_values(arguments, block)?;
        let builder = &self.state.builder;
        let result_type = builder.types.contract(&contract_name, payable);
        let val = builder.emit_sol_constant(0, builder.types.ui256, &block);

        let mut operation: Operation =
            NewOperation::builder(builder.context, builder.unknown_location)
                .obj_name(StringAttribute::new(builder.context, &contract_name))
                .val(val)
                .ctor_args(&ctor_args)
                .out(result_type)
                .build()
                .into();
        // Set `operand_segment_sizes` manually (val=1, salt=0, ctorArgs=N): the
        // optional `salt` and `try_call` are left unset, and melior's ODS builder
        // does not synthesize the attribute for this `AttrSizedOperandSegments`
        // op, so the dialect verifier rejects the op without it.
        let ctor_args_count =
            i32::try_from(ctor_args.len()).expect("constructor argument count fits in i32");
        let segment_sizes = DenseI32ArrayAttribute::new(builder.context, &[1, 0, ctor_args_count]);
        operation.set_inherent_attribute("operand_segment_sizes", segment_sizes.into());
        let value = block
            .append_operation(operation)
            .result(0)
            .expect("sol.new always produces one result")
            .into();
        Ok((Some(value), block))
    }

    /// Evaluates a positional argument list left-to-right, threading the block
    /// through each sub-expression, and returns the values with the final block.
    fn emit_argument_values(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let mut values = Vec::with_capacity(arguments.len());
        let mut current_block = block;
        for argument in arguments.iter() {
            let (value, next) = self.emit_value(&argument, current_block)?;
            values.push(value);
            current_block = next;
        }
        Ok((values, current_block))
    }
}
