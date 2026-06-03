//!
//! Yul intrinsic (EVM opcode) lowering.
//!

use super::*;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    // A flat per-opcode dispatch: one `match name` arm per Yul/EVM intrinsic,
    // each a thin op emission (mostly via the `binop!` / `ctx_intrinsic!`
    // macros). The line count and cognitive complexity are inherent to the
    // opcode count, not nested logic, so both are allowed rather than split.
    #[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
    pub(super) fn emit_yul_intrinsic(
        &self,
        name: &str,
        arguments: &[Value<'context, 'block>],
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let builder = &self.state.builder;
        let ctx = builder.context;
        let loc = builder.unknown_location;
        let ui256 = builder.types.ui256;
        let i256_signless: Type<'context> = IntegerType::new(ctx, 256).into();
        // Cast a single argument from ui256 to signless i256 (the type yul
        // dialect ops require).
        let to_signless = |value: Value<'context, 'block>,
                           block: &BlockRef<'context, 'block>|
         -> Value<'context, 'block> {
            if value.r#type() == i256_signless {
                value
            } else {
                builder.emit_sol_cast(value, i256_signless, block)
            }
        };
        let from_signless = |value: Value<'context, 'block>,
                             block: &BlockRef<'context, 'block>|
         -> Value<'context, 'block> {
            if value.r#type() == ui256 {
                value
            } else {
                builder.emit_sol_cast(value, ui256, block)
            }
        };
        macro_rules! binop {
            ($op:ident) => {{
                if arguments.len() != 2 {
                    unreachable!("yul {} needs 2 args", name);
                }
                let value = block
                    .append_operation(
                        $op::builder(ctx, loc)
                            .lhs(arguments[0])
                            .rhs(arguments[1])
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul binop produces one result")
                    .into();
                Ok((value, block))
            }};
        }
        macro_rules! ctx_intrinsic {
            ($op:ident, $setter:ident, $type:expr) => {{
                let value = block
                    .append_operation($op::builder(ctx, loc).$setter($type).build().into())
                    .result(0)
                    .expect("yul intrinsic produces one result")
                    .into();
                Ok((value, block))
            }};
        }
        match name {
            "add" => binop!(AddOperation),
            "sub" => binop!(SubOperation),
            "mul" => binop!(MulOperation),
            "div" => {
                // Yul `div(x, 0)` returns 0 (no revert), unlike `sol.div`.
                if arguments.len() != 2 {
                    unreachable!("yul div needs 2 args");
                }
                let dividend = to_signless(arguments[0], &block);
                let divisor = to_signless(arguments[1], &block);
                let value = block
                    .append_operation(
                        YulDivOp::builder(ctx, loc)
                            .dividend(dividend)
                            .divisor(divisor)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul div produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "mod" => {
                // Yul `mod(x, 0)` returns 0 (no revert), unlike `sol.mod`.
                if arguments.len() != 2 {
                    unreachable!("yul mod needs 2 args");
                }
                let value_arg = to_signless(arguments[0], &block);
                let mod_arg = to_signless(arguments[1], &block);
                let value = block
                    .append_operation(
                        YulModOp::builder(ctx, loc)
                            .value(value_arg)
                            .r#mod(mod_arg)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul mod produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "and" => binop!(AndOperation),
            "or" => binop!(OrOperation),
            "xor" => binop!(XorOperation),
            "shl" => {
                if arguments.len() != 2 {
                    unreachable!("yul shl needs 2 args");
                }
                let value = block
                    .append_operation(
                        ShlOperation::builder(ctx, loc)
                            .lhs(arguments[1])
                            .rhs(arguments[0])
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul shl produces one result")
                    .into();
                Ok((value, block))
            }
            "shr" => {
                if arguments.len() != 2 {
                    unreachable!("yul shr needs 2 args");
                }
                let value = block
                    .append_operation(
                        ShrOperation::builder(ctx, loc)
                            .lhs(arguments[1])
                            .rhs(arguments[0])
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul shr produces one result")
                    .into();
                Ok((value, block))
            }
            "exp" => {
                if arguments.len() != 2 {
                    unreachable!("yul exp needs 2 args");
                }
                let value = block
                    .append_operation(
                        ExpOperation::builder(ctx, loc)
                            .result(ui256)
                            .lhs(arguments[0])
                            .rhs(arguments[1])
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul exp produces one result")
                    .into();
                Ok((value, block))
            }
            "lt" => {
                let cmp = builder.emit_sol_cmp(arguments[0], arguments[1], CmpPredicate::Lt, &block);
                let value = builder.emit_sol_cast(cmp, ui256, &block);
                Ok((value, block))
            }
            "gt" => {
                let cmp = builder.emit_sol_cmp(arguments[0], arguments[1], CmpPredicate::Gt, &block);
                let value = builder.emit_sol_cast(cmp, ui256, &block);
                Ok((value, block))
            }
            "eq" => {
                let cmp = builder.emit_sol_cmp(arguments[0], arguments[1], CmpPredicate::Eq, &block);
                let value = builder.emit_sol_cast(cmp, ui256, &block);
                Ok((value, block))
            }
            "slt" => {
                let si256 = Type::from(IntegerType::signed(ctx, 256));
                let lhs = builder.emit_sol_cast(arguments[0], si256, &block);
                let rhs = builder.emit_sol_cast(arguments[1], si256, &block);
                let cmp = builder.emit_sol_cmp(lhs, rhs, CmpPredicate::Lt, &block);
                let value = builder.emit_sol_cast(cmp, ui256, &block);
                Ok((value, block))
            }
            "sgt" => {
                let si256 = Type::from(IntegerType::signed(ctx, 256));
                let lhs = builder.emit_sol_cast(arguments[0], si256, &block);
                let rhs = builder.emit_sol_cast(arguments[1], si256, &block);
                let cmp = builder.emit_sol_cmp(lhs, rhs, CmpPredicate::Gt, &block);
                let value = builder.emit_sol_cast(cmp, ui256, &block);
                Ok((value, block))
            }
            "iszero" => {
                if arguments.len() != 1 {
                    unreachable!("yul iszero needs 1 arg");
                }
                let zero = builder.emit_sol_constant(0, ui256, &block);
                let cmp = builder.emit_sol_cmp(arguments[0], zero, CmpPredicate::Eq, &block);
                let value = builder.emit_sol_cast(cmp, ui256, &block);
                Ok((value, block))
            }
            "caller" => {
                let addr = block
                    .append_operation(
                        CallerOperation::builder(ctx, loc)
                            .addr(builder.types.sol_address)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul caller produces one result")
                    .into();
                let ui160 = builder.types.ui160;
                let value = builder.emit_sol_address_cast(addr, ui160, &block);
                Ok((builder.emit_sol_cast(value, ui256, &block), block))
            }
            "callvalue" => ctx_intrinsic!(CallValueOperation, val, ui256),
            "gasprice" => ctx_intrinsic!(GasPriceOperation, val, ui256),
            "origin" => {
                let addr = block
                    .append_operation(
                        OriginOperation::builder(ctx, loc)
                            .addr(builder.types.sol_address)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul origin produces one result")
                    .into();
                let ui160 = builder.types.ui160;
                let value = builder.emit_sol_address_cast(addr, ui160, &block);
                Ok((builder.emit_sol_cast(value, ui256, &block), block))
            }
            "coinbase" => {
                let addr = block
                    .append_operation(
                        CoinbaseOperation::builder(ctx, loc)
                            .addr(builder.types.sol_address)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul coinbase produces one result")
                    .into();
                let ui160 = builder.types.ui160;
                let value = builder.emit_sol_address_cast(addr, ui160, &block);
                Ok((builder.emit_sol_cast(value, ui256, &block), block))
            }
            "timestamp" => ctx_intrinsic!(TimestampOperation, val, ui256),
            "number" => ctx_intrinsic!(BlockNumberOperation, val, ui256),
            "difficulty" => ctx_intrinsic!(DifficultyOperation, val, ui256),
            "prevrandao" => ctx_intrinsic!(PrevRandaoOperation, val, ui256),
            "chainid" => ctx_intrinsic!(ChainIdOperation, val, ui256),
            "basefee" => ctx_intrinsic!(BaseFeeOperation, val, ui256),
            "gaslimit" => ctx_intrinsic!(GasLimitOperation, val, ui256),
            "gas" => ctx_intrinsic!(GasLeftOperation, val, ui256),
            "blockhash" => {
                if arguments.len() != 1 {
                    unreachable!("yul blockhash needs 1 arg");
                }
                let value = block
                    .append_operation(
                        BlockHashOperation::builder(ctx, loc)
                            .block_number(arguments[0])
                            .val(builder.types.fixed_bytes(32))
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul blockhash produces one result")
                    .into();
                // fixedbytes<32> → ui256 requires bytes_cast, not sol.cast.
                let cast = builder.emit_sol_bytes_cast(value, ui256, &block);
                Ok((cast, block))
            }
            "mload" => {
                if arguments.len() != 1 {
                    unreachable!("yul mload needs 1 arg");
                }
                let addr = to_signless(arguments[0], &block);
                let value = block
                    .append_operation(
                        YulMLoadOp::builder(ctx, loc)
                            .addr(addr)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul mload produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "mstore" => {
                if arguments.len() != 2 {
                    unreachable!("yul mstore needs 2 args");
                }
                let addr = to_signless(arguments[0], &block);
                let val = to_signless(arguments[1], &block);
                block.append_operation(
                    YulMStoreOp::builder(ctx, loc)
                        .addr(addr)
                        .val(val)
                        .build()
                        .into(),
                );
                Ok((builder.emit_sol_constant(0, ui256, &block), block))
            }
            "mcopy" => {
                if arguments.len() != 3 {
                    unreachable!("yul mcopy needs 3 args");
                }
                let dst = to_signless(arguments[0], &block);
                let src = to_signless(arguments[1], &block);
                let size = to_signless(arguments[2], &block);
                block.append_operation(
                    YulMCopyOp::builder(ctx, loc)
                        .dst(dst)
                        .src(src)
                        .size(size)
                        .build()
                        .into(),
                );
                Ok((builder.emit_sol_constant(0, ui256, &block), block))
            }
            "sload" => {
                if arguments.len() != 1 {
                    unreachable!("yul sload needs 1 arg");
                }
                let addr = to_signless(arguments[0], &block);
                let value = block
                    .append_operation(
                        YulSLoadOp::builder(ctx, loc)
                            .addr(addr)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul sload produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "sstore" => {
                if arguments.len() != 2 {
                    unreachable!("yul sstore needs 2 args");
                }
                let addr = to_signless(arguments[0], &block);
                let val = to_signless(arguments[1], &block);
                block.append_operation(
                    YulSStoreOp::builder(ctx, loc)
                        .addr(addr)
                        .val(val)
                        .build()
                        .into(),
                );
                Ok((builder.emit_sol_constant(0, ui256, &block), block))
            }
            "tload" => {
                if arguments.len() != 1 {
                    unreachable!("yul tload needs 1 arg");
                }
                let addr = to_signless(arguments[0], &block);
                let value = block
                    .append_operation(
                        YulTLoadOp::builder(ctx, loc)
                            .addr(addr)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul tload produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "tstore" => {
                if arguments.len() != 2 {
                    unreachable!("yul tstore needs 2 args");
                }
                let addr = to_signless(arguments[0], &block);
                let val = to_signless(arguments[1], &block);
                block.append_operation(
                    YulTStoreOp::builder(ctx, loc)
                        .addr(addr)
                        .val(val)
                        .build()
                        .into(),
                );
                Ok((builder.emit_sol_constant(0, ui256, &block), block))
            }
            "keccak256" => {
                if arguments.len() != 2 {
                    unreachable!("yul keccak256 needs 2 args");
                }
                let addr = to_signless(arguments[0], &block);
                let size = to_signless(arguments[1], &block);
                let value = block
                    .append_operation(
                        YulKeccak256Op::builder(ctx, loc)
                            .addr(addr)
                            .size(size)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul keccak256 produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "address" => {
                let value = block
                    .append_operation(
                        YulAddressOp::builder(ctx, loc).out(i256_signless).build().into(),
                    )
                    .result(0)
                    .expect("yul address produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "selfbalance" => {
                let value = block
                    .append_operation(
                        YulSelfBalanceOp::builder(ctx, loc).out(i256_signless).build().into(),
                    )
                    .result(0)
                    .expect("yul selfbalance produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "calldataload" => {
                if arguments.len() != 1 {
                    unreachable!("yul calldataload needs 1 arg");
                }
                let addr = to_signless(arguments[0], &block);
                let value = block
                    .append_operation(
                        YulCallDataLoadOp::builder(ctx, loc)
                            .addr(addr)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul calldataload produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "calldatasize" => {
                let value = block
                    .append_operation(
                        YulCallDataSizeOp::builder(ctx, loc).out(i256_signless).build().into(),
                    )
                    .result(0)
                    .expect("yul calldatasize produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "codesize" => {
                let value = block
                    .append_operation(
                        YulCodeSizeOp::builder(ctx, loc).out(i256_signless).build().into(),
                    )
                    .result(0)
                    .expect("yul codesize produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "returndatasize" => {
                let value = block
                    .append_operation(
                        YulReturnDataSizeOp::builder(ctx, loc)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul returndatasize produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "mstore8" => {
                if arguments.len() != 2 {
                    unreachable!("yul mstore8 needs 2 args");
                }
                let addr = to_signless(arguments[0], &block);
                let val = to_signless(arguments[1], &block);
                block.append_operation(
                    YulMStore8Op::builder(ctx, loc).addr(addr).val(val).build().into(),
                );
                Ok((builder.emit_sol_constant(0, ui256, &block), block))
            }
            "return" => {
                if arguments.len() != 2 {
                    unreachable!("yul return needs 2 args");
                }
                let addr = to_signless(arguments[0], &block);
                let size = to_signless(arguments[1], &block);
                block.append_operation(
                    YulReturnOp::builder(ctx, loc).addr(addr).size(size).build().into(),
                );
                Ok((builder.emit_sol_constant(0, ui256, &block), block))
            }
            "revert" => {
                if arguments.len() != 2 {
                    unreachable!("yul revert needs 2 args");
                }
                let addr = to_signless(arguments[0], &block);
                let size = to_signless(arguments[1], &block);
                block.append_operation(
                    YulRevertOp::builder(ctx, loc).addr(addr).size(size).build().into(),
                );
                Ok((builder.emit_sol_constant(0, ui256, &block), block))
            }
            "stop" => {
                block.append_operation(YulStopOp::builder(ctx, loc).build().into());
                Ok((builder.emit_sol_constant(0, ui256, &block), block))
            }
            "invalid" => {
                block.append_operation(YulInvalidOp::builder(ctx, loc).build().into());
                Ok((builder.emit_sol_constant(0, ui256, &block), block))
            }
            "not" => {
                if arguments.len() != 1 {
                    unreachable!("yul not needs 1 arg");
                }
                let v = to_signless(arguments[0], &block);
                let value = block
                    .append_operation(
                        YulNotOp::builder(ctx, loc).value(v).out(i256_signless).build().into(),
                    )
                    .result(0)
                    .expect("yul not produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "byte" => {
                if arguments.len() != 2 {
                    unreachable!("yul byte needs 2 args");
                }
                let idx = to_signless(arguments[0], &block);
                let val = to_signless(arguments[1], &block);
                let value = block
                    .append_operation(
                        YulByteOp::builder(ctx, loc)
                            .idx(idx)
                            .val(val)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul byte produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "signextend" => {
                if arguments.len() != 2 {
                    unreachable!("yul signextend needs 2 args");
                }
                let val = to_signless(arguments[0], &block);
                let off = to_signless(arguments[1], &block);
                let value = block
                    .append_operation(
                        YulSignExtendOp::builder(ctx, loc)
                            .val(val)
                            .off(off)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul signextend produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "sdiv" => {
                if arguments.len() != 2 {
                    unreachable!("yul sdiv needs 2 args");
                }
                let dividend = to_signless(arguments[0], &block);
                let divisor = to_signless(arguments[1], &block);
                let value = block
                    .append_operation(
                        YulSDivOp::builder(ctx, loc)
                            .dividend(dividend)
                            .divisor(divisor)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul sdiv produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "smod" => {
                if arguments.len() != 2 {
                    unreachable!("yul smod needs 2 args");
                }
                let value_arg = to_signless(arguments[0], &block);
                let mod_arg = to_signless(arguments[1], &block);
                let value = block
                    .append_operation(
                        YulSModOp::builder(ctx, loc)
                            .value(value_arg)
                            .r#mod(mod_arg)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul smod produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "sar" => {
                if arguments.len() != 2 {
                    unreachable!("yul sar needs 2 args");
                }
                let shift = to_signless(arguments[0], &block);
                let val = to_signless(arguments[1], &block);
                let value = block
                    .append_operation(
                        YulSarOp::builder(ctx, loc)
                            .shift(shift)
                            .val(val)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul sar produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "addmod" => {
                if arguments.len() != 3 {
                    unreachable!("yul addmod needs 3 args");
                }
                let x = to_signless(arguments[0], &block);
                let y = to_signless(arguments[1], &block);
                let m = to_signless(arguments[2], &block);
                let value = block
                    .append_operation(
                        YulAddModOp::builder(ctx, loc)
                            .x(x)
                            .y(y)
                            .r#mod(m)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul addmod produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "mulmod" => {
                if arguments.len() != 3 {
                    unreachable!("yul mulmod needs 3 args");
                }
                let x = to_signless(arguments[0], &block);
                let y = to_signless(arguments[1], &block);
                let m = to_signless(arguments[2], &block);
                let value = block
                    .append_operation(
                        YulMulModOp::builder(ctx, loc)
                            .x(x)
                            .y(y)
                            .r#mod(m)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul mulmod produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "balance" => {
                if arguments.len() != 1 {
                    unreachable!("yul balance needs 1 arg");
                }
                let addr = to_signless(arguments[0], &block);
                let value = block
                    .append_operation(
                        YulBalanceOp::builder(ctx, loc)
                            .addr(addr)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul balance produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "extcodesize" => {
                if arguments.len() != 1 {
                    unreachable!("yul extcodesize needs 1 arg");
                }
                let addr = to_signless(arguments[0], &block);
                let value = block
                    .append_operation(
                        YulExtCodeSizeOp::builder(ctx, loc)
                            .addr(addr)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul extcodesize produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "extcodehash" => {
                if arguments.len() != 1 {
                    unreachable!("yul extcodehash needs 1 arg");
                }
                let addr = to_signless(arguments[0], &block);
                let value = block
                    .append_operation(
                        YulExtCodeHashOp::builder(ctx, loc)
                            .addr(addr)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul extcodehash produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "calldatacopy" => {
                if arguments.len() != 3 {
                    unreachable!("yul calldatacopy needs 3 args");
                }
                let dst = to_signless(arguments[0], &block);
                let src = to_signless(arguments[1], &block);
                let size = to_signless(arguments[2], &block);
                block.append_operation(
                    YulCallDataCopyOp::builder(ctx, loc)
                        .dst(dst)
                        .src(src)
                        .size(size)
                        .build()
                        .into(),
                );
                Ok((builder.emit_sol_constant(0, ui256, &block), block))
            }
            "codecopy" => {
                if arguments.len() != 3 {
                    unreachable!("yul codecopy needs 3 args");
                }
                let dst = to_signless(arguments[0], &block);
                let src = to_signless(arguments[1], &block);
                let size = to_signless(arguments[2], &block);
                block.append_operation(
                    YulCodeCopyOp::builder(ctx, loc)
                        .dst(dst)
                        .src(src)
                        .size(size)
                        .build()
                        .into(),
                );
                Ok((builder.emit_sol_constant(0, ui256, &block), block))
            }
            "returndatacopy" => {
                if arguments.len() != 3 {
                    unreachable!("yul returndatacopy needs 3 args");
                }
                let dst = to_signless(arguments[0], &block);
                let src = to_signless(arguments[1], &block);
                let size = to_signless(arguments[2], &block);
                block.append_operation(
                    YulReturnDataCopyOp::builder(ctx, loc)
                        .dst(dst)
                        .src(src)
                        .size(size)
                        .build()
                        .into(),
                );
                Ok((builder.emit_sol_constant(0, ui256, &block), block))
            }
            "call" => {
                if arguments.len() != 7 {
                    unreachable!("yul call needs 7 args");
                }
                let gas = to_signless(arguments[0], &block);
                let address = to_signless(arguments[1], &block);
                let value_arg = to_signless(arguments[2], &block);
                let inp_offset = to_signless(arguments[3], &block);
                let inp_size = to_signless(arguments[4], &block);
                let out_offset = to_signless(arguments[5], &block);
                let out_size = to_signless(arguments[6], &block);
                let value = block
                    .append_operation(
                        YulCallOp::builder(ctx, loc)
                            .gas(gas)
                            .address(address)
                            .value(value_arg)
                            .inp_offset(inp_offset)
                            .inp_size(inp_size)
                            .out_offset(out_offset)
                            .out_size(out_size)
                            .status(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul call produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "staticcall" => {
                if arguments.len() != 6 {
                    unreachable!("yul staticcall needs 6 args");
                }
                let gas = to_signless(arguments[0], &block);
                let address = to_signless(arguments[1], &block);
                let inp_offset = to_signless(arguments[2], &block);
                let inp_size = to_signless(arguments[3], &block);
                let out_offset = to_signless(arguments[4], &block);
                let out_size = to_signless(arguments[5], &block);
                let value = block
                    .append_operation(
                        YulStaticCallOp::builder(ctx, loc)
                            .gas(gas)
                            .address(address)
                            .inp_offset(inp_offset)
                            .inp_size(inp_size)
                            .out_offset(out_offset)
                            .out_size(out_size)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul staticcall produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "delegatecall" => {
                if arguments.len() != 6 {
                    unreachable!("yul delegatecall needs 6 args");
                }
                let gas = to_signless(arguments[0], &block);
                let address = to_signless(arguments[1], &block);
                let inp_offset = to_signless(arguments[2], &block);
                let inp_size = to_signless(arguments[3], &block);
                let out_offset = to_signless(arguments[4], &block);
                let out_size = to_signless(arguments[5], &block);
                let value = block
                    .append_operation(
                        YulDelegateCallOp::builder(ctx, loc)
                            .gas(gas)
                            .address(address)
                            .inp_offset(inp_offset)
                            .inp_size(inp_size)
                            .out_offset(out_offset)
                            .out_size(out_size)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul delegatecall produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "pop" => {
                // Pop the argument off (discard); yul `pop(x)` returns nothing.
                if arguments.len() != 1 {
                    unreachable!("yul pop needs 1 arg");
                }
                Ok((builder.emit_sol_constant(0, ui256, &block), block))
            }
            "memoryguard" => {
                // memoryguard(x) is an optimizer hint that returns its argument.
                if arguments.is_empty() {
                    Ok((builder.emit_sol_constant(0, ui256, &block), block))
                } else {
                    Ok((arguments[0], block))
                }
            }
            "verbatim" => {
                // `verbatim_<n>i_<m>o` injects opaque raw EVM bytecode; returning
                // an argument unchanged would silently drop it. (The suffixed
                // forms also reach the unsupported-intrinsic bail below.)
                unimplemented!("verbatim inline assembly")
            }
            "log0" | "log1" | "log2" | "log3" | "log4" => {
                if arguments.len() < 2 {
                    unreachable!("yul {name} needs at least 2 args");
                }
                let addr = to_signless(arguments[0], &block);
                let size = to_signless(arguments[1], &block);
                let topics: Vec<_> = arguments[2..]
                    .iter()
                    .map(|arg| to_signless(*arg, &block))
                    .collect();
                block.append_operation(
                    YulLogOp::builder(ctx, loc)
                        .addr(addr)
                        .size(size)
                        .topics(&topics)
                        .build()
                        .into(),
                );
                Ok((builder.emit_sol_constant(0, ui256, &block), block))
            }
            "create" => {
                if arguments.len() != 3 {
                    unreachable!("yul create needs 3 args");
                }
                let val = to_signless(arguments[0], &block);
                let addr = to_signless(arguments[1], &block);
                let size = to_signless(arguments[2], &block);
                let value = block
                    .append_operation(
                        YulCreateOp::builder(ctx, loc)
                            .val(val)
                            .addr(addr)
                            .size(size)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul create produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            "create2" => {
                if arguments.len() != 4 {
                    unreachable!("yul create2 needs 4 args");
                }
                let val = to_signless(arguments[0], &block);
                let addr = to_signless(arguments[1], &block);
                let size = to_signless(arguments[2], &block);
                let salt = to_signless(arguments[3], &block);
                let value = block
                    .append_operation(
                        YulCreate2Op::builder(ctx, loc)
                            .val(val)
                            .addr(addr)
                            .size(size)
                            .salt(salt)
                            .out(i256_signless)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul create2 produces one result")
                    .into();
                Ok((from_signless(value, &block), block))
            }
            _ => unimplemented!("yul intrinsic: {name}"),
        }
    }
}
