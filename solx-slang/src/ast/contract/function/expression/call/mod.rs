//!
//! Function and built-in call lowering.
//!

/// Type conversions between Solidity and Sol dialect MLIR types.
pub mod type_conversion;

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::PositionalArguments;

use self::type_conversion::TypeConversion;
use super::ExpressionEmitter;

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
    /// Built-in dispatch, internal/external/library calls, struct constructors,
    /// `new`, and named-argument calls are lowered by later domains.
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

        unimplemented!(
            "call lowering: {:?}",
            std::mem::discriminant(&call.operand())
        )
    }

    /// Emits an explicit type conversion `T(x)` (e.g. `uint256(x)`, `uint8(x)`)
    /// as a `sol.cast`/`sol.address_cast`/comparison via [`TypeConversion`].
    ///
    /// Returns `None` when the call is not a single-argument type conversion.
    /// Conversions slang leaves untyped (enum, `bytes` of a constant) and
    /// single-field struct constructors (which slang also reports as
    /// conversions) defer to later domains.
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
