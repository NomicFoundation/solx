//!
//! Yul dialect IR wrappers: the entities an inline-assembly block is emitted through, and the
//! builtin table that produces the dialect's ops.
//!
//! The table is the Yul half of [`dialect_ops!`], reaching `ods::yul` rather than `ods::sol`.
//!
#![expect(missing_docs, reason = "generated Yul op wrapper")]

pub mod block;
pub mod pointer;
pub mod predicate;
pub mod word;

use melior::ir::BlockLike;

use crate::Pointer;
use crate::Word;
use crate::YulBlock;
use crate::YulFunction;
use crate::ods::yul::*;

dialect_ops! {
    Word::constant(value: u256) -> word {
        ConstantOperation.value(word_attr(value)).out(yul_word())
    }
    Word::compare(lhs: word, rhs: word, predicate: yul_predicate) -> word {
        CmpOperation.lhs(lhs).rhs(rhs).predicate(predicate_attr(predicate)).out(yul_word())
    }

    Word::add(lhs: word, rhs: word) -> word { AddOperation.lhs(lhs).rhs(rhs).out(yul_word()) }
    Word::sub(lhs: word, rhs: word) -> word { SubOperation.lhs(lhs).rhs(rhs).out(yul_word()) }
    Word::mul(lhs: word, rhs: word) -> word { MulOperation.lhs(lhs).rhs(rhs).out(yul_word()) }
    Word::and(lhs: word, rhs: word) -> word { AndOperation.lhs(lhs).rhs(rhs).out(yul_word()) }
    Word::or(lhs: word, rhs: word) -> word { OrOperation.lhs(lhs).rhs(rhs).out(yul_word()) }
    Word::xor(lhs: word, rhs: word) -> word { XOrOperation.lhs(lhs).rhs(rhs).out(yul_word()) }
    Word::not(value: word) -> word { NotOperation.value(value).out(yul_word()) }
    Word::div(dividend: word, divisor: word) -> word {
        DivOperation.dividend(dividend).divisor(divisor).out(yul_word())
    }
    Word::sdiv(dividend: word, divisor: word) -> word {
        SDivOperation.dividend(dividend).divisor(divisor).out(yul_word())
    }
    Word::r#mod(value: word, modulus: word) -> word {
        ModOperation.value(value).r#mod(modulus).out(yul_word())
    }
    Word::smod(value: word, modulus: word) -> word {
        SModOperation.value(value).r#mod(modulus).out(yul_word())
    }
    Word::addmod(x: word, y: word, modulus: word) -> word {
        AddModOperation.x(x).y(y).r#mod(modulus).out(yul_word())
    }
    Word::mulmod(x: word, y: word, modulus: word) -> word {
        MulModOperation.x(x).y(y).r#mod(modulus).out(yul_word())
    }
    Word::exp(base: word, exponent: word) -> word {
        ExpOperation.base(base).exp(exponent).out(yul_word())
    }
    Word::shl(shift: word, value: word) -> word {
        ShlOperation.shift(shift).val(value).out(yul_word())
    }
    Word::shr(shift: word, value: word) -> word {
        ShrOperation.shift(shift).val(value).out(yul_word())
    }
    Word::sar(shift: word, value: word) -> word {
        SarOperation.shift(shift).val(value).out(yul_word())
    }
    Word::clz(value: word) -> word { ClzOperation.val(value).out(yul_word()) }
    Word::byte(index: word, value: word) -> word {
        ByteOperation.idx(index).val(value).out(yul_word())
    }
    Word::signextend(index: word, value: word) -> word {
        SignExtendOperation.idx(index).val(value).out(yul_word())
    }

    Word::mload(offset: word) -> word { MLoadOperation.addr(offset).out(yul_word()) }
    Word::mstore(offset: word, value: word) { MStoreOperation.addr(offset).val(value) }
    Word::mstore8(offset: word, value: word) { MStore8Operation.addr(offset).val(value) }
    Word::mcopy(destination: word, source: word, size: word) {
        MCopyOperation.dst(destination).src(source).size(size)
    }
    Word::msize() -> word { MSizeOperation.out(yul_word()) }
    Word::keccak256(offset: word, size: word) -> word {
        Keccak256Operation.addr(offset).size(size).out(yul_word())
    }

    Word::sload(slot: word) -> word { SLoadOperation.addr(slot).out(yul_word()) }
    Word::sstore(slot: word, value: word) { SStoreOperation.addr(slot).val(value) }
    Word::tload(slot: word) -> word { TLoadOperation.addr(slot).out(yul_word()) }
    Word::tstore(slot: word, value: word) { TStoreOperation.addr(slot).val(value) }

    Word::calldataload(offset: word) -> word { CallDataLoadOperation.addr(offset).out(yul_word()) }
    Word::calldatasize() -> word { CallDataSizeOperation.out(yul_word()) }
    Word::calldatacopy(destination: word, source: word, size: word) {
        CallDataCopyOperation.dst(destination).src(source).size(size)
    }
    Word::returndatasize() -> word { ReturnDataSizeOperation.out(yul_word()) }
    Word::returndatacopy(destination: word, source: word, size: word) {
        ReturnDataCopyOperation.dst(destination).src(source).size(size)
    }
    Word::codesize() -> word { CodeSizeOperation.out(yul_word()) }
    Word::codecopy(destination: word, source: word, size: word) {
        CodeCopyOperation.dst(destination).src(source).size(size)
    }
    Word::extcodesize(address: word) -> word { ExtCodeSizeOperation.addr(address).out(yul_word()) }
    Word::extcodehash(address: word) -> word { ExtCodeHashOperation.addr(address).out(yul_word()) }
    Word::extcodecopy(address: word, destination: word, source: word, size: word) {
        ExtCodeCopyOperation.addr(address).dst(destination).src(source).size(size)
    }

    Word::create(value: word, offset: word, size: word) -> word {
        CreateOperation.val(value).addr(offset).size(size).out(yul_word())
    }
    Word::create2(value: word, offset: word, size: word, salt: word) -> word {
        Create2Operation.val(value).addr(offset).size(size).salt(salt).out(yul_word())
    }
    Word::call(
        gas: word, address: word, value: word,
        input_offset: word, input_size: word, output_offset: word, output_size: word,
    ) -> word {
        CallOperation.gas(gas).address(address).value(value)
            .inp_offset(input_offset).inp_size(input_size)
            .out_offset(output_offset).out_size(output_size).status(yul_word())
    }
    Word::callcode(
        gas: word, address: word, value: word,
        input_offset: word, input_size: word, output_offset: word, output_size: word,
    ) -> word {
        CallCodeOperation.gas(gas).address(address).value(value)
            .inp_offset(input_offset).inp_size(input_size)
            .out_offset(output_offset).out_size(output_size).status(yul_word())
    }
    Word::staticcall(
        gas: word, address: word,
        input_offset: word, input_size: word, output_offset: word, output_size: word,
    ) -> word {
        StaticCallOperation.gas(gas).address(address)
            .inp_offset(input_offset).inp_size(input_size)
            .out_offset(output_offset).out_size(output_size).out(yul_word())
    }
    Word::delegatecall(
        gas: word, address: word,
        input_offset: word, input_size: word, output_offset: word, output_size: word,
    ) -> word {
        DelegateCallOperation.gas(gas).address(address)
            .inp_offset(input_offset).inp_size(input_size)
            .out_offset(output_offset).out_size(output_size).out(yul_word())
    }

    Word::r#return(offset: word, size: word) { ReturnOperation.addr(offset).size(size) }
    Word::revert(offset: word, size: word) { RevertOperation.addr(offset).size(size) }
    Word::selfdestruct(recipient: word) { SelfDestructOperation.addr(recipient) }
    Word::log(offset: word, size: word, topics: words) {
        LogOperation.addr(offset).size(size).topics(many(topics))
    }

    Word::address() -> word { AddressOperation.out(yul_word()) }
    Word::balance(address: word) -> word { BalanceOperation.addr(address).out(yul_word()) }
    Word::selfbalance() -> word { SelfBalanceOperation.out(yul_word()) }
    Word::caller() -> word { CallerOperation.out(yul_word()) }
    Word::callvalue() -> word { CallValOperation.out(yul_word()) }
    Word::gas() -> word { GasOperation.out(yul_word()) }
    Word::gasprice() -> word { GasPriceOperation.out(yul_word()) }
    Word::gaslimit() -> word { GasLimitOperation.out(yul_word()) }
    Word::origin() -> word { OriginOperation.out(yul_word()) }
    Word::chainid() -> word { ChainIdOperation.out(yul_word()) }
    Word::basefee() -> word { BaseFeeOperation.out(yul_word()) }
    Word::blobbasefee() -> word { BlobBaseFeeOperation.out(yul_word()) }
    Word::coinbase() -> word { CoinBaseOperation.out(yul_word()) }
    Word::timestamp() -> word { TimeStampOperation.out(yul_word()) }
    Word::number() -> word { NumberOperation.out(yul_word()) }
    Word::prevrandao() -> word { PrevrandaoOperation.out(yul_word()) }
    Word::blockhash(block: word) -> word { BlockHashOperation.block(block).out(yul_word()) }
    Word::blobhash(index: word) -> word { BlobHashOperation.idx(index).out(yul_word()) }

    YulFunction::call(callee: yul_function, operands: words) -> words {
        FuncCallOperation.callee(symbol_attr(&callee.mlir_name)).operands(many(operands))
            .outs(many(callee.function_type.results))
    }

    Pointer::alloca() -> yul_pointer { AllocaOperation.out(yul_ptr()) }
    Pointer::load(self) -> word { LoadOperation.ptr(self).out(yul_word()) }
    Pointer::store(self, value: word) { StoreOperation.val(value).ptr(self) }

    YulBlock::r#break(self) { BreakOperation }
    YulBlock::r#continue(self) { ContinueOperation }
    YulBlock::stop(self) { StopOperation }
    YulBlock::invalid(self) { InvalidOperation }
    YulBlock::r#yield(self) { YieldOperation.operands(empty()) }
    YulBlock::condition(self, condition: word) {
        ConditionOperation.condition(condition).args(empty())
    }
    YulBlock::branch(self, condition: word) {
        IfOperation.results(empty()).cond(condition); then_region ; empty else_region
    }
    YulBlock::for_loop(self) {
        ForOperation.results(empty()).init_args(empty()); cond, body, step
    }
}
