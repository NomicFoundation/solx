//!
//! Function call and member access expression lowering.
//!

pub mod built_in;
pub mod type_conversion;

use anyhow::Context as _;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::r#type::IntegerType;
use slang_solidity::backend::ir::ast::ArgumentsDeclaration;
use slang_solidity::backend::ir::ast::Definition;
use slang_solidity::backend::ir::ast::Expression;
use slang_solidity::backend::ir::ast::FunctionCallExpression;
use slang_solidity::backend::ir::ast::MemberAccessExpression;
use slang_solidity::backend::ir::ast::PositionalArguments;
use slang_solidity::backend::ir::ast::StructDefinition;

use crate::ast::contract::function::expression::ExpressionEmitter;

use self::type_conversion::TypeConversion;

/// Lowers function call and member access expressions to MLIR.
pub struct CallEmitter<'emitter, 'state, 'context, 'block> {
    /// The parent expression emitter for recursive subexpression emission.
    expression_emitter: &'emitter ExpressionEmitter<'state, 'context, 'block>,
}

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Creates a new call emitter.
    pub fn new(expression_emitter: &'emitter ExpressionEmitter<'state, 'context, 'block>) -> Self {
        Self { expression_emitter }
    }

    /// Emits a function call expression.
    ///
    /// Handles type conversions and built-in dispatch, then resolves
    /// user-defined callees through slang's binder to a function definition
    /// node id and looks up the registered MLIR signature.
    ///
    /// # Errors
    ///
    /// Returns an error if the callee is unsupported, arguments contain
    /// unsupported constructs, or the callee does not resolve to a registered
    /// function definition.
    pub fn emit_function_call(
        &self,
        call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let ArgumentsDeclaration::PositionalArguments(positional_arguments) = &call.arguments()
        else {
            anyhow::bail!("only positional arguments supported");
        };

        let callee = call.operand();

        if call.is_type_conversion() && positional_arguments.len() == 1 {
            let first = positional_arguments
                .iter()
                .next()
                .expect("len checked to be 1 above");
            let (value, block) = self.expression_emitter.emit_value(&first, block)?;
            let builder = &self.expression_emitter.state.builder;

            let target_type = self
                .expression_emitter
                .resolve_slang_type(call.get_type())
                .ok_or_else(|| anyhow::anyhow!("unresolved type conversion target"))?;

            let result =
                TypeConversion::from_target_type(target_type, builder).emit(value, builder, &block);
            return Ok((Some(result), block));
        }

        if let Some(block) = self.try_emit_built_in_call(&callee, positional_arguments, block)? {
            return Ok((None, block));
        }

        let Expression::Identifier(callee_identifier) = &callee else {
            anyhow::bail!("unsupported callee expression");
        };
        let function_definition = match callee_identifier.resolve_to_definition() {
            Some(Definition::Function(function_definition)) => function_definition,
            Some(Definition::Struct(struct_definition)) => {
                let result_type = self
                    .expression_emitter
                    .resolve_slang_type(call.get_type())
                    .ok_or_else(|| anyhow::anyhow!("unresolved struct constructor type"))?;
                return self
                    .emit_struct_constructor(
                        &struct_definition,
                        result_type,
                        positional_arguments,
                        block,
                    )
                    .map(|(value, block)| (Some(value), block));
            }
            _ => anyhow::bail!(
                "callee '{}' does not resolve to a function",
                callee_identifier.name()
            ),
        };

        let mut argument_values = Vec::new();
        let mut current_block = block;

        for argument in positional_arguments.iter() {
            let (value, next_block) = self
                .expression_emitter
                .emit_value(&argument, current_block)?;
            argument_values.push(value);
            current_block = next_block;
        }

        let (mlir_name, parameter_types, return_types) = self
            .expression_emitter
            .state
            .resolve_function(function_definition.node_id().into())
            .with_context(|| format!("resolving callee '{}'", callee_identifier.name()))?;

        // Cast arguments to match the callee's declared parameter types.
        let builder = &self.expression_emitter.state.builder;
        for (value, &param_type) in argument_values.iter_mut().zip(parameter_types) {
            let conversion = TypeConversion::from_target_type(param_type, builder);
            *value = conversion.emit(*value, builder, &current_block);
        }

        if return_types.is_empty() {
            self.expression_emitter.state.builder.emit_sol_call(
                mlir_name,
                &argument_values,
                &[],
                &current_block,
            )?;
            Ok((None, current_block))
        } else {
            let result = self
                .expression_emitter
                .state
                .builder
                .emit_sol_call(mlir_name, &argument_values, return_types, &current_block)?
                .expect("function call always produces at least one result");
            Ok((Some(result), current_block))
        }
    }

    /// Emits a struct-literal constructor `S(a, b, c)` in memory.
    ///
    /// Allocates the struct via `sol.malloc` and stores each positional
    /// argument into the matching field through a `sol.gep` field index.
    fn emit_struct_constructor(
        &self,
        struct_definition: &StructDefinition,
        result_type: Type<'context>,
        positional_arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let members = struct_definition.members();
        anyhow::ensure!(
            positional_arguments.len() == members.len(),
            "struct constructor '{}' expects {} arguments, got {}",
            struct_definition.name().name(),
            members.len(),
            positional_arguments.len()
        );
        let builder = &self.expression_emitter.state.builder;
        let struct_address = builder.emit_sol_malloc(result_type, &block);
        let ui64_type = Type::from(IntegerType::unsigned(builder.context, 64));
        let mut current_block = block;
        for (index, (member, argument)) in
            members.iter().zip(positional_arguments.iter()).enumerate()
        {
            let member_slang_type = member
                .get_type()
                .ok_or_else(|| anyhow::anyhow!("struct member has no resolved type"))?;
            let field_type = TypeConversion::resolve_slang_type(
                &member_slang_type,
                Some(solx_utils::DataLocation::Memory),
                builder,
            );
            let address_type = builder
                .types
                .pointer(field_type, solx_utils::DataLocation::Memory);
            let index_value = builder.emit_sol_constant(index as i64, ui64_type, &current_block);
            let field_address =
                builder.emit_sol_gep(struct_address, index_value, address_type, &current_block);
            let (argument_value, next_block) = self
                .expression_emitter
                .emit_value(&argument, current_block)?;
            let stored = TypeConversion::from_target_type(field_type, builder).emit(
                argument_value,
                builder,
                &next_block,
            );
            builder.emit_sol_store(stored, field_address, &next_block);
            current_block = next_block;
        }
        Ok((struct_address, current_block))
    }

    /// Emits a member access expression (e.g. `tx.origin`, `msg.sender`).
    ///
    /// Resolves the member via slang's binder to a specific `BuiltIn` variant
    /// rather than string-matching the member name.
    ///
    /// # Errors
    ///
    /// Returns an error if the member access is not a recognized EVM intrinsic.
    pub fn emit_member_access(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        self.emit_built_in_member_access(access, block)
    }
}
