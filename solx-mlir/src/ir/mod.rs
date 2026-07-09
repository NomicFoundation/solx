//!
//! Sol dialect IR wrappers: types, values, and attributes.
//!
#![expect(missing_docs, reason = "generated Sol op wrapper")]

pub mod attributes;
pub mod block;
pub mod place;
pub mod r#type;
pub mod value;

use melior::ir::BlockLike;

use crate::Block;
use crate::Place;
use crate::Value;
use crate::ods::sol::*;

sol_ops! {
    Value::constant(value: i64, result_type: ty) -> value {
        ConstantOperation.value(int_attr(value, result_type)).result(result_type)
    }
    Value::string_literal(text: str) -> value {
        StringLitOperation.value(str_attr(text)).addr(memory())
    }
    Value::array_literal(elements: values, array_type: ty) -> value {
        ArrayLitOperation.ins(many(elements)).addr(array_type)
    }
    Value::cast(self, target_type: ty) -> value nop_if_same(target_type) {
        CastOperation.inp(self).out(target_type)
    }
    Value::bytes_cast(self, target_type: ty) -> value nop_if_same(target_type) {
        BytesCastOperation.inp(self).out(target_type)
    }
    Value::address_cast(self, target_type: ty) -> value {
        AddressCastOperation.inp(self).out(target_type)
    }
    Value::fixed_bytes_index(self, index: value) -> value {
        FixedBytesIndexOperation.value(self).index(index).result(fixed_bytes(1))
    }
    Value::push(self, slot_type: ty) -> value {
        PushOperation.inp(self).addr(slot_type)
    }
    Value::pop(self) {
        PopOperation.inp(self)
    }
    Value::length(self) -> value {
        LengthOperation.inp(self).len(field())
    }
    Value::compare(self, other: value, predicate: predicate) -> value {
        CmpOperation.predicate(predicate_attr(predicate)).lhs(self).rhs(other).result(boolean())
    }

    Value::add(self, rhs: value) -> value {
        checked(CAddOperation, AddOperation).lhs(self).rhs(rhs)
    }
    Value::subtract(self, rhs: value) -> value {
        checked(CSubOperation, SubOperation).lhs(self).rhs(rhs)
    }
    Value::multiply(self, rhs: value) -> value {
        checked(CMulOperation, MulOperation).lhs(self).rhs(rhs)
    }
    Value::divide(self, rhs: value) -> value {
        checked(CDivOperation, DivOperation).lhs(self).rhs(rhs)
    }
    Value::remainder(self, rhs: value) -> value {
        ModOperation.lhs(self).rhs(rhs)
    }
    Value::exponentiate(self, rhs: value) -> value {
        checked(CExpOperation, ExpOperation).lhs(self).rhs(rhs).result(self_ty)
    }
    Value::bitand(self, rhs: value) -> value {
        AndOperation.lhs(self).rhs(rhs)
    }
    Value::bitor(self, rhs: value) -> value {
        OrOperation.lhs(self).rhs(rhs)
    }
    Value::bitxor(self, rhs: value) -> value {
        XorOperation.lhs(self).rhs(rhs)
    }
    Value::shl(self, rhs: value) -> value {
        ShlOperation.lhs(self).rhs(rhs).result(self_ty)
    }
    Value::shr(self, rhs: value) -> value {
        ShrOperation.lhs(self).rhs(rhs).result(self_ty)
    }
    Value::not(self) -> value {
        NotOperation.value(self)
    }
    Value::addmod(x: value, y: value, modulus: value) -> value {
        AddModOperation.x(x).y(y).r#mod(modulus)
    }
    Value::mulmod(x: value, y: value, modulus: value) -> value {
        MulModOperation.x(x).y(y).r#mod(modulus)
    }

    Value::encode(inputs: values, selector: optional_value, packed: bool) -> value {
        EncodeOperation.ins(many(inputs)).res(memory()).selector(optional_value(selector)).packed(flag(packed))
    }
    Value::decode(payload: value, result_type: ty) -> value {
        DecodeOperation.addr(payload).outs(single(result_type))
    }

    Value::keccak256(data: value) -> value {
        Keccak256Operation.addr(data).result(fixed_bytes(32))
    }
    Value::ecrecover(hash: value, v: value, r: value, s: value) -> value {
        EcrecoverOperation.hash(hash).v(v).r(r).s(s).result(address())
    }
    Value::sha256(data: value) -> value {
        Sha256Operation.data(data).result(fixed_bytes(32))
    }
    Value::ripemd160(data: value) -> value {
        Ripemd160Operation.data(data).result(fixed_bytes(20))
    }

    Value::balance(address: value) -> value {
        BalanceOperation.cont_addr(address).out(field())
    }
    Value::code_hash(address: value) -> value {
        CodeHashOperation.cont_addr(address).out(field())
    }
    Value::code(address: value) -> value {
        CodeOperation.cont_addr(address).out(memory())
    }
    Value::send(address: value, amount: value) -> value {
        SendOperation.addr(address).val(amount).status(boolean())
    }
    Value::transfer(address: value, amount: value) {
        TransferOperation.addr(address).val(amount)
    }

    Value::block_number() -> value { BlockNumberOperation.val(field()) }
    Value::block_timestamp() -> value { TimestampOperation.val(field()) }
    Value::block_coinbase() -> value { CoinbaseOperation.addr(address()) }
    Value::block_difficulty() -> value { DifficultyOperation.val(field()) }
    Value::block_prev_randao() -> value { PrevRandaoOperation.val(field()) }
    Value::block_gas_limit() -> value { GasLimitOperation.val(field()) }
    Value::block_base_fee() -> value { BaseFeeOperation.val(field()) }
    Value::block_blob_base_fee() -> value { BlobBaseFeeOperation.val(field()) }
    Value::block_chain_id() -> value { ChainIdOperation.val(field()) }
    Value::tx_origin() -> value { OriginOperation.addr(address()) }
    Value::tx_gas_price() -> value { GasPriceOperation.val(field()) }
    Value::msg_sender() -> value { CallerOperation.addr(address()) }
    Value::msg_value() -> value { CallValueOperation.val(field()) }
    Value::msg_sig() -> value { SigOperation.val(fixed_bytes(4)) }
    Value::msg_data() -> value { GetCallDataOperation.addr(calldata()) }
    Value::gas_left() -> value { GasLeftOperation.val(field()) }
    Value::this(contract_type: ty) -> value { ThisOperation.addr(contract_type) }

    Place::stack(pointee: ty) -> place {
        AllocaOperation.alloc_type(ty_attr(ptr(pointee, stack))).addr(ptr(pointee, stack))
    }
    Place::malloc(pointee: ty) -> place {
        MallocOperation.addr(pointee)
    }
    Place::addr_of(symbol: str, place_type: ty) -> place {
        AddrOfOperation.var(symbol_attr(symbol)).addr(place_type)
    }
    Place::load(self, result_type: ty) -> value nop_if_same(result_type) {
        LoadOperation.addr(self).out(result_type)
    }
    Place::store(self, value: value) {
        StoreOperation.val(value).addr(self)
    }
    Place::copy_from(self, value: value) {
        CopyOperation.src(value).dst(self)
    }
    Place::gep(self, index: value, element_type: ty) -> place {
        GepOperation.base_addr(self).idx(index).addr(gep_of(element_type))
    }
    Place::map(self, key: value, entry_type: ty) -> place {
        MapOperation.mapping(self).key(key).addr(entry_type)
    }

    Block::emit(self, signature: optional_str, indexed: values, non_indexed: values) {
        EmitOperation.args(concat(indexed, non_indexed)).indexed_args_count(count_attr(indexed)).signature(optional_str(signature))
    }
    Block::require(self, condition: value, arguments: values, message: optional_str, custom: bool) {
        RequireOperation.cond(condition).args(many(arguments)).msg(optional_str(message)).call(flag(custom))
    }
    Block::assert(self, condition: value) {
        AssertOperation.cond(condition)
    }
    Block::revert(self, signature: str, arguments: values, custom: bool) {
        RevertOperation.signature(str_attr(signature)).args(many(arguments)).call(flag(custom))
    }
    Block::r#return(self, operands: values) {
        ReturnOperation.operands(many(operands))
    }
    Block::r#break(self) {
        BreakOperation
    }
    Block::r#continue(self) {
        ContinueOperation
    }
    Block::r#yield(self, results: values) {
        YieldOperation.ins(many(results))
    }
    Block::condition(self, condition: value) {
        ConditionOperation.condition(condition)
    }
    Block::branch(self, condition: value, with_else: bool) {
        IfOperation.cond(condition); then_region; else_region if with_else
    }
    Block::for_loop(self) {
        ForOperation; cond, body, step
    }
    Block::while_loop(self) {
        WhileOperation; cond, body
    }
    Block::do_while(self) {
        DoWhileOperation; body, cond
    }
}
