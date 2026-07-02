//!
//! Yul function-call emission: EVM-opcode intrinsics and user-defined inlining.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use num_bigint::BigInt;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::YulExpression;
use slang_solidity_v2::ast::YulFunctionCallExpression;
use slang_solidity_v2::ast::YulStatement;

use solx_mlir::Type as AstType;
use solx_mlir::YulCmpPredicate;
use solx_mlir::YulValue;
use solx_mlir::ods::yul::AddModOperation;
use solx_mlir::ods::yul::AddOperation;
use solx_mlir::ods::yul::AddressOperation;
use solx_mlir::ods::yul::AndOperation;
use solx_mlir::ods::yul::BalanceOperation;
use solx_mlir::ods::yul::BaseFeeOperation;
use solx_mlir::ods::yul::BlobBaseFeeOperation;
use solx_mlir::ods::yul::BlobHashOperation;
use solx_mlir::ods::yul::BlockHashOperation;
use solx_mlir::ods::yul::ByteOperation;
use solx_mlir::ods::yul::CallCodeOperation;
use solx_mlir::ods::yul::CallDataCopyOperation;
use solx_mlir::ods::yul::CallDataLoadOperation;
use solx_mlir::ods::yul::CallDataSizeOperation;
use solx_mlir::ods::yul::CallOperation;
use solx_mlir::ods::yul::CallValOperation;
use solx_mlir::ods::yul::CallerOperation;
use solx_mlir::ods::yul::ChainIdOperation;
use solx_mlir::ods::yul::CodeCopyOperation;
use solx_mlir::ods::yul::CodeSizeOperation;
use solx_mlir::ods::yul::CoinBaseOperation;
use solx_mlir::ods::yul::Create2Operation;
use solx_mlir::ods::yul::CreateOperation;
use solx_mlir::ods::yul::DelegateCallOperation;
use solx_mlir::ods::yul::DivOperation;
use solx_mlir::ods::yul::ExpOperation;
use solx_mlir::ods::yul::ExtCodeCopyOperation;
use solx_mlir::ods::yul::ExtCodeHashOperation;
use solx_mlir::ods::yul::ExtCodeSizeOperation;
use solx_mlir::ods::yul::GasLimitOperation;
use solx_mlir::ods::yul::GasOperation;
use solx_mlir::ods::yul::GasPriceOperation;
use solx_mlir::ods::yul::InvalidOperation;
use solx_mlir::ods::yul::Keccak256Operation;
use solx_mlir::ods::yul::LogOperation;
use solx_mlir::ods::yul::MCopyOperation;
use solx_mlir::ods::yul::MLoadOperation;
use solx_mlir::ods::yul::MSizeOperation;
use solx_mlir::ods::yul::MStore8Operation;
use solx_mlir::ods::yul::MStoreOperation;
use solx_mlir::ods::yul::ModOperation;
use solx_mlir::ods::yul::MulModOperation;
use solx_mlir::ods::yul::MulOperation;
use solx_mlir::ods::yul::NotOperation;
use solx_mlir::ods::yul::NumberOperation;
use solx_mlir::ods::yul::OrOperation;
use solx_mlir::ods::yul::OriginOperation;
use solx_mlir::ods::yul::PrevrandaoOperation;
use solx_mlir::ods::yul::ReturnDataCopyOperation;
use solx_mlir::ods::yul::ReturnDataSizeOperation;
use solx_mlir::ods::yul::ReturnOperation;
use solx_mlir::ods::yul::RevertOperation;
use solx_mlir::ods::yul::SDivOperation;
use solx_mlir::ods::yul::SLoadOperation;
use solx_mlir::ods::yul::SModOperation;
use solx_mlir::ods::yul::SStoreOperation;
use solx_mlir::ods::yul::SarOperation;
use solx_mlir::ods::yul::SelfBalanceOperation;
use solx_mlir::ods::yul::SelfDestructOperation;
use solx_mlir::ods::yul::ShlOperation;
use solx_mlir::ods::yul::ShrOperation;
use solx_mlir::ods::yul::SignExtendOperation;
use solx_mlir::ods::yul::StaticCallOperation;
use solx_mlir::ods::yul::StopOperation;
use solx_mlir::ods::yul::SubOperation;
use solx_mlir::ods::yul::TLoadOperation;
use solx_mlir::ods::yul::TStoreOperation;
use solx_mlir::ods::yul::TimeStampOperation;
use solx_mlir::ods::yul::XOrOperation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::statement::assembly::YulContext;
use crate::ast::emit::emit_yul::EmitYul;

