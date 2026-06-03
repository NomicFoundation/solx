//!
//! `new` expression lowering: contract creation (`new C(args)` → `sol.new`)
//! and dynamic memory allocation (`new T[](n)` / `new bytes(n)` /
//! `new string(n)` → `sol.malloc`).
//!

use super::*;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Emits a `new Contract(args)` expression as a `sol.new` operation.
    ///
    /// The contract type comes from the binder; payability is derived the same
    /// way it is when resolving a `SlangType::Contract` reference. Value
    /// transfer (`new C{value: x}()`) and `CREATE2` salt (`new C{salt: s}()`)
    /// are not yet handled — those go through `CallOptionsExpression`.
    pub fn emit_new(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let slang_type = call.get_type();
        // `new T[](n)` / `new bytes(n)` / `new string(n)` allocate a dynamic
        // memory aggregate of `n` elements/bytes via `sol.malloc`, passing the
        // count as the `size` operand so the length slot is initialised. slang
        // resolves the array forms' call type, but `new bytes`/`new string`
        // surface no call type, so fall back to the syntactic type name (both
        // lower to a memory string).
        let dynamic_result_type = match &slang_type {
            Some(inner @ (SlangType::Array(_) | SlangType::Bytes(_) | SlangType::String(_))) => {
                Some(TypeConversion::resolve_slang_type(
                    inner,
                    Some(solx_utils::DataLocation::Memory),
                    &self.expression_emitter.state.builder,
                ))
            }
            None
                if matches!(
                    call.operand(),
                    Expression::NewExpression(new_expression)
                        if matches!(new_expression.type_name(), SlangTypeName::ElementaryType(_))
                ) =>
            {
                Some(
                    self.expression_emitter
                        .state
                        .builder
                        .types
                        .string(solx_utils::DataLocation::Memory),
                )
            }
            _ => None,
        };
        if let Some(result_type) = dynamic_result_type {
            let (values, block) = self.emit_argument_values(arguments, block)?;
            let builder = &self.expression_emitter.state.builder;
            let address = match values.first() {
                Some(&size_value) => {
                    let size = TypeConversion::from_target_type(builder.types.ui256, builder)
                        .emit(size_value, builder, &block);
                    // `new T[](n)` / `new bytes(n)` are zeroed per Solidity.
                    builder.emit_sol_malloc_sized_zeroed(result_type, size, &block)
                }
                None => builder.emit_sol_malloc_zeroed(result_type, &block),
            };
            return Ok((Some(address), block));
        }
        let Some(SlangType::Contract(contract_type)) = slang_type else {
            unimplemented!("new expression has no resolved type or unsupported new target");
        };
        let Definition::Contract(contract_definition) = contract_type.definition() else {
            unreachable!("Slang ContractType always references a Contract definition");
        };
        let contract_name = contract_definition.name().name();
        let payable = ContractEmitter::is_contract_payable(&contract_definition);

        // Tell the linker that this contract embeds `contract_name`'s deploy
        // bytecode so the assembler pulls it in.
        self.expression_emitter
            .state
            .add_dependency(contract_name.clone());

        let builder = &self.expression_emitter.state.builder;
        let result_type = builder.types.contract(&contract_name, payable);

        let (ctor_args, block) = self.emit_argument_values(arguments, block)?;
        let val = builder.emit_sol_constant(0, builder.types.ui256, &block);

        // `operand_segment_sizes` (TableGen order: val=1, salt=0, ctorArgs=N) is
        // synthesized by the melior op-builder macro for this
        // `AttrSizedOperandSegments` op — `.val()` and `.ctor_args()` are set while
        // the optional `salt` is left unset, yielding [1, 0, ctor_args.len()].
        let operation: Operation =
            NewOperation::builder(builder.context, builder.unknown_location)
                .obj_name(StringAttribute::new(builder.context, &contract_name))
                .val(val)
                .ctor_args(&ctor_args)
                .out(result_type)
                .build()
                .into();

        let value = block
            .append_operation(operation)
            .result(0)
            .expect("sol.new always produces one result")
            .into();
        Ok((Some(value), block))
    }
}
