//!
//! Yul dialect IR wrappers: the word, the slot it lives in, and the block it is emitted into.
//!
//! Inline assembly is lowered into a `sol.inline_asm` region holding Yul dialect ops, so the Yul
//! entities mirror the Sol ones: [`Word`] is the value, [`Slot`] the place, [`YulBlock`] the
//! statement receiver. The builtin table below is the Yul half of [`dialect_ops!`], reaching
//! `ods::yul` rather than `ods::sol`.
//!
#![expect(missing_docs, reason = "generated Yul op wrapper")]

pub mod block;
pub mod function;
pub mod predicate;
pub mod reference;
pub mod slot;
pub mod word;

use crate::Slot;
use crate::Word;
use crate::ods::yul::*;

dialect_ops! {
    Word::constant(value: u256) -> word {
        ConstantOperation.value(word_attr(value)).out(word())
    }
    Word::compare(lhs: word, rhs: word, predicate: yul_predicate) -> word {
        CmpOperation.lhs(lhs).rhs(rhs).predicate(predicate_attr(predicate)).out(word())
    }

    Word::add(lhs: word, rhs: word) -> word { AddOperation.lhs(lhs).rhs(rhs).out(word()) }
    Word::sub(lhs: word, rhs: word) -> word { SubOperation.lhs(lhs).rhs(rhs).out(word()) }
    Word::mul(lhs: word, rhs: word) -> word { MulOperation.lhs(lhs).rhs(rhs).out(word()) }
    Word::and(lhs: word, rhs: word) -> word { AndOperation.lhs(lhs).rhs(rhs).out(word()) }
    Word::or(lhs: word, rhs: word) -> word { OrOperation.lhs(lhs).rhs(rhs).out(word()) }
    Word::xor(lhs: word, rhs: word) -> word { XOrOperation.lhs(lhs).rhs(rhs).out(word()) }
    Word::not(value: word) -> word { NotOperation.value(value).out(word()) }
    Word::div(dividend: word, divisor: word) -> word {
        DivOperation.dividend(dividend).divisor(divisor).out(word())
    }
    Word::sdiv(dividend: word, divisor: word) -> word {
        SDivOperation.dividend(dividend).divisor(divisor).out(word())
    }
    Word::r#mod(value: word, modulus: word) -> word {
        ModOperation.value(value).r#mod(modulus).out(word())
    }
    Word::smod(value: word, modulus: word) -> word {
        SModOperation.value(value).r#mod(modulus).out(word())
    }
    Word::addmod(x: word, y: word, modulus: word) -> word {
        AddModOperation.x(x).y(y).r#mod(modulus).out(word())
    }
    Word::mulmod(x: word, y: word, modulus: word) -> word {
        MulModOperation.x(x).y(y).r#mod(modulus).out(word())
    }
    Word::exp(base: word, exponent: word) -> word {
        ExpOperation.base(base).exp(exponent).out(word())
    }
    Word::shl(shift: word, value: word) -> word {
        ShlOperation.shift(shift).val(value).out(word())
    }
    Word::shr(shift: word, value: word) -> word {
        ShrOperation.shift(shift).val(value).out(word())
    }
    Word::sar(shift: word, value: word) -> word {
        SarOperation.shift(shift).val(value).out(word())
    }
    Word::clz(value: word) -> word { ClzOperation.val(value).out(word()) }
    Word::byte(index: word, value: word) -> word {
        ByteOperation.idx(index).val(value).out(word())
    }
    // `Yul_SignExtendOp` names its operands the other way round: `$val` takes the byte index and
    // `$off` the value, which is the order `evm.signextend` reads them back in.
    Word::signextend(index: word, value: word) -> word {
        SignExtendOperation.val(index).off(value).out(word())
    }

    Word::mload(address: word) -> word { MLoadOperation.addr(address).out(word()) }
    Word::mstore(address: word, value: word) { MStoreOperation.addr(address).val(value) }
    Word::mstore8(address: word, value: word) { MStore8Operation.addr(address).val(value) }
    Word::mcopy(destination: word, source: word, size: word) {
        MCopyOperation.dst(destination).src(source).size(size)
    }
    Word::msize() -> word { MSizeOperation.out(word()) }
    Word::keccak256(address: word, size: word) -> word {
        Keccak256Operation.addr(address).size(size).out(word())
    }

    Word::sload(slot: word) -> word { SLoadOperation.addr(slot).out(word()) }
    Word::sstore(slot: word, value: word) { SStoreOperation.addr(slot).val(value) }
    Word::tload(slot: word) -> word { TLoadOperation.addr(slot).out(word()) }
    Word::tstore(slot: word, value: word) { TStoreOperation.addr(slot).val(value) }

    Word::calldataload(address: word) -> word { CallDataLoadOperation.addr(address).out(word()) }
    Word::calldatasize() -> word { CallDataSizeOperation.out(word()) }
    Word::calldatacopy(destination: word, source: word, size: word) {
        CallDataCopyOperation.dst(destination).src(source).size(size)
    }
    Word::returndatasize() -> word { ReturnDataSizeOperation.out(word()) }
    Word::returndatacopy(destination: word, source: word, size: word) {
        ReturnDataCopyOperation.dst(destination).src(source).size(size)
    }
    Word::codesize() -> word { CodeSizeOperation.out(word()) }
    Word::codecopy(destination: word, source: word, size: word) {
        CodeCopyOperation.dst(destination).src(source).size(size)
    }
    Word::extcodesize(address: word) -> word { ExtCodeSizeOperation.addr(address).out(word()) }
    Word::extcodehash(address: word) -> word { ExtCodeHashOperation.addr(address).out(word()) }
    Word::extcodecopy(address: word, destination: word, source: word, size: word) {
        ExtCodeCopyOperation.addr(address).dst(destination).src(source).size(size)
    }

    Word::create(value: word, address: word, size: word) -> word {
        CreateOperation.val(value).addr(address).size(size).out(word())
    }
    Word::create2(value: word, address: word, size: word, salt: word) -> word {
        Create2Operation.val(value).addr(address).size(size).salt(salt).out(word())
    }
    Word::call(
        gas: word, address: word, value: word,
        input_offset: word, input_size: word, output_offset: word, output_size: word,
    ) -> word {
        CallOperation.gas(gas).address(address).value(value)
            .inp_offset(input_offset).inp_size(input_size)
            .out_offset(output_offset).out_size(output_size).status(word())
    }
    Word::callcode(
        gas: word, address: word, value: word,
        input_offset: word, input_size: word, output_offset: word, output_size: word,
    ) -> word {
        CallCodeOperation.gas(gas).address(address).value(value)
            .inp_offset(input_offset).inp_size(input_size)
            .out_offset(output_offset).out_size(output_size).status(word())
    }
    Word::staticcall(
        gas: word, address: word,
        input_offset: word, input_size: word, output_offset: word, output_size: word,
    ) -> word {
        StaticCallOperation.gas(gas).address(address)
            .inp_offset(input_offset).inp_size(input_size)
            .out_offset(output_offset).out_size(output_size).out(word())
    }
    Word::delegatecall(
        gas: word, address: word,
        input_offset: word, input_size: word, output_offset: word, output_size: word,
    ) -> word {
        DelegateCallOperation.gas(gas).address(address)
            .inp_offset(input_offset).inp_size(input_size)
            .out_offset(output_offset).out_size(output_size).out(word())
    }

    Word::r#return(address: word, size: word) { ReturnOperation.addr(address).size(size) }
    Word::revert(address: word, size: word) { RevertOperation.addr(address).size(size) }
    Word::stop() { StopOperation }
    Word::invalid() { InvalidOperation }
    Word::selfdestruct(recipient: word) { SelfDestructOperation.addr(recipient) }
    Word::log(address: word, size: word, topics: words) {
        LogOperation.addr(address).size(size).topics(many(topics))
    }

    Word::address() -> word { AddressOperation.out(word()) }
    Word::balance(address: word) -> word { BalanceOperation.addr(address).out(word()) }
    Word::selfbalance() -> word { SelfBalanceOperation.out(word()) }
    Word::caller() -> word { CallerOperation.out(word()) }
    Word::callvalue() -> word { CallValOperation.out(word()) }
    Word::gas() -> word { GasOperation.out(word()) }
    Word::gasprice() -> word { GasPriceOperation.out(word()) }
    Word::gaslimit() -> word { GasLimitOperation.out(word()) }
    Word::origin() -> word { OriginOperation.out(word()) }
    Word::chainid() -> word { ChainIdOperation.out(word()) }
    Word::basefee() -> word { BaseFeeOperation.out(word()) }
    Word::blobbasefee() -> word { BlobBaseFeeOperation.out(word()) }
    Word::coinbase() -> word { CoinBaseOperation.out(word()) }
    Word::timestamp() -> word { TimeStampOperation.out(word()) }
    Word::number() -> word { NumberOperation.out(word()) }
    Word::prevrandao() -> word { PrevrandaoOperation.out(word()) }
    Word::blockhash(block: word) -> word { BlockHashOperation.block(block).out(word()) }
    Word::blobhash(index: word) -> word { BlobHashOperation.idx(index).out(word()) }

    Word::call_function(callee: str, operands: words, result_types: types) -> words {
        FuncCallOperation.callee(symbol_attr(callee)).operands(many(operands))
            .outs(many(result_types))
    }

    Slot::alloca() -> slot { AllocaOperation.out(yul_ptr()) }
    Slot::load(self) -> word { LoadOperation.ptr(self).out(word()) }
    Slot::store(self, value: word) { StoreOperation.val(value).ptr(self) }
}