yul_emit!(YulFunctionCallExpression => BlockAnd<'context, 'block, Vec<YulValue<'context, 'block>>>; |call, context, block| {
    let YulExpression::YulPath(path) = call.operand() else {
        unreachable!("unsupported yul callee expression");
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
        let function_id = callee
            .resolve_to_definition()
            .expect("yul function call resolves to a definition")
            .node_id();
        let depth = context.yul_inline_depth.entry(function_id).or_insert(0);
        if *depth >= 1 {
            unimplemented!("recursive yul function `{}` cannot be inlined", callee.name());
        }
        *depth += 1;
        let definition = context
            .yul_functions
            .get(&function_id)
            .cloned()
            .expect("yul function not registered");
        let parameters: Vec<_> = definition.parameters().iter().collect();
        let returns: Vec<_> = definition
            .returns()
            .map(|names| names.iter().collect::<Vec<_>>())
            .unwrap_or_default();

        let state = context.state;
        context.environment.enter_scope();
        for (parameter, argument) in parameters.iter().zip(arguments.iter()) {
            let slot = YulValue::alloca(state, &current);
            argument.store(slot, state, &current);
            context.environment.define_variable(parameter.node_id(), slot);
        }
        for return_identifier in returns.iter() {
            let slot = YulValue::alloca(state, &current);
            YulValue::constant(&BigInt::from(0u32), state, &current).store(slot, state, &current);
            context.environment.define_variable(return_identifier.node_id(), slot);
        }

        let mut hoisted: Vec<NodeId> = Vec::new();
        for inner in definition.body().statements().iter() {
            if let YulStatement::YulFunctionDefinition(nested) = &inner {
                let nested_id = nested.node_id();
                if context
                    .yul_functions
                    .insert(nested_id, nested.clone())
                    .is_none()
                {
                    hoisted.push(nested_id);
                }
            }
        }
        for inner in definition.body().statements().iter() {
            if matches!(inner, YulStatement::YulFunctionDefinition(_)) {
                continue;
            }
            if matches!(inner, YulStatement::YulLeaveStatement(_)) {
                break;
            }
            match inner.emit(context, current) {
                Some(next) => current = next,
                None => break,
            }
        }
        for nested_id in hoisted.iter() {
            context.yul_functions.remove(nested_id);
        }

        let mut return_values = Vec::with_capacity(returns.len());
        for return_identifier in returns.iter() {
            let slot = context.environment.variable(return_identifier.node_id());
            return_values.push(YulValue::load(slot, context.state, &current));
        }
        context.environment.exit_scope();
        *context
            .yul_inline_depth
            .get_mut(&function_id)
            .expect("inline depth recorded on entry") -= 1;
        return BlockAnd { value: return_values, block: current };
    };

    let state = context.state;
    let i256 = AstType::signless(state.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir();

    let value = match opcode {
        BuiltIn::YulAdd => YulValue::new(mlir_op!(state, &current, AddOperation.lhs(arguments[0]).rhs(arguments[1]).out(i256))),
        BuiltIn::YulSub => YulValue::new(mlir_op!(state, &current, SubOperation.lhs(arguments[0]).rhs(arguments[1]).out(i256))),
        BuiltIn::YulMul => YulValue::new(mlir_op!(state, &current, MulOperation.lhs(arguments[0]).rhs(arguments[1]).out(i256))),
        BuiltIn::YulDiv => YulValue::new(mlir_op!(state, &current, DivOperation.dividend(arguments[0]).divisor(arguments[1]).out(i256))),
        BuiltIn::YulSdiv => YulValue::new(mlir_op!(state, &current, SDivOperation.dividend(arguments[0]).divisor(arguments[1]).out(i256))),
        BuiltIn::YulMod => YulValue::new(mlir_op!(state, &current, ModOperation.value(arguments[0]).r#mod(arguments[1]).out(i256))),
        BuiltIn::YulSmod => YulValue::new(mlir_op!(state, &current, SModOperation.value(arguments[0]).r#mod(arguments[1]).out(i256))),
        BuiltIn::YulExp => YulValue::new(mlir_op!(state, &current, ExpOperation.base(arguments[0]).exp(arguments[1]).out(i256))),
        BuiltIn::YulAddmod => YulValue::new(mlir_op!(state, &current, AddModOperation.x(arguments[0]).y(arguments[1]).r#mod(arguments[2]).out(i256))),
        BuiltIn::YulMulmod => YulValue::new(mlir_op!(state, &current, MulModOperation.x(arguments[0]).y(arguments[1]).r#mod(arguments[2]).out(i256))),
        BuiltIn::YulAnd => YulValue::new(mlir_op!(state, &current, AndOperation.lhs(arguments[0]).rhs(arguments[1]).out(i256))),
        BuiltIn::YulOr => YulValue::new(mlir_op!(state, &current, OrOperation.lhs(arguments[0]).rhs(arguments[1]).out(i256))),
        BuiltIn::YulXor => YulValue::new(mlir_op!(state, &current, XOrOperation.lhs(arguments[0]).rhs(arguments[1]).out(i256))),
        BuiltIn::YulNot => YulValue::new(mlir_op!(state, &current, NotOperation.value(arguments[0]).out(i256))),
        BuiltIn::YulShl => YulValue::new(mlir_op!(state, &current, ShlOperation.shift(arguments[0]).val(arguments[1]).out(i256))),
        BuiltIn::YulShr => YulValue::new(mlir_op!(state, &current, ShrOperation.shift(arguments[0]).val(arguments[1]).out(i256))),
        BuiltIn::YulSar => YulValue::new(mlir_op!(state, &current, SarOperation.shift(arguments[0]).val(arguments[1]).out(i256))),
        BuiltIn::YulByte => YulValue::new(mlir_op!(state, &current, ByteOperation.idx(arguments[0]).val(arguments[1]).out(i256))),
        BuiltIn::YulSignextend => YulValue::new(mlir_op!(state, &current, SignExtendOperation.val(arguments[0]).off(arguments[1]).out(i256))),

        BuiltIn::YulLt => arguments[0].compare(arguments[1], YulCmpPredicate::Ult, state, &current),
        BuiltIn::YulGt => arguments[0].compare(arguments[1], YulCmpPredicate::Ugt, state, &current),
        BuiltIn::YulEq => arguments[0].compare(arguments[1], YulCmpPredicate::Eq, state, &current),
        BuiltIn::YulSlt => arguments[0].compare(arguments[1], YulCmpPredicate::Slt, state, &current),
        BuiltIn::YulSgt => arguments[0].compare(arguments[1], YulCmpPredicate::Sgt, state, &current),
        BuiltIn::YulIszero => arguments[0].compare(
            YulValue::constant(&BigInt::from(0u32), state, &current),
            YulCmpPredicate::Eq,
            state,
            &current,
        ),

        BuiltIn::YulCaller => YulValue::new(mlir_op!(state, &current, CallerOperation.out(i256))),
        BuiltIn::YulOrigin => YulValue::new(mlir_op!(state, &current, OriginOperation.out(i256))),
        BuiltIn::YulCoinbase => YulValue::new(mlir_op!(state, &current, CoinBaseOperation.out(i256))),
        BuiltIn::YulCallvalue => YulValue::new(mlir_op!(state, &current, CallValOperation.out(i256))),
        BuiltIn::YulGasprice => YulValue::new(mlir_op!(state, &current, GasPriceOperation.out(i256))),
        BuiltIn::YulTimestamp => YulValue::new(mlir_op!(state, &current, TimeStampOperation.out(i256))),
        BuiltIn::YulNumber => YulValue::new(mlir_op!(state, &current, NumberOperation.out(i256))),
        BuiltIn::YulDifficulty | BuiltIn::YulPrevrandao => {
            YulValue::new(mlir_op!(state, &current, PrevrandaoOperation.out(i256)))
        }
        BuiltIn::YulChainid => YulValue::new(mlir_op!(state, &current, ChainIdOperation.out(i256))),
        BuiltIn::YulBasefee => YulValue::new(mlir_op!(state, &current, BaseFeeOperation.out(i256))),
        BuiltIn::YulBlobbasefee => YulValue::new(mlir_op!(state, &current, BlobBaseFeeOperation.out(i256))),
        BuiltIn::YulGaslimit => YulValue::new(mlir_op!(state, &current, GasLimitOperation.out(i256))),
        BuiltIn::YulGas => YulValue::new(mlir_op!(state, &current, GasOperation.out(i256))),
        BuiltIn::YulBlockhash => YulValue::new(mlir_op!(state, &current, BlockHashOperation.block(arguments[0]).out(i256))),
        BuiltIn::YulBlobhash => YulValue::new(mlir_op!(state, &current, BlobHashOperation.idx(arguments[0]).out(i256))),

        BuiltIn::YulBalance => YulValue::new(mlir_op!(state, &current, BalanceOperation.addr(arguments[0]).out(i256))),
        BuiltIn::YulExtcodehash => YulValue::new(mlir_op!(state, &current, ExtCodeHashOperation.addr(arguments[0]).out(i256))),
        BuiltIn::YulExtcodesize => YulValue::new(mlir_op!(state, &current, ExtCodeSizeOperation.addr(arguments[0]).out(i256))),
        BuiltIn::YulExtcodecopy => {
            mlir_op_void!(state, &current, ExtCodeCopyOperation.addr(arguments[0]).dst(arguments[1]).src(arguments[2]).size(arguments[3]));
            arguments[0]
        }
        BuiltIn::YulAddress => YulValue::new(mlir_op!(state, &current, AddressOperation.out(i256))),
        BuiltIn::YulSelfbalance => YulValue::new(mlir_op!(state, &current, SelfBalanceOperation.out(i256))),

        BuiltIn::YulMload => YulValue::new(mlir_op!(state, &current, MLoadOperation.addr(arguments[0]).out(i256))),
        BuiltIn::YulMstore => {
            mlir_op_void!(state, &current, MStoreOperation.addr(arguments[0]).val(arguments[1]));
            arguments[0]
        }
        BuiltIn::YulMstore8 => {
            mlir_op_void!(state, &current, MStore8Operation.addr(arguments[0]).val(arguments[1]));
            arguments[0]
        }
        BuiltIn::YulMcopy => {
            mlir_op_void!(state, &current, MCopyOperation.dst(arguments[0]).src(arguments[1]).size(arguments[2]));
            arguments[0]
        }
        BuiltIn::YulMsize => YulValue::new(mlir_op!(state, &current, MSizeOperation.out(i256))),
        BuiltIn::YulSload => YulValue::new(mlir_op!(state, &current, SLoadOperation.addr(arguments[0]).out(i256))),
        BuiltIn::YulSstore => {
            mlir_op_void!(state, &current, SStoreOperation.addr(arguments[0]).val(arguments[1]));
            arguments[0]
        }
        BuiltIn::YulTload => YulValue::new(mlir_op!(state, &current, TLoadOperation.addr(arguments[0]).out(i256))),
        BuiltIn::YulTstore => {
            mlir_op_void!(state, &current, TStoreOperation.addr(arguments[0]).val(arguments[1]));
            arguments[0]
        }

        BuiltIn::YulKeccak256 => YulValue::new(mlir_op!(state, &current, Keccak256Operation.addr(arguments[0]).size(arguments[1]).out(i256))),

        BuiltIn::YulCalldataload => YulValue::new(mlir_op!(state, &current, CallDataLoadOperation.addr(arguments[0]).out(i256))),
        BuiltIn::YulCalldatasize => YulValue::new(mlir_op!(state, &current, CallDataSizeOperation.out(i256))),
        BuiltIn::YulCalldatacopy => {
            mlir_op_void!(state, &current, CallDataCopyOperation.dst(arguments[0]).src(arguments[1]).size(arguments[2]));
            arguments[0]
        }
        BuiltIn::YulCodesize => YulValue::new(mlir_op!(state, &current, CodeSizeOperation.out(i256))),
        BuiltIn::YulCodecopy => {
            mlir_op_void!(state, &current, CodeCopyOperation.dst(arguments[0]).src(arguments[1]).size(arguments[2]));
            arguments[0]
        }
        BuiltIn::YulReturndatasize => YulValue::new(mlir_op!(state, &current, ReturnDataSizeOperation.out(i256))),
        BuiltIn::YulReturndatacopy => {
            mlir_op_void!(state, &current, ReturnDataCopyOperation.dst(arguments[0]).src(arguments[1]).size(arguments[2]));
            arguments[0]
        }

        BuiltIn::YulCall => YulValue::new(mlir_op!(
            state,
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
            state,
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
            state,
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
            state,
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

        BuiltIn::YulCreate => YulValue::new(mlir_op!(state, &current, CreateOperation.val(arguments[0]).addr(arguments[1]).size(arguments[2]).out(i256))),
        BuiltIn::YulCreate2 => YulValue::new(mlir_op!(
            state,
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
            mlir_op_void!(state, &current, LogOperation.addr(arguments[0]).size(arguments[1]).topics(topics.as_slice()));
            arguments[0]
        }

        BuiltIn::YulReturn => {
            mlir_op_void!(state, &current, ReturnOperation.addr(arguments[0]).size(arguments[1]));
            arguments[0]
        }
        BuiltIn::YulRevert => {
            mlir_op_void!(state, &current, RevertOperation.addr(arguments[0]).size(arguments[1]));
            arguments[0]
        }
        BuiltIn::YulSelfdestruct => {
            mlir_op_void!(state, &current, SelfDestructOperation.addr(arguments[0]));
            arguments[0]
        }
        BuiltIn::YulStop => {
            mlir_op_void!(state, &current, StopOperation);
            YulValue::constant(&BigInt::from(0u32), state, &current)
        }
        BuiltIn::YulInvalid => {
            mlir_op_void!(state, &current, InvalidOperation);
            YulValue::constant(&BigInt::from(0u32), state, &current)
        }

        BuiltIn::YulPop => arguments[0],

        _ => unimplemented!("unsupported yul intrinsic: {opcode:?}"),
    };
    BlockAnd { value: vec![value], block: current }
});
