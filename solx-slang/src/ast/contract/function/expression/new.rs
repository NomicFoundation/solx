//!
//! `new` expression emission: dynamic-aggregate allocation (`new T[](n)`,
//! `new bytes(n)`, `new string(n)`) and contract creation (`new C(args)`).
//!
//! The `New` arm of [`CallKind`] — `new C()` is a `FunctionCallExpression`.
//!

use melior::ir::Attribute;
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

use solx_mlir::ods::sol::MallocOperation;
use solx_mlir::ods::sol::NewOperation;
use solx_utils::DataLocation;

use crate::ast::ContractPayable;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_kind::CallKind;
use crate::ast::type_conversion::LocationPolicy;
use crate::ast::type_conversion::ResolveSignature;
use crate::ast::type_conversion::ResolveType;

impl CallKind {
    /// Emits a `new` expression: dynamic-aggregate allocation (`new T[](n)`,
    /// `new bytes(n)`) or contract creation (`new C(args)`).
    pub fn emit_new<'state, 'context, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        value: Option<Value<'context, 'block>>,
        salt: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> (Option<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        let slang_type = call.get_type();
        // `new T[](n)` / `new bytes(n)` / `new string(n)` allocate a dynamic
        // memory aggregate of `n` elements/bytes via a zeroed `sol.malloc`, the
        // count driving the length slot. slang resolves the array forms' call
        // type, but `new bytes` / `new string` surface no call type, so fall back
        // to the syntactic elementary type name (both lower to a memory string).
        let dynamic_result_type = match &slang_type {
            Some(inner @ (SlangType::Array(_) | SlangType::Bytes(_) | SlangType::String(_))) => {
                Some(inner.resolve_type(
                    LocationPolicy::Declared(Some(DataLocation::Memory)),
                    &context.state.builder,
                ))
            }
            None if matches!(
                call.operand(),
                Expression::NewExpression(new_expression)
                    if matches!(new_expression.type_name(), SlangTypeName::ElementaryType(_))
            ) =>
            {
                Some(
                    crate::ast::Type::string(context.state.builder.context, DataLocation::Memory)
                        .into_mlir(),
                )
            }
            _ => None,
        };
        if let Some(result_type) = dynamic_result_type {
            let (values, current_block) = context.emit_argument_values(arguments, block);
            let builder = &context.state.builder;
            let address = match values.first() {
                Some(&size_value) => {
                    let size = crate::ast::Value::from(size_value)
                        .coerce_to(
                            crate::ast::Type::unsigned(
                                builder.context,
                                solx_utils::BIT_LENGTH_FIELD,
                            ),
                            builder,
                            &current_block,
                        )
                        .into_mlir();
                    sol_op!(
                        builder,
                        &current_block,
                        MallocOperation
                            .addr(result_type)
                            .size(size)
                            .zero_init(Attribute::unit(builder.context))
                    )
                }
                None => sol_op!(
                    builder,
                    &current_block,
                    MallocOperation
                        .addr(result_type)
                        .zero_init(Attribute::unit(builder.context))
                ),
            };
            return (Some(address), current_block);
        }

        // Contract creation: `new C(args)` lowers to `sol.new`, which embeds
        // `C`'s deploy bytecode. Record the dependency so the linker pulls the
        // object in. A `new C{value: v}()` forwards `v` wei; a `new C{salt: s}()`
        // selects CREATE2 with the (already `ui256`-cast) salt operand.
        let Some(SlangType::Contract(contract_type)) = slang_type else {
            unimplemented!("new expression has no resolved type or unsupported new target");
        };
        let Definition::Contract(contract_definition) = contract_type.definition() else {
            unreachable!("Slang ContractType always references a Contract definition");
        };
        let contract_name = contract_definition.name().name();
        let payable = contract_definition.is_payable();
        context.state.add_dependency(contract_name.clone());

        // Coerce each constructor argument to its declared parameter type so a
        // literal materialises in the parameter's representation (e.g. "abc" as
        // `bytes3`, not a memory `string`) — the deployed constructor ABI-decodes
        // its arguments by parameter type, so a mismatched encoding reverts.
        let parameter_types = contract_definition
            .constructor()
            .map(|constructor| {
                constructor
                    .resolve_signature_types(LocationPolicy::Declared(None), &context.state.builder)
                    .0
            })
            .unwrap_or_default();
        let (ctor_args, block) = context.emit_coerced_arguments(arguments, &parameter_types, block);
        let builder = &context.state.builder;
        let result_type =
            crate::ast::Type::contract(builder.context, &contract_name, payable).into_mlir();
        let val = value.unwrap_or_else(|| {
            crate::ast::Value::constant(
                0,
                crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                builder,
                &block,
            )
            .into_mlir()
        });

        // Append operands in the ODS declaration order (val, salt, ctorArgs) so
        // the flat operand list matches `operand_segment_sizes` below: melior's
        // builder appends in call order, so adding the optional CREATE2 salt
        // *before* the variadic ctor args is required — appending it after would
        // transpose the salt and the first constructor argument (the salt value
        // would be passed to the constructor, and a ctor arg read as the salt).
        let mut new_builder = NewOperation::builder(builder.context, builder.unknown_location)
            .obj_name(StringAttribute::new(builder.context, &contract_name))
            .val(val);
        if let Some(salt) = salt {
            new_builder = new_builder.salt(salt);
        }
        let new_builder = new_builder.ctor_args(&ctor_args).out(result_type);
        let mut operation: Operation = new_builder.build().into();
        // Set `operand_segment_sizes` manually (val=1, salt=0|1, ctorArgs=N):
        // melior's ODS builder does not synthesize the attribute for this
        // `AttrSizedOperandSegments` op, so the dialect verifier rejects the op
        // without it. The `salt` segment is 1 when CREATE2 is requested, else 0
        // (and `try_call` is left unset).
        let ctor_args_count =
            i32::try_from(ctor_args.len()).expect("constructor argument count fits in i32");
        let salt_segment = i32::from(salt.is_some());
        let segment_sizes =
            DenseI32ArrayAttribute::new(builder.context, &[1, salt_segment, ctor_args_count]);
        operation.set_inherent_attribute("operand_segment_sizes", segment_sizes.into());
        let value = block
            .append_operation(operation)
            .result(0)
            .expect("sol.new always produces one result")
            .into();
        (Some(value), block)
    }
}
