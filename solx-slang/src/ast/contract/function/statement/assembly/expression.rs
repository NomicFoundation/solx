//!
//! Yul expression emission: literals, path reads, calls.
//!

use melior::ir::BlockRef;
use num_bigint::BigInt;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::YulExpression;
use slang_solidity_v2::ast::YulPath;
use solx_mlir::YulValue;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::function::statement::assembly::YulContext;

// A Yul expression produces an `i256` word; a call collapses to its first result
// (`0` for a no-return user function), the value a statement-position call
// discards.
yul_emit!(YulExpression => (YulValue<'context, 'block>, BlockRef<'context, 'block>); |expression, context, block| {
    match expression {
        YulExpression::YulLiteral(literal) => {
            (YulValue::constant(&literal.value(), &context.state.builder, &block), block)
        }
        YulExpression::YulPath(path) => path.emit(context, block),
        YulExpression::YulFunctionCallExpression(call) => {
            let (values, block) = call.emit(context, block);
            let value = match values.into_iter().next() {
                Some(value) => value,
                None => YulValue::constant(&BigInt::from(0u32), &context.state.builder, &block),
            };
            (value, block)
        }
    }
});

// A Yul path read to a 256-bit word: a single-segment path resolves to a
// Solidity constant's widened initializer or a local/Yul variable's loaded value;
// a two-segment `x.slot` / `x.offset` (keyed by the typed `BuiltIn::YulSlot` /
// `BuiltIn::YulOffset` suffix — never the member name string) resolves to a state
// variable's slot index / in-slot byte offset.
yul_emit!(YulPath => (YulValue<'context, 'block>, BlockRef<'context, 'block>); |path, context, block| {
    let identifier = path.iter().next().expect("empty yul path");
    let builder = &context.state.builder;

    // A Solidity constant referenced in assembly resolves to a definition, not a
    // Yul/local variable; emit its initializer widened to a word. The mode is
    // immaterial — a constant initializer folds at compile time.
    if path.len() == 1
        && let Some(Definition::Constant(constant)) = identifier.resolve_to_definition()
    {
        let initializer = constant.value().expect("slang validated");
        let emitter = ExpressionContext::new(
            context.state,
            context.environment,
            context.storage_layout,
            ArithmeticMode::Checked,
        );
        let BlockAnd { value, block } = initializer.emit(&emitter, block);
        let widened = value.cast(
            AstType::signless(builder.context, solx_utils::BIT_LENGTH_FIELD),
            builder,
            &block,
        );
        return (YulValue::new(widened.into_mlir()), block);
    }

    if path.len() == 2 {
        let parts: Vec<_> = path.iter().collect();
        if let Some(Definition::StateVariable(state_variable)) = parts[0].resolve_to_definition() {
            let slot = context
                .storage_layout
                .get(&state_variable.node_id())
                .expect("unregistered state variable");
            match parts[1].resolve_to_built_in() {
                Some(BuiltIn::YulSlot) => {
                    let slot_word =
                        BigInt::from_bytes_be(num_bigint::Sign::Plus, &slot.slot.to_be_bytes_vec());
                    return (YulValue::constant(&slot_word, builder, &block), block);
                }
                Some(BuiltIn::YulOffset) => {
                    return (
                        YulValue::constant(&BigInt::from(slot.byte_offset), builder, &block),
                        block,
                    );
                }
                _ => {}
            }
        }

        // `localRef.slot` / `localRef.offset` for a `storage` reference local
        // (`T storage x = …`): the local stores the slot index, and a storage
        // reference is slot-aligned, so the in-slot byte offset is 0 (matching
        // solc). Without this, the fall-through below loads the local for both
        // suffixes, so `.offset` wrongly returns the slot.
        if matches!(
            parts[0].resolve_to_definition(),
            Some(Definition::Variable(_) | Definition::Parameter(_))
        ) {
            match parts[1].resolve_to_built_in() {
                Some(BuiltIn::YulSlot) => {
                    let declaration = parts[0]
                        .resolve_to_definition()
                        .expect("yul path head resolves to a declaration")
                        .node_id();
                    let slot = AstValue::from(context.environment.variable(declaration))
                        .reinterpret(AstType::llvm_ptr(builder.context), builder, &block)
                        .into_mlir();
                    return (YulValue::load(slot, builder, &block), block);
                }
                Some(BuiltIn::YulOffset) => {
                    return (YulValue::constant(&BigInt::from(0u32), builder, &block), block);
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
        .reinterpret(AstType::llvm_ptr(builder.context), builder, &block)
        .into_mlir();
    (YulValue::load(slot, builder, &block), block)
});
