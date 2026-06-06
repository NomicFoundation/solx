//!
//! Public state-variable getter synthesis.
//!
//! Solidity synthesises an external accessor for every `public` state variable.
//! This module carries the per-getter frame ([`GetterAbi`]) and the emission /
//! classification methods that lower it; the dispatching entry points are `impl`
//! blocks on the foreign [`ContractEmitter`] (§2a: the SOLE top-level type here
//! is `GetterAbi`).
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use num_bigint::BigInt;
use ruint::aliases::U256;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::StructDefinition;
use slang_solidity_v2::ast::Type as SlangType;

use solx_utils::DataLocation;

use crate::ast::contract::ContractEmitter;
use crate::ast::contract::getter_level::GetterLevel;

/// The per-getter emission frame: the state variable, its canonical ABI
/// signature and selector, declared type, storage coordinate, and the
/// `sol.contract` body the getter `sol.func` is appended to.
///
/// The SOLE top-level type of this module (§2a).
pub struct GetterAbi<'a, 'context, 'block> {
    /// The state variable whose accessor is being generated.
    state_variable: &'a StateVariableDefinition,
    /// The canonical ABI signature, e.g. `balances(address)`.
    signature: &'a str,
    /// The 4-byte function selector derived from the signature.
    selector: u32,
    /// The variable's declared Solidity type (mapping/array/struct/scalar).
    declared_type: &'a SlangType,
    /// The storage slot holding the variable.
    slot: U256,
    /// The byte offset within the slot (for packed value types).
    byte_offset: u32,
    /// `Storage` vs `Transient` — selects SLOAD/SSTORE vs TLOAD/TSTORE.
    location: DataLocation,
    /// The `sol.contract` body the getter `sol.func` is appended to.
    contract_body: &'a BlockRef<'context, 'block>,
}

impl<'a, 'context, 'block> GetterAbi<'a, 'context, 'block> {
    /// Derives the indexed (mapping/array) getter signature: the MLIR parameter
    /// types, the per-level access plan, and the final result Solidity type.
    ///
    /// OPEN-A (R8-6): this name clashes with the kept `CallEmitter::getter_signature`
    /// built-in classifier; the two must be de-clashed at the getter fill. It is
    /// really a classifier over `indexed_getter_levels`.
    pub fn getter_signature(
        &self,
        declared_type: &SlangType,
        location: DataLocation,
    ) -> Option<(Vec<Type<'context>>, Vec<GetterLevel<'context>>, SlangType)> {
        let _ = (
            self.state_variable,
            self.signature,
            self.selector,
            self.declared_type,
            self.slot,
            self.byte_offset,
            self.location,
            self.contract_body,
            declared_type,
            location,
        );
        unimplemented!("getter signature derivation")
    }

    /// Plans the per-level access chain for an indexed getter, owning the
    /// [`GetterLevel`] construction.
    pub fn indexed_getter_levels(
        &self,
        declared_type: &SlangType,
        location: DataLocation,
    ) -> (Vec<Type<'context>>, Vec<GetterLevel<'context>>, SlangType) {
        let _ = (declared_type, location);
        unimplemented!("indexed getter level planning")
    }

    /// Emits an indexed (mapping/array) getter `sol.func`.
    ///
    /// A4 (#H-M7/#H-M10/M11): arg-bearing nested / reference getters stay a LOUD
    /// `unimplemented!` at the tip.
    pub fn emit_indexed_getter(&self, abi_input_count: usize) -> anyhow::Result<()> {
        let _ = abi_input_count;
        unimplemented!("indexed (mapping/array) getter emission")
    }

    /// Threads the storage access chain for an indexed getter, level by level.
    pub fn emit_getter_access_chain(
        &self,
        base: Value<'context, 'block>,
        levels: &[GetterLevel<'context>],
        entry: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        let _ = (base, levels, entry);
        unimplemented!("getter storage access chain")
    }

    /// Emits the return of an indexed getter (scalar or destructured struct).
    pub fn emit_indexed_getter_result(
        &self,
        base: Value<'context, 'block>,
        struct_plan: &Option<Vec<(u64, Type<'context>, Type<'context>)>>,
        result_type: Type<'context>,
        entry: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let _ = (base, struct_plan, result_type, entry);
        unimplemented!("indexed getter return emission")
    }

    /// Emits a struct getter; returns whether the variable was a struct getter.
    pub fn emit_struct_getter(&self) -> anyhow::Result<bool> {
        unimplemented!("struct getter emission")
    }

    /// Emits a scalar / reference getter.
    pub fn emit_scalar_getter(&self) -> anyhow::Result<()> {
        unimplemented!("scalar/reference getter emission")
    }
}

impl<'state, 'context> ContractEmitter<'state, 'context> {
    /// Dispatches getter synthesis for one state variable to the scalar /
    /// struct / indexed path.
    pub fn emit_state_variable_getter(
        &self,
        state_variable: &StateVariableDefinition,
        slot: U256,
        byte_offset: u32,
        location: DataLocation,
        contract_body: &BlockRef<'context, '_>,
    ) -> anyhow::Result<()> {
        let _ = (state_variable, slot, byte_offset, location, contract_body);
        unimplemented!("state-variable getter dispatcher")
    }

    /// Emits a `constant` state variable's getter (a folded compile-time value).
    pub fn emit_constant_getter(
        &self,
        state_variable: &StateVariableDefinition,
        contract_body: &BlockRef<'context, '_>,
    ) -> anyhow::Result<()> {
        let _ = (state_variable, contract_body);
        unimplemented!("constant getter emission")
    }

    /// Folds a constant integer expression to a [`BigInt`], when it is one of the
    /// closed set of integer-foldable forms.
    pub fn fold_constant_int(expression: &Expression) -> Option<BigInt> {
        let _ = expression;
        unimplemented!("constant integer folding")
    }

    /// Plans a struct getter's destructured member layout (offset, member type,
    /// result member type), when the result type is a struct.
    pub fn struct_getter_layout(
        struct_definition: &StructDefinition,
        struct_mlir_type: Type<'context>,
        builder: &solx_mlir::Builder<'context>,
    ) -> Option<Vec<(u64, Type<'context>, Type<'context>)>> {
        let _ = (struct_definition, struct_mlir_type, builder);
        unimplemented!("struct getter member layout")
    }

    /// Loads one struct getter member, casting it to its ABI result type through
    /// the single [`TypeConversion`](crate::ast::type_conversion::TypeConversion)
    /// entry.
    pub fn load_getter_member<'block>(
        builder: &solx_mlir::Builder<'context>,
        address: Value<'context, 'block>,
        member_type: Type<'context>,
        result_member_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        let _ = (builder, address, member_type, result_member_type, block);
        unimplemented!("struct getter member load")
    }
}
