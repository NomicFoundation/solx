//!
//! Function and built-in call lowering.
//!

/// Built-in function call lowering.
pub mod built_in;
/// Struct constructor call lowering.
pub mod struct_constructor;
/// Type conversions between Solidity and Sol dialect MLIR types.
pub mod type_conversion;

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::PositionalArguments;

use self::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::ExpressionEmitter;

/// Lowers a `FunctionCallExpression`, classifying the callee and routing to the
/// matching call kind.
pub struct CallEmitter<'emitter, 'state, 'context, 'block> {
    /// The parent expression emitter, for recursive subexpression lowering.
    expression_emitter: &'emitter ExpressionEmitter<'state, 'context, 'block>,
}

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Creates a call emitter borrowing its parent expression emitter.
    pub fn new(expression_emitter: &'emitter ExpressionEmitter<'state, 'context, 'block>) -> Self {
        Self { expression_emitter }
    }

    /// Lowers a function call by trying each call-kind handler in turn.
    ///
    /// External / library calls, `new`, and named-argument calls are lowered by
    /// later domains.
    pub fn emit_function_call(
        &self,
        call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let ArgumentsDeclaration::PositionalArguments(arguments) = &call.arguments() else {
            unimplemented!("named-argument call lowering");
        };

        if let Some(result) = self.try_emit_type_conversion(call, arguments, block)? {
            return Ok(result);
        }

        let callee = call.operand();
        if let Some(result) = self.try_emit_built_in_call(&callee, arguments, block)? {
            return Ok(result);
        }

        if let Some(result) = self.try_emit_member_built_in_call(call, arguments, block)? {
            return Ok(result);
        }

        if let Some(result) = self.try_emit_struct_constructor(call, arguments, block)? {
            return Ok(result);
        }

        self.emit_internal_call(&callee, arguments, block)
    }

    /// Emits a direct internal call `f(args)` to a contract function as a
    /// `sol.call` to its registered symbol.
    fn emit_internal_call(
        &self,
        _callee: &Expression,
        _arguments: &PositionalArguments,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("internal call")
    }

    /// Emits an explicit type conversion `T(x)` via [`TypeConversion`], or
    /// `Ok(None)` when the call is not a single-argument type conversion.
    fn try_emit_type_conversion(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        if !call.is_type_conversion() || arguments.len() != 1 {
            return Ok(None);
        }
        let Some(target_slang_type) = call.get_type() else {
            unimplemented!("untyped type conversion");
        };
        let builder = &self.expression_emitter.state.builder;
        let target_type = TypeConversion::resolve_slang_type(&target_slang_type, None, builder);

        let argument = arguments
            .iter()
            .next()
            .expect("argument count checked to be one");
        let (value, block) = self.expression_emitter.emit_value(&argument, block)?;
        let result =
            TypeConversion::from_target_type(target_type, builder).emit(value, builder, &block);
        Ok(Some((Some(result), block)))
    }
}
