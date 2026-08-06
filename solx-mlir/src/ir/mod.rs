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
use crate::Function;
use crate::Place;
use crate::Value;
use crate::ods::sol::*;

sol_ops! {
    Value::constant(value: i64, result_type: ty) -> value {
        ConstantOperation.value(int_attr(value, result_type)).result(result_type)
    }
    Value::string_literal(text: bytes) -> value {
        StringLitOperation.value(bytes_attr(text)).addr(memory())
    }
    Value::array_literal(elements: values, array_type: ty) -> value {
        ArrayLitOperation.ins(many(elements)).addr(array_type)
    }
    Value::function_constant(symbol: str, result_type: ty) -> value {
        FuncConstantOperation.sym(symbol_attr(symbol)).addr(result_type)
    }
    Value::default_function_constant(result_type: ty) -> value {
        DefaultFuncConstantOperation.addr(result_type)
    }
    Value::external_function_constant(address: value, selector: selector, result_type: ty) -> value {
        ExtFuncConstantOperation.addr(address).selector(selector_attr(selector)).result(result_type)
    }
    Value::external_function_selector(self) -> value {
        ExtFuncSelectorOperation.func(self).result(selector())
    }
    Value::external_function_address(self) -> value {
        ExtFuncAddrOperation.func(self).result(address())
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
    Value::enum_cast(self, target_type: ty) -> value {
        EnumCastOperation.inp(self).out(target_type)
    }
    Value::dyn_bytes_to_fixedbytes(self, target_type: ty) -> value {
        DynBytesToFixedBytesOperation.inp(self).out(target_type)
    }
    Value::data_loc_cast(self, target_type: ty) -> value {
        DataLocCastOperation.inp(self).out(target_type)
    }

    Value::fixed_bytes_index(self, index: value) -> value {
        FixedBytesIndexOperation.value(self).index(index).result(fixed_bytes(solx_utils::BYTE_LENGTH_BYTE))
    }
    Value::length(self) -> value {
        LengthOperation.inp(self).len(field())
    }
    Value::slice(self, start: value, end: value) -> value {
        SliceOperation.arr(self).start(start).end(end).res(self_ty)
    }
    Value::concat(operands: values) -> value {
        ConcatOperation.args(many(operands)).result(memory())
    }

    Value::compare(self, other: value, predicate: predicate) -> value {
        CmpOperation.predicate(predicate_attr(predicate)).lhs(self).rhs(other).result(boolean())
    }

    Value::add(self, rhs: value) -> value checked(CAddOperation) {
        AddOperation.lhs(self).rhs(rhs)
    }
    Value::subtract(self, rhs: value) -> value checked(CSubOperation) {
        SubOperation.lhs(self).rhs(rhs)
    }
    Value::multiply(self, rhs: value) -> value checked(CMulOperation) {
        MulOperation.lhs(self).rhs(rhs)
    }
    Value::divide(self, rhs: value) -> value checked(CDivOperation) {
        DivOperation.lhs(self).rhs(rhs)
    }
    Value::remainder(self, rhs: value) -> value {
        ModOperation.lhs(self).rhs(rhs)
    }
    Value::exponentiate(self, rhs: value) -> value checked(CExpOperation) {
        ExpOperation.lhs(self).rhs(rhs).result(self_ty)
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

    Value::encode | encode_packed (inputs: values, selector: optional_value) -> value {
        EncodeOperation.ins(many(inputs)).res(memory()).selector(optional_value(selector))
    } flagged .packed;
    Value::decode(payload: value, result_types: types) -> values {
        DecodeOperation.addr(payload).outs(many(result_types))
    }

    Value::keccak256(data: value) -> value {
        Keccak256Operation.addr(data).result(fixed_bytes(solx_utils::BYTE_LENGTH_FIELD))
    }
    Value::ecrecover(hash: value, v: value, r: value, s: value) -> value {
        EcrecoverOperation.hash(hash).v(v).r(r).s(s).result(address())
    }
    Value::sha256(data: value) -> value {
        Sha256Operation.data(data).result(fixed_bytes(solx_utils::BYTE_LENGTH_FIELD))
    }
    Value::ripemd160(data: value) -> value {
        Ripemd160Operation.data(data).result(fixed_bytes(solx_utils::BYTE_LENGTH_ETH_ADDRESS))
    }
    Value::blockhash(block_number: value) -> value {
        BlockHashOperation.block_number(block_number).val(fixed_bytes(solx_utils::BYTE_LENGTH_FIELD))
    }
    Value::blobhash(index: value) -> value {
        BlobHashOperation.idx(index).val(fixed_bytes(solx_utils::BYTE_LENGTH_FIELD))
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
    Value::bare_call(address: value, gas: value, amount: value, input: value) -> values {
        BareCallOperation.addr(address).gas(gas).val(amount).inp(input)
            .status(boolean()).ret_data(memory())
    }
    Value::bare_delegate_call(address: value, gas: value, input: value) -> values {
        BareDelegateCallOperation.addr(address).gas(gas).inp(input)
            .status(boolean()).ret_data(memory())
    }
    Value::bare_static_call(address: value, gas: value, input: value) -> values {
        BareStaticCallOperation.addr(address).gas(gas).inp(input)
            .status(boolean()).ret_data(memory())
    }
    Value::send(address: value, amount: value) -> value {
        SendOperation.addr(address).val(amount).status(boolean())
    }
    Value::transfer(address: value, amount: value) {
        TransferOperation.addr(address).val(amount)
    }
    Value::selfdestruct(recipient: value) {
        SelfdestructOperation.recipient(recipient)
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
    Value::msg_sig() -> value { SigOperation.val(selector()) }
    Value::msg_data() -> value { GetCallDataOperation.addr(calldata()) }
    Value::gas_left() -> value { GasLeftOperation.val(field()) }
    Value::this(contract_type: ty) -> value { ThisOperation.addr(contract_type) }

    Function::call(callee: function, operands: values) -> values {
        CallOperation.callee(symbol_attr(&callee.mlir_name))
            .outs(many(callee.function_type.results)).operands(many(operands))
    }
    Function::external_call | external_static_call | external_try_call | external_static_try_call (
        callee: function, arguments: values,
        address: value, selector: value, gas: value, amount: value,
    ) -> status_and_values {
        ExtCallOperation.callee(str_attr(&callee.mlir_name)).ins(many(arguments)).addr(address)
            .gas(gas).val(amount).selector(selector)
            .callee_type(ty_attr(signature(callee)))
            .status(boolean()).outs(many(callee.function_type.results))
    } flagged .static_call, .try_call;
    Value::indirect_call(self, operands: values, result_types: types) -> values {
        ICallOperation.callee(self).outs(many(result_types)).callee_operands(many(operands))
    }
    Value::external_call | external_static_call | external_try_call | external_static_try_call (
        self, operands: values, result_types: types, gas: value, amount: value,
    ) -> status_and_values {
        ExtICallOperation.callee(self).outs(status_and(result_types))
            .callee_operands(many(operands)).gas(gas).value(amount)
    } flagged .static_call, .try_call;
    Value::create_contract | create_contract_try (
        object_name: str, amount: value, salt: optional_value, arguments: values, result_type: ty,
    ) -> value {
        NewOperation.obj_name(str_attr(object_name)).val(amount).salt(optional_value(salt))
            .ctor_args(many(arguments)).out(result_type)
    } flagged .try_call;

    Place::stack(pointee: ty) -> place {
        AllocaOperation.alloc_type(ty_attr(ptr(pointee, stack))).addr(ptr(pointee, stack))
    }
    Place::malloc | malloc_zeroed (pointee: ty, size: optional_value) -> place {
        MallocOperation.size(optional_value(size)).addr(pointee)
    } flagged .zero_init;
    Place::default_storage(place_type: ty) -> place {
        DefaultStorageOperation.result(place_type)
    }
    Place::default_calldata(place_type: ty) -> place {
        DefaultCallDataOperation.result(place_type)
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

    Place::push(self, slot_type: ty) -> value {
        PushOperation.inp(self).addr(slot_type)
    }
    Place::push_string(self, value: value) {
        PushStringOperation.addr(self).value(value)
    }
    Place::pop(self) {
        PopOperation.inp(self)
    }
    Place::delete(self) {
        DeleteOperation.reference(self)
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
    Block::require | require_custom (self, condition: value, arguments: values, message: optional_str) {
        RequireOperation.cond(condition).args(many(arguments)).msg(optional_str(message))
    } flagged .call;
    Block::assert(self, condition: value) {
        AssertOperation.cond(condition)
    }
    Block::revert | revert_custom (self, signature: optional_str, arguments: values) {
        RevertOperation.signature(optional_str(signature)).args(many(arguments))
    } flagged .call;
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
    Block::branch(self, condition: value) {
        IfOperation.cond(condition); then_region ; empty else_region
    }
    Block::branch_with_else(self, condition: value) {
        IfOperation.cond(condition); then_region, else_region
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
