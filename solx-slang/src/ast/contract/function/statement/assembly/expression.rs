//!
//! Yul expression emission: literals, path reads, calls.
//!

use melior::ir::BlockRef;
use num_bigint::BigInt;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::YulExpression;
use slang_solidity_v2::ast::YulPath;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::YulValue;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::function::statement::assembly::YulContext;
use crate::ast::contract::storage_layout::StateVariableSlot;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_yul::EmitYul;

yul_emit!(YulExpression => BlockAnd<'context, 'block, YulValue<'context, 'block>>; |expression, context, block| {
    match expression {
        YulExpression::YulLiteral(literal) => {
            BlockAnd { value: YulValue::constant(&literal.value(), context.state, &block), block }
        }
        YulExpression::YulPath(path) => path.emit(context, block),
        YulExpression::YulFunctionCallExpression(call) => {
            let BlockAnd { value: values, block } = call.emit(context, block);
            let value = match values.into_iter().next() {
                Some(value) => value,
                None => YulValue::constant(&BigInt::from(0u32), context.state, &block),
            };
            BlockAnd { value, block }
        }
    }
});

yul_emit!(YulPath => BlockAnd<'context, 'block, YulValue<'context, 'block>>; |path, context, block| {
    let identifier = path.iter().next().expect("empty yul path");
    let state = context.state;

    if path.len() == 1
        && let Some(Definition::Constant(constant)) = identifier.resolve_to_definition()
    {
        let initializer = constant.value().expect("slang validated");
        let emitter = ExpressionContext::new(
            context.state,
            context.environment,
            context.dispatch,
            context.storage_layout,
            None,
            ArithmeticMode::Checked,
        );
        let BlockAnd { value, block } = initializer.emit(&emitter, block);
        let widened = value.cast(
            AstType::signless(state.mlir_context, solx_utils::BIT_LENGTH_FIELD),
            state,
            &block,
        );
        return BlockAnd { value: YulValue::new(widened.into_mlir()), block };
    }

    if path.len() == 2 {
        let parts: Vec<_> = path.iter().collect();
        let head_definition = parts[0].resolve_to_definition();
        if let Some(Definition::StateVariable(state_variable)) = &head_definition {
            let slot = context.storage_layout.slot(state_variable.node_id());
            match parts[1].resolve_to_built_in() {
                Some(BuiltIn::YulSlot) => {
                    let slot_word =
                        BigInt::from_bytes_be(num_bigint::Sign::Plus, &slot.slot.to_be_bytes_vec());
                    return BlockAnd { value: YulValue::constant(&slot_word, state, &block), block };
                }
                Some(BuiltIn::YulOffset) => {
                    return BlockAnd {
                        value: YulValue::constant(&BigInt::from(slot.byte_offset), state, &block),
                        block,
                    };
                }
                _ => {}
            }
        }

        if matches!(
            head_definition,
            Some(Definition::Variable(_) | Definition::Parameter(_))
        ) {
            match parts[1].resolve_to_built_in() {
                Some(BuiltIn::YulSlot) => {
                    let declaration = head_definition
                        .as_ref()
                        .expect("yul path head resolves to a declaration")
                        .node_id();
                    let slot = AstValue::from(context.environment.variable(declaration))
                        .reinterpret(AstType::llvm_ptr(state.mlir_context), state, &block)
                        .into_mlir();
                    return BlockAnd { value: YulValue::load(slot, state, &block), block };
                }
                Some(BuiltIn::YulOffset) => {
                    return BlockAnd { value: YulValue::constant(&BigInt::from(0u32), state, &block), block };
                }
                _ => {}
            }
        }
    }

    let declaration = identifier
        .resolve_to_definition()
        .expect("yul variable reference resolves to a declaration")
        .node_id();
    let slot = AstValue::from(context.environment.variable(declaration))
        .reinterpret(AstType::llvm_ptr(state.mlir_context), state, &block)
        .into_mlir();
    BlockAnd { value: YulValue::load(slot, state, &block), block }
});
