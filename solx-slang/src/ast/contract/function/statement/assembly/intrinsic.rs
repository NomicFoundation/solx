//!
//! Yul EVM-opcode intrinsic emission.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use num_bigint::BigInt;
use slang_solidity_v2::ast::BuiltIn;

use solx_mlir::YulCmpPredicate;
use solx_mlir::ods::yul;

use crate::ast::Type as AstType;
use crate::ast::contract::function::statement::StatementContext;

impl<'state, 'context, 'block> StatementContext<'state, 'context, 'block> {
    /// Emits a Yul EVM-opcode intrinsic, dispatching on the TYPED
    /// `BuiltIn::Yul*` variant resolved via `resolve_to_built_in()`, never a
    /// `match` on the opcode name as text.
    ///
    /// Every operand and result is the signless `i256` Yul word, so each opcode
    /// is a one-to-one `yul.*` op with no Sol-dialect crossing: the
    /// arithmetic/bitwise/shift ops, the unsigned and signed comparisons
    /// (`yul.cmp`), the block/account context ops, raw memory/storage/transient
    /// access, calldata/code/return-data, the call/create family, `log`,
    /// `keccak256`, and the `return`/`revert`/`stop`/`invalid` effects.
    ///
    /// An effect op with no Yul result (a store, copy, log, or terminator)
    /// yields its first operand so the surrounding statement has a value to
    /// discard; `stop`/`invalid` (no operands) yield a fresh zero word.
    ///
    /// Unsupported opcodes (e.g. `verbatim`, `msize`, `blobhash`) are a loud
    /// residual.
    pub fn emit_yul_intrinsic(
        &self,
        opcode: BuiltIn,
        arguments: &[Value<'context, 'block>],
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let builder = &self.state.builder;
        let context = builder.context;
        let loc = builder.unknown_location;
        let i256 = AstType::signless(builder.context, solx_utils::BIT_LENGTH_FIELD).into_mlir();

        // A Yul value op: build the op, set its single `i256` result, return it.
        macro_rules! yul_value {
            ($operation:ty $(, $setter:ident = $argument:expr)* $(,)?) => {{
                let value: Value<'context, 'block> = block
                    .append_operation(
                        <$operation>::builder(context, loc)$(.$setter($argument))*.out(i256).build().into(),
                    )
                    .result(0)
                    .expect("yul value op produces one result")
                    .into();
                (value, block)
            }};
        }
        // A Yul effect op (no result): build it, then yield the first operand as
        // the discarded statement value.
        macro_rules! yul_effect {
            ($operation:ty $(, $setter:ident = $argument:expr)* $(,)?) => {{
                block.append_operation(<$operation>::builder(context, loc)$(.$setter($argument))*.build().into());
                (arguments[0], block)
            }};
        }
        // A no-operand Yul context op producing one `i256` word.
        macro_rules! yul_context {
            ($operation:ty) => {{
                let value: Value<'context, 'block> = block
                    .append_operation(<$operation>::builder(context, loc).out(i256).build().into())
                    .result(0)
                    .expect("yul context op produces one result")
                    .into();
                (value, block)
            }};
        }
        // A comparison `yul.cmp <predicate>, lhs, rhs`.
        let cmp = |predicate: YulCmpPredicate,
                   lhs: Value<'context, 'block>,
                   rhs: Value<'context, 'block>| {
            (builder.emit_yul_cmp(predicate, lhs, rhs, &block), block)
        };

        match opcode {
            BuiltIn::YulAdd => {
                yul_value!(yul::AddOperation, lhs = arguments[0], rhs = arguments[1])
            }
            BuiltIn::YulSub => {
                yul_value!(yul::SubOperation, lhs = arguments[0], rhs = arguments[1])
            }
            BuiltIn::YulMul => {
                yul_value!(yul::MulOperation, lhs = arguments[0], rhs = arguments[1])
            }
            BuiltIn::YulDiv => {
                yul_value!(
                    yul::DivOperation,
                    dividend = arguments[0],
                    divisor = arguments[1]
                )
            }
            BuiltIn::YulSdiv => {
                yul_value!(
                    yul::SDivOperation,
                    dividend = arguments[0],
                    divisor = arguments[1]
                )
            }
            BuiltIn::YulMod => {
                yul_value!(
                    yul::ModOperation,
                    value = arguments[0],
                    r#mod = arguments[1]
                )
            }
            BuiltIn::YulSmod => {
                yul_value!(
                    yul::SModOperation,
                    value = arguments[0],
                    r#mod = arguments[1]
                )
            }
            BuiltIn::YulExp => {
                yul_value!(yul::ExpOperation, base = arguments[0], exp = arguments[1])
            }
            BuiltIn::YulAddmod => {
                yul_value!(
                    yul::AddModOperation,
                    x = arguments[0],
                    y = arguments[1],
                    r#mod = arguments[2]
                )
            }
            BuiltIn::YulMulmod => {
                yul_value!(
                    yul::MulModOperation,
                    x = arguments[0],
                    y = arguments[1],
                    r#mod = arguments[2]
                )
            }
            BuiltIn::YulAnd => {
                yul_value!(yul::AndOperation, lhs = arguments[0], rhs = arguments[1])
            }
            BuiltIn::YulOr => yul_value!(yul::OrOperation, lhs = arguments[0], rhs = arguments[1]),
            BuiltIn::YulXor => {
                yul_value!(yul::XOrOperation, lhs = arguments[0], rhs = arguments[1])
            }
            BuiltIn::YulNot => yul_value!(yul::NotOperation, value = arguments[0]),
            // Yul `shl(shift, value)` / `shr` / `sar` shift the SECOND operand by
            // the first; the op operands are `(shift, val)` in that order.
            BuiltIn::YulShl => {
                yul_value!(yul::ShlOperation, shift = arguments[0], val = arguments[1])
            }
            BuiltIn::YulShr => {
                yul_value!(yul::ShrOperation, shift = arguments[0], val = arguments[1])
            }
            BuiltIn::YulSar => {
                yul_value!(yul::SarOperation, shift = arguments[0], val = arguments[1])
            }
            BuiltIn::YulByte => {
                yul_value!(yul::ByteOperation, idx = arguments[0], val = arguments[1])
            }
            BuiltIn::YulSignextend => {
                yul_value!(
                    yul::SignExtendOperation,
                    val = arguments[0],
                    off = arguments[1]
                )
            }

            BuiltIn::YulLt => cmp(YulCmpPredicate::Ult, arguments[0], arguments[1]),
            BuiltIn::YulGt => cmp(YulCmpPredicate::Ugt, arguments[0], arguments[1]),
            BuiltIn::YulEq => cmp(YulCmpPredicate::Eq, arguments[0], arguments[1]),
            BuiltIn::YulSlt => cmp(YulCmpPredicate::Slt, arguments[0], arguments[1]),
            BuiltIn::YulSgt => cmp(YulCmpPredicate::Sgt, arguments[0], arguments[1]),
            BuiltIn::YulIszero => {
                let zero = builder.emit_yul_constant(&BigInt::from(0u32), &block);
                cmp(YulCmpPredicate::Eq, arguments[0], zero)
            }

            BuiltIn::YulCaller => yul_context!(yul::CallerOperation),
            BuiltIn::YulOrigin => yul_context!(yul::OriginOperation),
            BuiltIn::YulCoinbase => yul_context!(yul::CoinBaseOperation),
            BuiltIn::YulCallvalue => yul_context!(yul::CallValOperation),
            BuiltIn::YulGasprice => yul_context!(yul::GasPriceOperation),
            BuiltIn::YulTimestamp => yul_context!(yul::TimeStampOperation),
            BuiltIn::YulNumber => yul_context!(yul::NumberOperation),
            // Pre-merge `difficulty` and post-merge `prevrandao` are EVM opcode
            // 0x44; the Yul dialect exposes it as `prevrandao`.
            BuiltIn::YulDifficulty | BuiltIn::YulPrevrandao => {
                yul_context!(yul::PrevrandaoOperation)
            }
            BuiltIn::YulChainid => yul_context!(yul::ChainIdOperation),
            BuiltIn::YulBasefee => yul_context!(yul::BaseFeeOperation),
            BuiltIn::YulGaslimit => yul_context!(yul::GasLimitOperation),
            BuiltIn::YulGas => yul_context!(yul::GasOperation),
            BuiltIn::YulBlockhash => yul_value!(yul::BlockHashOperation, block = arguments[0]),

            BuiltIn::YulBalance => yul_value!(yul::BalanceOperation, addr = arguments[0]),
            BuiltIn::YulExtcodehash => yul_value!(yul::ExtCodeHashOperation, addr = arguments[0]),
            BuiltIn::YulExtcodesize => yul_value!(yul::ExtCodeSizeOperation, addr = arguments[0]),
            BuiltIn::YulAddress => yul_context!(yul::AddressOperation),
            BuiltIn::YulSelfbalance => yul_context!(yul::SelfBalanceOperation),

            BuiltIn::YulMload => yul_value!(yul::MLoadOperation, addr = arguments[0]),
            BuiltIn::YulMstore => {
                yul_effect!(
                    yul::MStoreOperation,
                    addr = arguments[0],
                    val = arguments[1]
                )
            }
            BuiltIn::YulMstore8 => {
                yul_effect!(
                    yul::MStore8Operation,
                    addr = arguments[0],
                    val = arguments[1]
                )
            }
            BuiltIn::YulMcopy => {
                yul_effect!(
                    yul::MCopyOperation,
                    dst = arguments[0],
                    src = arguments[1],
                    size = arguments[2]
                )
            }
            BuiltIn::YulSload => yul_value!(yul::SLoadOperation, addr = arguments[0]),
            BuiltIn::YulSstore => {
                yul_effect!(
                    yul::SStoreOperation,
                    addr = arguments[0],
                    val = arguments[1]
                )
            }
            BuiltIn::YulTload => yul_value!(yul::TLoadOperation, addr = arguments[0]),
            BuiltIn::YulTstore => {
                yul_effect!(
                    yul::TStoreOperation,
                    addr = arguments[0],
                    val = arguments[1]
                )
            }

            BuiltIn::YulKeccak256 => {
                yul_value!(
                    yul::Keccak256Operation,
                    addr = arguments[0],
                    size = arguments[1]
                )
            }

            BuiltIn::YulCalldataload => yul_value!(yul::CallDataLoadOperation, addr = arguments[0]),
            BuiltIn::YulCalldatasize => yul_context!(yul::CallDataSizeOperation),
            BuiltIn::YulCalldatacopy => {
                yul_effect!(
                    yul::CallDataCopyOperation,
                    dst = arguments[0],
                    src = arguments[1],
                    size = arguments[2]
                )
            }
            BuiltIn::YulCodesize => yul_context!(yul::CodeSizeOperation),
            BuiltIn::YulCodecopy => {
                yul_effect!(
                    yul::CodeCopyOperation,
                    dst = arguments[0],
                    src = arguments[1],
                    size = arguments[2]
                )
            }
            BuiltIn::YulReturndatasize => yul_context!(yul::ReturnDataSizeOperation),
            BuiltIn::YulReturndatacopy => {
                yul_effect!(
                    yul::ReturnDataCopyOperation,
                    dst = arguments[0],
                    src = arguments[1],
                    size = arguments[2]
                )
            }

            // `yul.call`'s single result is `$status`, not `$out`, so it cannot
            // use the `yul_value!` macro.
            BuiltIn::YulCall => {
                let status: Value<'context, 'block> = block
                    .append_operation(
                        yul::CallOperation::builder(context, loc)
                            .gas(arguments[0])
                            .address(arguments[1])
                            .value(arguments[2])
                            .inp_offset(arguments[3])
                            .inp_size(arguments[4])
                            .out_offset(arguments[5])
                            .out_size(arguments[6])
                            .status(i256)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("yul.call produces one result")
                    .into();
                (status, block)
            }
            BuiltIn::YulStaticcall => yul_value!(
                yul::StaticCallOperation,
                gas = arguments[0],
                address = arguments[1],
                inp_offset = arguments[2],
                inp_size = arguments[3],
                out_offset = arguments[4],
                out_size = arguments[5],
            ),
            BuiltIn::YulDelegatecall => yul_value!(
                yul::DelegateCallOperation,
                gas = arguments[0],
                address = arguments[1],
                inp_offset = arguments[2],
                inp_size = arguments[3],
                out_offset = arguments[4],
                out_size = arguments[5],
            ),

            BuiltIn::YulCreate => {
                yul_value!(
                    yul::CreateOperation,
                    val = arguments[0],
                    addr = arguments[1],
                    size = arguments[2]
                )
            }
            BuiltIn::YulCreate2 => yul_value!(
                yul::Create2Operation,
                val = arguments[0],
                addr = arguments[1],
                size = arguments[2],
                salt = arguments[3],
            ),

            BuiltIn::YulLog => {
                block.append_operation(
                    yul::LogOperation::builder(context, loc)
                        .addr(arguments[0])
                        .size(arguments[1])
                        .topics(&arguments[2..])
                        .build()
                        .into(),
                );
                (arguments[0], block)
            }

            BuiltIn::YulReturn => {
                yul_effect!(
                    yul::ReturnOperation,
                    addr = arguments[0],
                    size = arguments[1]
                )
            }
            BuiltIn::YulRevert => {
                yul_effect!(
                    yul::RevertOperation,
                    addr = arguments[0],
                    size = arguments[1]
                )
            }
            BuiltIn::YulStop => {
                block.append_operation(yul::StopOperation::builder(context, loc).build().into());
                (
                    builder.emit_yul_constant(&BigInt::from(0u32), &block),
                    block,
                )
            }
            BuiltIn::YulInvalid => {
                block.append_operation(yul::InvalidOperation::builder(context, loc).build().into());
                (
                    builder.emit_yul_constant(&BigInt::from(0u32), &block),
                    block,
                )
            }

            // `pop(x)` evaluates and discards; the argument is already emitted.
            BuiltIn::YulPop => (arguments[0], block),

            _ => unimplemented!("unsupported yul intrinsic: {opcode:?}"),
        }
    }
}
