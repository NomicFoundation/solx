//!
//! Calls whose callee is a `new` expression.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::Type as SlangType;
use slang_solidity_v2::ast::TypeName as SlangTypeName;

use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::call::CallContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'emitter, 'state, 'context, 'block> CallContext<'emitter, 'state, 'context, 'block> {
    /// Emits a `new` call: `new T[](n)` / `new bytes(n)` / `new string(n)` dynamic allocation, or
    /// `new C(args)` contract creation.
    ///
    /// The array, `bytes`, and `string` forms allocate a dynamic memory aggregate of `n` elements via
    /// a zeroed `sol.malloc`. The array forms resolve a call type; `new bytes` / `new string` surface
    /// none, so the syntactic elementary type name selects the `string` aggregate.
    pub(super) fn emit_new_expression(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        call_value: Option<Value<'context, 'block>>,
        salt: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let context = self.expression_context.state;
        let slang_type = call.get_type();
        let dynamic_result_type = match &slang_type {
            Some(inner @ (SlangType::Array(_) | SlangType::Bytes(_) | SlangType::String(_))) => {
                Some(TypeConversion::resolve_slang_type(
                    inner,
                    Some(DataLocation::Memory),
                    context,
                ))
            }
            None if matches!(
                call.operand(),
                Expression::NewExpression(new_expression)
                    if matches!(new_expression.type_name(), SlangTypeName::ElementaryType(_))
            ) =>
            {
                Some(AstType::string(context.mlir_context, DataLocation::Memory).into_mlir())
            }
            _ => None,
        };
        if let Some(result_type) = dynamic_result_type {
            let (values, current_block) = self.emit_argument_values(arguments, block);
            let size = AstValue::from(*values.first().expect("slang validates the size argument"))
                .cast(
                    AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD),
                    context,
                    &current_block,
                )
                .into_mlir();
            let address = AstValue::malloc(
                AstType::new(result_type),
                Some(size),
                true,
                context,
                &current_block,
            )
            .into_mlir();
            return (address, current_block);
        }

        let Some(SlangType::Contract(contract_type)) = slang_type else {
            unimplemented!("new expression has no resolved type or unsupported new target");
        };
        let Definition::Contract(contract_definition) = contract_type.definition() else {
            unreachable!("Slang ContractType always references a Contract definition");
        };
        let ordered_arguments: Vec<Expression> = arguments.iter().collect();
        self.emit_contract_creation(
            &contract_definition,
            &ordered_arguments,
            call_value,
            salt,
            block,
        )
    }
}
