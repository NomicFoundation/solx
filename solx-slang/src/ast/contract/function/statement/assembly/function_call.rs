//!
//! Yul function-call emission: EVM-opcode intrinsics and user-defined inlining.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use num_bigint::BigInt;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::YulExpression;
use slang_solidity_v2::ast::YulFunctionCallExpression;
use slang_solidity_v2::ast::YulStatement;
use solx_mlir::YulCmpPredicate;
use solx_mlir::YulValue;
use solx_mlir::ods::yul::*;

use crate::ast::BlockAnd;
use crate::ast::EmitYul;
use crate::ast::Type as AstType;
use crate::ast::contract::function::statement::assembly::YulContext;

// A Yul function call producing its result words. Yul evaluates arguments right-to-left; a
// `BuiltIn::Yul*` callee is a one-to-one `yul.*` EVM intrinsic, otherwise a user-defined function
// is inlined. An effect op (store/copy/log/terminator) yields its first operand for the statement to discard.
yul_emit!(YulFunctionCallExpression => BlockAnd<'context, 'block, Vec<YulValue<'context, 'block>>>; |call, context, block| {
    let YulExpression::YulPath(path) = call.operand() else {
        unimplemented!("unsupported yul callee expression");
    };
    let callee = path.iter().next().expect("empty yul function path");
    let argument_nodes: Vec<_> = call.arguments().iter().collect();
    let mut arguments: Vec<Option<YulValue<'context, 'block>>> = vec![None; argument_nodes.len()];
    let mut current = block;
    for (index, argument) in argument_nodes.iter().enumerate().rev() {
        let BlockAnd { value, block: next } = argument.emit(context, current);
        arguments[index] = Some(value);
        current = next;
    }
    let arguments: Vec<YulValue<'context, 'block>> = arguments
        .into_iter()
        .map(|value| value.expect("filled in the loop above"))
        .collect();

    let Some(opcode) = callee.resolve_to_built_in() else {
        // A user-defined Yul function is inlined (solc's MLIR backend asserts on these, so there is no
        // ground truth to mirror). Recursion is rejected.
        let name = callee.name();
        let depth = context.yul_inline_depth.entry(name.clone()).or_insert(0);
        if *depth >= 1 {
            unimplemented!("recursive yul function `{name}` cannot be inlined");
        }
        *depth += 1;
        let definition = context
            .yul_functions
            .get(&name)
            .cloned()
            .expect("yul function not registered");
        let parameters: Vec<_> = definition.parameters().iter().collect();
        let returns: Vec<_> = definition
            .returns()
            .map(|names| names.iter().collect::<Vec<_>>())
            .unwrap_or_default();

        let builder = &context.state.builder;
        context.environment.enter_scope();
        for (parameter, argument) in parameters.iter().zip(arguments.iter()) {
            let slot = YulValue::alloca(builder, &current);
            argument.store(slot, builder, &current);
            context.environment.define_variable(parameter.node_id(), slot);
        }
        for return_identifier in returns.iter() {
            let slot = YulValue::alloca(builder, &current);
            YulValue::constant(&BigInt::from(0u32), builder, &current).store(slot, builder, &current);
            context.environment.define_variable(return_identifier.node_id(), slot);
        }

        // Yul hoists nested functions: register them for this frame so calls
        // resolve regardless of textual order.
        let mut hoisted: Vec<String> = Vec::new();
        for inner in definition.body().statements().iter() {
            if let YulStatement::YulFunctionDefinition(nested) = &inner {
                let nested_name = nested.name().name();
                if !context.yul_functions.contains_key(&nested_name) {
                    context.yul_functions.insert(nested_name.clone(), nested.clone());
                    hoisted.push(nested_name);
                }
            }
        }
        for inner in definition.body().statements().iter() {
            if matches!(inner, YulStatement::YulFunctionDefinition(_)) {
                continue;
            }
            // `leave` returns from the function: stop emitting the body.
            if matches!(inner, YulStatement::YulLeaveStatement(_)) {
                break;
            }
            match inner.emit(context, current) {
                Some(next) => current = next,
                None => break,
            }
        }
        for nested_name in hoisted.iter() {
            context.yul_functions.remove(nested_name);
        }

        let mut return_values = Vec::with_capacity(returns.len());
        for return_identifier in returns.iter() {
            let slot = context.environment.variable(return_identifier.node_id());
            return_values.push(YulValue::load(slot, &context.state.builder, &current));
        }
        context.environment.exit_scope();
        if let Some(depth) = context.yul_inline_depth.get_mut(&name) {
            *depth = depth.saturating_sub(1);
        }
        return BlockAnd { value: return_values, block: current };
    };

    let builder = &context.state.builder;
    let i256 = AstType::signless(builder.context, solx_utils::BIT_LENGTH_FIELD).into_mlir();

    let value = match opcode {
        BuiltIn::YulAdd => YulValue::new(mlir_op!(builder, &current, AddOperation.lhs(arguments[0]).rhs(arguments[1]).out(i256))),
        BuiltIn::YulSub => YulValue::new(mlir_op!(builder, &current, SubOperation.lhs(arguments[0]).rhs(arguments[1]).out(i256))),
        BuiltIn::YulMul => YulValue::new(mlir_op!(builder, &current, MulOperation.lhs(arguments[0]).rhs(arguments[1]).out(i256))),
        BuiltIn::YulDiv => YulValue::new(mlir_op!(builder, &current, DivOperation.dividend(arguments[0]).divisor(arguments[1]).out(i256))),
        BuiltIn::YulSdiv => YulValue::new(mlir_op!(builder, &current, SDivOperation.dividend(arguments[0]).divisor(arguments[1]).out(i256))),
        BuiltIn::YulMod => YulValue::new(mlir_op!(builder, &current, ModOperation.value(arguments[0]).r#mod(arguments[1]).out(i256))),
        BuiltIn::YulSmod => YulValue::new(mlir_op!(builder, &current, SModOperation.value(arguments[0]).r#mod(arguments[1]).out(i256))),
        BuiltIn::YulExp => YulValue::new(mlir_op!(builder, &current, ExpOperation.base(arguments[0]).exp(arguments[1]).out(i256))),
        BuiltIn::YulAddmod => YulValue::new(mlir_op!(builder, &current, AddModOperation.x(arguments[0]).y(arguments[1]).r#mod(arguments[2]).out(i256))),
        BuiltIn::YulMulmod => YulValue::new(mlir_op!(builder, &current, MulModOperation.x(arguments[0]).y(arguments[1]).r#mod(arguments[2]).out(i256))),
        BuiltIn::YulAnd => YulValue::new(mlir_op!(builder, &current, AndOperation.lhs(arguments[0]).rhs(arguments[1]).out(i256))),
        BuiltIn::YulOr => YulValue::new(mlir_op!(builder, &current, OrOperation.lhs(arguments[0]).rhs(arguments[1]).out(i256))),
        BuiltIn::YulXor => YulValue::new(mlir_op!(builder, &current, XOrOperation.lhs(arguments[0]).rhs(arguments[1]).out(i256))),
        BuiltIn::YulNot => YulValue::new(mlir_op!(builder, &current, NotOperation.value(arguments[0]).out(i256))),
        // Yul `shl(shift, value)` / `shr` / `sar` shift the SECOND operand by the
        // first; the op operands are `(shift, val)` in that order.
        BuiltIn::YulShl => YulValue::new(mlir_op!(builder, &current, ShlOperation.shift(arguments[0]).val(arguments[1]).out(i256))),
        BuiltIn::YulShr => YulValue::new(mlir_op!(builder, &current, ShrOperation.shift(arguments[0]).val(arguments[1]).out(i256))),
        BuiltIn::YulSar => YulValue::new(mlir_op!(builder, &current, SarOperation.shift(arguments[0]).val(arguments[1]).out(i256))),
        BuiltIn::YulByte => YulValue::new(mlir_op!(builder, &current, ByteOperation.idx(arguments[0]).val(arguments[1]).out(i256))),
        BuiltIn::YulSignextend => YulValue::new(mlir_op!(builder, &current, SignExtendOperation.val(arguments[0]).off(arguments[1]).out(i256))),

        BuiltIn::YulLt => arguments[0].compare(arguments[1], YulCmpPredicate::Ult, builder, &current),
        BuiltIn::YulGt => arguments[0].compare(arguments[1], YulCmpPredicate::Ugt, builder, &current),
        BuiltIn::YulEq => arguments[0].compare(arguments[1], YulCmpPredicate::Eq, builder, &current),
        BuiltIn::YulSlt => arguments[0].compare(arguments[1], YulCmpPredicate::Slt, builder, &current),
        BuiltIn::YulSgt => arguments[0].compare(arguments[1], YulCmpPredicate::Sgt, builder, &current),
        BuiltIn::YulIszero => arguments[0].compare(
            YulValue::constant(&BigInt::from(0u32), builder, &current),
            YulCmpPredicate::Eq,
            builder,
            &current,
        ),

        BuiltIn::YulCaller => YulValue::new(mlir_op!(builder, &current, CallerOperation.out(i256))),
        BuiltIn::YulOrigin => YulValue::new(mlir_op!(builder, &current, OriginOperation.out(i256))),
        BuiltIn::YulCoinbase => YulValue::new(mlir_op!(builder, &current, CoinBaseOperation.out(i256))),
        BuiltIn::YulCallvalue => YulValue::new(mlir_op!(builder, &current, CallValOperation.out(i256))),
        BuiltIn::YulGasprice => YulValue::new(mlir_op!(builder, &current, GasPriceOperation.out(i256))),
        BuiltIn::YulTimestamp => YulValue::new(mlir_op!(builder, &current, TimeStampOperation.out(i256))),
        BuiltIn::YulNumber => YulValue::new(mlir_op!(builder, &current, NumberOperation.out(i256))),
        // Pre-merge `difficulty` and post-merge `prevrandao` are EVM opcode 0x44;
        // the Yul dialect exposes it as `prevrandao`.
        BuiltIn::YulDifficulty | BuiltIn::YulPrevrandao => {
            YulValue::new(mlir_op!(builder, &current, PrevrandaoOperation.out(i256)))
        }
        BuiltIn::YulChainid => YulValue::new(mlir_op!(builder, &current, ChainIdOperation.out(i256))),
        BuiltIn::YulBasefee => YulValue::new(mlir_op!(builder, &current, BaseFeeOperation.out(i256))),
        BuiltIn::YulBlobbasefee => YulValue::new(mlir_op!(builder, &current, BlobBaseFeeOperation.out(i256))),
        BuiltIn::YulGaslimit => YulValue::new(mlir_op!(builder, &current, GasLimitOperation.out(i256))),
        BuiltIn::YulGas => YulValue::new(mlir_op!(builder, &current, GasOperation.out(i256))),
        BuiltIn::YulBlockhash => YulValue::new(mlir_op!(builder, &current, BlockHashOperation.block(arguments[0]).out(i256))),
        BuiltIn::YulBlobhash => YulValue::new(mlir_op!(builder, &current, BlobHashOperation.idx(arguments[0]).out(i256))),

        BuiltIn::YulBalance => YulValue::new(mlir_op!(builder, &current, BalanceOperation.addr(arguments[0]).out(i256))),
        BuiltIn::YulExtcodehash => YulValue::new(mlir_op!(builder, &current, ExtCodeHashOperation.addr(arguments[0]).out(i256))),
        BuiltIn::YulExtcodesize => YulValue::new(mlir_op!(builder, &current, ExtCodeSizeOperation.addr(arguments[0]).out(i256))),
        BuiltIn::YulExtcodecopy => {
            mlir_op_void!(builder, &current, ExtCodeCopyOperation.addr(arguments[0]).dst(arguments[1]).src(arguments[2]).size(arguments[3]));
            arguments[0]
        }
        BuiltIn::YulAddress => YulValue::new(mlir_op!(builder, &current, AddressOperation.out(i256))),
        BuiltIn::YulSelfbalance => YulValue::new(mlir_op!(builder, &current, SelfBalanceOperation.out(i256))),

        BuiltIn::YulMload => YulValue::new(mlir_op!(builder, &current, MLoadOperation.addr(arguments[0]).out(i256))),
        BuiltIn::YulMstore => {
            mlir_op_void!(builder, &current, MStoreOperation.addr(arguments[0]).val(arguments[1]));
            arguments[0]
        }
        BuiltIn::YulMstore8 => {
            mlir_op_void!(builder, &current, MStore8Operation.addr(arguments[0]).val(arguments[1]));
            arguments[0]
        }
        BuiltIn::YulMcopy => {
            mlir_op_void!(builder, &current, MCopyOperation.dst(arguments[0]).src(arguments[1]).size(arguments[2]));
            arguments[0]
        }
        BuiltIn::YulMsize => YulValue::new(mlir_op!(builder, &current, MSizeOperation.out(i256))),
        BuiltIn::YulSload => YulValue::new(mlir_op!(builder, &current, SLoadOperation.addr(arguments[0]).out(i256))),
        BuiltIn::YulSstore => {
            mlir_op_void!(builder, &current, SStoreOperation.addr(arguments[0]).val(arguments[1]));
            arguments[0]
        }
        BuiltIn::YulTload => YulValue::new(mlir_op!(builder, &current, TLoadOperation.addr(arguments[0]).out(i256))),
        BuiltIn::YulTstore => {
            mlir_op_void!(builder, &current, TStoreOperation.addr(arguments[0]).val(arguments[1]));
            arguments[0]
        }

        BuiltIn::YulKeccak256 => YulValue::new(mlir_op!(builder, &current, Keccak256Operation.addr(arguments[0]).size(arguments[1]).out(i256))),

        BuiltIn::YulCalldataload => YulValue::new(mlir_op!(builder, &current, CallDataLoadOperation.addr(arguments[0]).out(i256))),
        BuiltIn::YulCalldatasize => YulValue::new(mlir_op!(builder, &current, CallDataSizeOperation.out(i256))),
        BuiltIn::YulCalldatacopy => {
            mlir_op_void!(builder, &current, CallDataCopyOperation.dst(arguments[0]).src(arguments[1]).size(arguments[2]));
            arguments[0]
        }
        BuiltIn::YulCodesize => YulValue::new(mlir_op!(builder, &current, CodeSizeOperation.out(i256))),
        BuiltIn::YulCodecopy => {
            mlir_op_void!(builder, &current, CodeCopyOperation.dst(arguments[0]).src(arguments[1]).size(arguments[2]));
            arguments[0]
        }
        BuiltIn::YulReturndatasize => YulValue::new(mlir_op!(builder, &current, ReturnDataSizeOperation.out(i256))),
        BuiltIn::YulReturndatacopy => {
            mlir_op_void!(builder, &current, ReturnDataCopyOperation.dst(arguments[0]).src(arguments[1]).size(arguments[2]));
            arguments[0]
        }

        // `yul.call`'s single result is `$status`, which `mlir_op!` extracts as
        // result 0.
        BuiltIn::YulCall => YulValue::new(mlir_op!(
            builder,
            &current,
            CallOperation
                .gas(arguments[0])
                .address(arguments[1])
                .value(arguments[2])
                .inp_offset(arguments[3])
                .inp_size(arguments[4])
                .out_offset(arguments[5])
                .out_size(arguments[6])
                .status(i256)
        )),
        BuiltIn::YulStaticcall => YulValue::new(mlir_op!(
            builder,
            &current,
            StaticCallOperation
                .gas(arguments[0])
                .address(arguments[1])
                .inp_offset(arguments[2])
                .inp_size(arguments[3])
                .out_offset(arguments[4])
                .out_size(arguments[5])
                .out(i256)
        )),
        BuiltIn::YulDelegatecall => YulValue::new(mlir_op!(
            builder,
            &current,
            DelegateCallOperation
                .gas(arguments[0])
                .address(arguments[1])
                .inp_offset(arguments[2])
                .inp_size(arguments[3])
                .out_offset(arguments[4])
                .out_size(arguments[5])
                .out(i256)
        )),
        BuiltIn::YulCallcode => YulValue::new(mlir_op!(
            builder,
            &current,
            CallCodeOperation
                .gas(arguments[0])
                .address(arguments[1])
                .value(arguments[2])
                .inp_offset(arguments[3])
                .inp_size(arguments[4])
                .out_offset(arguments[5])
                .out_size(arguments[6])
                .status(i256)
        )),

        BuiltIn::YulCreate => YulValue::new(mlir_op!(builder, &current, CreateOperation.val(arguments[0]).addr(arguments[1]).size(arguments[2]).out(i256))),
        BuiltIn::YulCreate2 => YulValue::new(mlir_op!(
            builder,
            &current,
            Create2Operation
                .val(arguments[0])
                .addr(arguments[1])
                .size(arguments[2])
                .salt(arguments[3])
                .out(i256)
        )),

        BuiltIn::YulLog => {
            let topics: Vec<_> = arguments[2..].iter().map(|value| value.into_mlir()).collect();
            mlir_op_void!(builder, &current, LogOperation.addr(arguments[0]).size(arguments[1]).topics(topics.as_slice()));
            arguments[0]
        }

        BuiltIn::YulReturn => {
            mlir_op_void!(builder, &current, ReturnOperation.addr(arguments[0]).size(arguments[1]));
            arguments[0]
        }
        BuiltIn::YulRevert => {
            mlir_op_void!(builder, &current, RevertOperation.addr(arguments[0]).size(arguments[1]));
            arguments[0]
        }
        BuiltIn::YulSelfdestruct => {
            mlir_op_void!(builder, &current, SelfDestructOperation.addr(arguments[0]));
            arguments[0]
        }
        BuiltIn::YulStop => {
            mlir_op_void!(builder, &current, StopOperation);
            YulValue::constant(&BigInt::from(0u32), builder, &current)
        }
        BuiltIn::YulInvalid => {
            mlir_op_void!(builder, &current, InvalidOperation);
            YulValue::constant(&BigInt::from(0u32), builder, &current)
        }

        // `pop(x)` evaluates and discards; the argument is already emitted.
        BuiltIn::YulPop => arguments[0],

        _ => unimplemented!("unsupported yul intrinsic: {opcode:?}"),
    };
    BlockAnd { value: vec![value], block: current }
});
