//!
//! MLIR builder for Sol dialect emission.
//!
//! Contains the [`Builder`] type with cached MLIR types and emission methods
//! for Sol dialect operations: contracts, functions, constants, control flow,
//! memory, comparisons, calls, state variables, and EVM context intrinsics.
//!

use std::collections::HashMap;

use melior::ir::Attribute;
use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Identifier;
use melior::ir::Location;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::Type;
use melior::ir::TypeLike;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::attribute::FlatSymbolRefAttribute;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::operation::OperationBuilder;
use melior::ir::operation::OperationLike;
use melior::ir::r#type::FunctionType;
use melior::ir::r#type::IntegerType;

use crate::CmpPredicate;
use crate::StateMutability;

/// Cached MLIR types and emission methods for building MLIR operations.
///
/// Types are cached in a [`HashMap`] keyed by their textual representation.
/// New types can be added without changing the struct definition.
pub struct Builder<'context> {
    /// The MLIR context with all dialects and translations registered.
    pub context: &'context melior::Context,
    /// Cached unknown source location.
    pub unknown_location: Location<'context>,
    /// Cached MLIR types, keyed by textual representation.
    pub types: HashMap<&'static str, Type<'context>>,
}

impl<'context> Builder<'context> {
    // ---- Type cache keys ----

    /// 1-bit boolean type key.
    pub const I1: &'static str = "i1";
    /// Unsigned 160-bit integer type key (address width).
    pub const UI160: &'static str = "ui160";
    /// Unsigned 256-bit integer type key.
    pub const UI256: &'static str = "ui256";
    /// Sol address type key.
    pub const SOL_ADDRESS: &'static str = "!sol.address";
    /// Sol storage pointer type key.
    pub const SOL_PTR_STORAGE: &'static str = "!sol.ptr<ui256, Storage>";

    // ---- Sol dialect operation names ----

    /// `sol.contract` — contract symbol table container.
    pub const SOL_CONTRACT: &'static str = "sol.contract";
    /// `sol.func` — function definition with selector and mutability.
    pub const SOL_FUNC: &'static str = "sol.func";
    /// `sol.constant` — compile-time constant.
    pub const SOL_CONSTANT: &'static str = "sol.constant";
    /// `sol.exp` — exponentiation.
    pub const SOL_EXP: &'static str = "sol.exp";
    /// `sol.return` — return from function.
    pub const SOL_RETURN: &'static str = "sol.return";
    /// `sol.revert` — revert execution.
    pub const SOL_REVERT: &'static str = "sol.revert";
    /// `sol.require` — conditional revert.
    pub const SOL_REQUIRE: &'static str = "sol.require";
    /// `sol.alloca` — stack allocation.
    pub const SOL_ALLOCA: &'static str = "sol.alloca";
    /// `sol.load` — load from pointer.
    pub const SOL_LOAD: &'static str = "sol.load";
    /// `sol.store` — store to pointer.
    pub const SOL_STORE: &'static str = "sol.store";
    /// `sol.call` — function call.
    pub const SOL_CALL: &'static str = "sol.call";
    /// `sol.if` — structured if/else.
    pub const SOL_IF: &'static str = "sol.if";
    /// `sol.add` — unchecked addition.
    pub const SOL_ADD: &'static str = "sol.add";
    /// `sol.sub` — unchecked subtraction.
    pub const SOL_SUB: &'static str = "sol.sub";
    /// `sol.mul` — unchecked multiplication.
    pub const SOL_MUL: &'static str = "sol.mul";
    /// `sol.cadd` — checked addition (reverts on overflow).
    pub const SOL_CADD: &'static str = "sol.cadd";
    /// `sol.csub` — checked subtraction (reverts on underflow).
    pub const SOL_CSUB: &'static str = "sol.csub";
    /// `sol.cmul` — checked multiplication (reverts on overflow).
    pub const SOL_CMUL: &'static str = "sol.cmul";
    /// `sol.cdiv` — checked division (reverts on `int.min / -1` overflow).
    pub const SOL_CDIV: &'static str = "sol.cdiv";
    /// `sol.div` — unchecked division.
    pub const SOL_DIV: &'static str = "sol.div";
    /// `sol.mod` — unchecked modulo.
    pub const SOL_MOD: &'static str = "sol.mod";
    /// `sol.cmp` — comparison.
    pub const SOL_CMP: &'static str = "sol.cmp";
    /// `sol.cast` — type cast.
    pub const SOL_CAST: &'static str = "sol.cast";
    /// `sol.address_cast` — address type cast.
    pub const SOL_ADDRESS_CAST: &'static str = "sol.address_cast";
    /// `sol.state_var` — state variable declaration.
    pub const SOL_STATE_VAR: &'static str = "sol.state_var";
    /// `sol.and` — bitwise AND.
    pub const SOL_AND: &'static str = "sol.and";
    /// `sol.or` — bitwise OR.
    pub const SOL_OR: &'static str = "sol.or";
    /// `sol.xor` — bitwise XOR.
    pub const SOL_XOR: &'static str = "sol.xor";
    /// `sol.shl` — shift left.
    pub const SOL_SHL: &'static str = "sol.shl";
    /// `sol.shr` — shift right.
    pub const SOL_SHR: &'static str = "sol.shr";
    /// `sol.addr_of` — address of a declared state variable.
    pub const SOL_ADDR_OF: &'static str = "sol.addr_of";

    // ---- Sol dialect EVM context operation names ----

    /// `sol.caller` — `msg.sender`.
    pub const SOL_CALLER: &'static str = "sol.caller";
    /// `sol.origin` — `tx.origin`.
    pub const SOL_ORIGIN: &'static str = "sol.origin";
    /// `sol.gasprice` — `tx.gasprice`.
    pub const SOL_GASPRICE: &'static str = "sol.gasprice";
    /// `sol.callvalue` — `msg.value`.
    pub const SOL_CALLVALUE: &'static str = "sol.callvalue";
    /// `sol.timestamp` — `block.timestamp`.
    pub const SOL_TIMESTAMP: &'static str = "sol.timestamp";
    /// `sol.blocknumber` — `block.number`.
    pub const SOL_BLOCKNUMBER: &'static str = "sol.blocknumber";
    /// `sol.coinbase` — `block.coinbase`.
    pub const SOL_COINBASE: &'static str = "sol.coinbase";
    /// `sol.chainid` — `block.chainid`.
    pub const SOL_CHAINID: &'static str = "sol.chainid";
    /// `sol.basefee` — `block.basefee`.
    pub const SOL_BASEFEE: &'static str = "sol.basefee";
    /// `sol.gaslimit` — `block.gaslimit`.
    pub const SOL_GASLIMIT: &'static str = "sol.gaslimit";

    // ---- Private constants ----

    /// Bit width of a Solidity function selector (4 bytes).
    const SELECTOR_BIT_WIDTH: u32 = solx_utils::BIT_LENGTH_X32 as u32;

    // ==== Constructor ====

    /// Creates a new builder with pre-cached types.
    pub fn new(context: &'context melior::Context) -> Self {
        let unknown_location = Location::unknown(context);
        let types = HashMap::from([
            (
                Self::I1,
                Type::from(IntegerType::new(
                    context,
                    solx_utils::BIT_LENGTH_BOOLEAN as u32,
                )),
            ),
            (
                Self::UI160,
                Type::from(IntegerType::unsigned(
                    context,
                    solx_utils::BIT_LENGTH_ETH_ADDRESS as u32,
                )),
            ),
            (
                Self::UI256,
                Type::from(IntegerType::unsigned(
                    context,
                    solx_utils::BIT_LENGTH_FIELD as u32,
                )),
            ),
            // SAFETY: `solxCreateAddressType` returns a valid MlirType from
            // the C++ Sol dialect. The context pointer is valid.
            (Self::SOL_ADDRESS, unsafe {
                Type::from_raw(crate::ffi::solxCreateAddressType(context.to_raw(), false))
            }),
            (
                Self::SOL_PTR_STORAGE,
                Type::parse(context, Self::SOL_PTR_STORAGE).expect("valid sol.ptr type syntax"),
            ),
        ]);
        Self {
            context,
            unknown_location,
            types,
        }
    }

    /// Returns a cached MLIR type by its textual key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not in the cache.
    pub fn get_type(&self, key: &str) -> Type<'context> {
        self.types[key]
    }

    /// Returns the bit width of an MLIR integer type, or 256 for non-integer types.
    pub fn integer_bit_width(r#type: Type<'_>) -> u32 {
        IntegerType::try_from(r#type).map_or(solx_utils::BIT_LENGTH_FIELD as u32, |integer_type| {
            integer_type.width()
        })
    }

    /// Creates a `sol::AddressType` with the given payability.
    pub fn create_address_type(&self, payable: bool) -> Type<'context> {
        // SAFETY: `solxCreateAddressType` returns a valid MlirType from the
        // C++ Sol dialect. The context pointer is valid.
        unsafe {
            Type::from_raw(crate::ffi::solxCreateAddressType(
                self.context.to_raw(),
                payable,
            ))
        }
    }

    /// Creates a `sol::PointerType` with the given element type and data location.
    pub fn create_pointer_type(
        &self,
        element_type: Type<'context>,
        location: solx_utils::DataLocation,
    ) -> Type<'context> {
        unsafe {
            Type::from_raw(crate::ffi::solxCreatePointerType(
                self.context.to_raw(),
                element_type.to_raw(),
                location as u32,
            ))
        }
    }

    // ==== Structure ====

    /// Emits a `sol.contract` operation with a body region.
    ///
    /// Returns the body block inside the contract region for appending
    /// function definitions.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_contract<'block>(
        &self,
        name: &str,
        kind: crate::ContractKind,
        block: &BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let body_region = Region::new();
        let body_block = Block::new(&[]);
        body_region.append_block(body_block);

        let operation = block.append_operation(
            OperationBuilder::new(Self::SOL_CONTRACT, self.unknown_location)
                .add_attributes(&[
                    (
                        Identifier::new(self.context, "sym_name"),
                        StringAttribute::new(self.context, name).into(),
                    ),
                    // SAFETY: `solxCreateContractKindAttr` returns a valid
                    // MlirAttribute from the C++ Sol dialect.
                    (Identifier::new(self.context, "kind"), unsafe {
                        Attribute::from_raw(crate::ffi::solxCreateContractKindAttr(
                            self.context.to_raw(),
                            kind as u32,
                        ))
                    }),
                ])
                .add_regions([body_region])
                .build()
                .expect("sol.contract operation is well-formed"),
        );
        operation
            .region(0)
            .expect("contract has one region")
            .first_block()
            .expect("contract body has one block")
    }

    /// Emits a `sol.func` operation with the given name, parameter types,
    /// result types, selector, and state mutability.
    ///
    /// Returns the entry block of the function body for appending operations.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_func<'block>(
        &self,
        name: &str,
        parameter_types: &[Type<'context>],
        result_types: &[Type<'context>],
        selector: Option<u32>,
        state_mutability: StateMutability,
        block: &BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let function_type = FunctionType::new(self.context, parameter_types, result_types);
        let body_region = Region::new();
        let block_argument_types: Vec<(Type<'context>, Location<'context>)> = parameter_types
            .iter()
            .map(|t| (*t, self.unknown_location))
            .collect();
        let entry_block = Block::new(&block_argument_types);
        body_region.append_block(entry_block);

        // SAFETY: `solxCreateStateMutabilityAttr` returns a valid
        // MlirAttribute from the C++ Sol dialect.
        let mutability_attribute = unsafe {
            Attribute::from_raw(crate::ffi::solxCreateStateMutabilityAttr(
                self.context.to_raw(),
                state_mutability as u32,
            ))
        };

        let mut attributes: Vec<(Identifier<'context>, Attribute<'context>)> = vec![
            (
                Identifier::new(self.context, "sym_name"),
                StringAttribute::new(self.context, name).into(),
            ),
            (
                Identifier::new(self.context, "function_type"),
                TypeAttribute::new(function_type.into()).into(),
            ),
            (
                Identifier::new(self.context, "state_mutability"),
                mutability_attribute,
            ),
        ];

        if let Some(selector_value) = selector {
            attributes.push((
                Identifier::new(self.context, "selector"),
                IntegerAttribute::new(
                    IntegerType::new(self.context, Self::SELECTOR_BIT_WIDTH).into(),
                    selector_value as i64,
                )
                .into(),
            ));
        }

        if selector.is_some() || matches!(kind, Some(crate::FunctionKind::Constructor)) {
            attributes.push((
                Identifier::new(self.context, "orig_fn_type"),
                TypeAttribute::new(function_type.into()).into(),
            ));
        }

        let operation = block.append_operation(
            OperationBuilder::new(Self::SOL_FUNC, self.unknown_location)
                .add_attributes(&attributes)
                .add_regions([body_region])
                .build()
                .expect("sol.func operation is well-formed"),
        );
        operation
            .region(0)
            .expect("func has one region")
            .first_block()
            .expect("func body has entry block")
    }

    // ==== Constants ====

    /// Emits a `sol.constant` of the given type.
    ///
    /// Use this variant when the constant type is known at emission time.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_constant<'block, B>(
        &self,
        value: i64,
        ty: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let attribute = IntegerAttribute::new(ty, value);
        block
            .append_operation(
                OperationBuilder::new(Self::SOL_CONSTANT, self.unknown_location)
                    .add_attributes(&[(Identifier::new(self.context, "value"), attribute.into())])
                    .add_results(&[ty])
                    .build()
                    .expect("sol.constant operation is well-formed"),
            )
            .result(0)
            .expect("sol.constant always produces one result")
            .into()
    }

    /// Emits a `sol.constant` of the given type from a decimal string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string cannot be parsed as an MLIR integer attribute.
    pub fn emit_sol_constant_from_decimal_str<'block, B>(
        &self,
        value: &str,
        ty: Type<'context>,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let attribute = Attribute::parse(self.context, &format!("{value} : {ty}"))
            .ok_or_else(|| anyhow::anyhow!("invalid {ty} decimal literal: {value}"))?;
        self.emit_constant_operation(Self::SOL_CONSTANT, attribute, ty, block)
    }

    /// Emits a `sol.constant` of the given type from a hex string (without `0x` prefix).
    ///
    /// # Errors
    ///
    /// Returns an error if the string cannot be parsed as an MLIR integer attribute.
    pub fn emit_sol_constant_from_hex_str<'block, B>(
        &self,
        hexadecimal: &str,
        ty: Type<'context>,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let attribute = Attribute::parse(self.context, &format!("0x{hexadecimal} : {ty}"))
            .ok_or_else(|| anyhow::anyhow!("invalid {ty} hex literal: 0x{hexadecimal}"))?;
        self.emit_constant_operation(Self::SOL_CONSTANT, attribute, ty, block)
    }

    /// Emits an all-ones `sol.constant` for the given integer type.
    ///
    /// # Errors
    ///
    /// Returns an error if the constant cannot be parsed as an MLIR integer attribute.
    pub fn emit_sol_constant_all_ones<'block, B>(
        &self,
        integer_type: Type<'context>,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let bit_width = Self::integer_bit_width(integer_type);
        let all_ones_hex = "f".repeat(bit_width as usize / 4);
        self.emit_sol_constant_from_hex_str(&all_ones_hex, integer_type, block)
    }

    // ==== Terminators ====

    /// Emits a `sol.revert` with an empty signature (no error data).
    // TODO(sol-dialect): mark `sol.revert` as `IsTerminator` like `sol.return`
    // so callers don't need to append `llvm.unreachable` after it.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_revert<'block, B>(&self, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            OperationBuilder::new(Self::SOL_REVERT, self.unknown_location)
                .add_attributes(&[(
                    Identifier::new(self.context, "signature"),
                    StringAttribute::new(self.context, "").into(),
                )])
                .build()
                .expect("sol.revert operation is well-formed"),
        );
    }

    /// Emits a `sol.require` conditional revert with an empty signature.
    ///
    /// Reverts if `condition` is false. Not a terminator — execution continues
    /// after this op when the condition is true.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_require<'block, B>(&self, condition: Value<'context, 'block>, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            OperationBuilder::new(Self::SOL_REQUIRE, self.unknown_location)
                .add_operands(&[condition])
                .add_attributes(&[(
                    Identifier::new(self.context, "signature"),
                    StringAttribute::new(self.context, "").into(),
                )])
                .build()
                .expect("sol.require operation is well-formed"),
        );
    }

    /// Emits a `sol.return` terminator.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_return<'block, B>(&self, operands: &[Value<'context, 'block>], block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            OperationBuilder::new(Self::SOL_RETURN, self.unknown_location)
                .add_operands(operands)
                .build()
                .expect("sol.return operation is well-formed"),
        );
    }

    // ==== Control flow ====

    /// Emits a `sol.if` with then and else regions.
    ///
    /// Returns `(then_block, else_block)`. The caller emits into each region
    /// and terminates them with `sol.yield`.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_if<'block>(
        &self,
        condition: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> (BlockRef<'context, 'block>, BlockRef<'context, 'block>)
    where
        'context: 'block,
    {
        let then_region = Region::new();
        let then_block = Block::new(&[]);
        then_region.append_block(then_block);

        let else_region = Region::new();
        let else_block = Block::new(&[]);
        else_region.append_block(else_block);

        let operation = block.append_operation(
            OperationBuilder::new("sol.if", self.unknown_location)
                .add_operands(&[condition])
                .add_regions([then_region, else_region])
                .build()
                .expect("sol.if operation is well-formed"),
        );

        let then_ref = operation
            .region(0)
            .expect("sol.if has then region")
            .first_block()
            .expect("then region has a block");
        let else_ref = operation
            .region(1)
            .expect("sol.if has else region")
            .first_block()
            .expect("else region has a block");
        (then_ref, else_ref)
    }

    /// Emits a value-producing `scf.if` with then and else regions.
    ///
    /// Returns `(then_block, else_block)`. Each region must be terminated
    /// with `emit_scf_yield` passing a value matching the result type.
    /// The operation result is the yielded value from the taken branch.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation result cannot be extracted.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_scf_if<'block>(
        &self,
        condition: Value<'context, 'block>,
        result_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        BlockRef<'context, 'block>,
        BlockRef<'context, 'block>,
        Value<'context, 'block>,
    )>
    where
        'context: 'block,
    {
        let then_region = Region::new();
        let then_block = Block::new(&[]);
        then_region.append_block(then_block);

        let else_region = Region::new();
        let else_block = Block::new(&[]);
        else_region.append_block(else_block);

        let operation = block.append_operation(
            OperationBuilder::new("scf.if", self.unknown_location)
                .add_operands(&[condition])
                .add_results(&[result_type])
                .add_regions([then_region, else_region])
                .build()
                .expect("scf.if operation is well-formed"),
        );

        let result = operation.result(0)?.into();
        let then_ref = operation
            .region(0)
            .expect("scf.if has then region")
            .first_block()
            .expect("then region has a block");
        let else_ref = operation
            .region(1)
            .expect("scf.if has else region")
            .first_block()
            .expect("else region has a block");
        Ok((then_ref, else_ref, result))
    }

    /// Emits a `scf.yield` region terminator with a value.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_scf_yield<'block, B>(&self, operands: &[Value<'context, 'block>], block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            OperationBuilder::new("scf.yield", self.unknown_location)
                .add_operands(operands)
                .build()
                .expect("scf.yield operation is well-formed"),
        );
    }

    /// Emits a `sol.while` with condition and body regions.
    ///
    /// Returns `(cond_block, body_block)`. The condition region must be
    /// terminated with `sol.condition`. The body region with `sol.yield`.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_while<'block>(
        &self,
        block: &BlockRef<'context, 'block>,
    ) -> (BlockRef<'context, 'block>, BlockRef<'context, 'block>) {
        let cond_region = Region::new();
        let cond_block = Block::new(&[]);
        cond_region.append_block(cond_block);

        let body_region = Region::new();
        let body_block = Block::new(&[]);
        body_region.append_block(body_block);

        let operation = block.append_operation(
            OperationBuilder::new("sol.while", self.unknown_location)
                .add_regions([cond_region, body_region])
                .build()
                .expect("sol.while operation is well-formed"),
        );

        let cond_ref = operation
            .region(0)
            .expect("sol.while has cond region")
            .first_block()
            .expect("cond region has a block");
        let body_ref = operation
            .region(1)
            .expect("sol.while has body region")
            .first_block()
            .expect("body region has a block");
        (cond_ref, body_ref)
    }

    /// Emits a `sol.do` (do-while) with body and condition regions.
    ///
    /// Returns `(body_block, cond_block)`. The body executes first.
    /// Body terminates with `sol.yield`, condition with `sol.condition`.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_do_while<'block>(
        &self,
        block: &BlockRef<'context, 'block>,
    ) -> (BlockRef<'context, 'block>, BlockRef<'context, 'block>) {
        let body_region = Region::new();
        let body_block = Block::new(&[]);
        body_region.append_block(body_block);

        let cond_region = Region::new();
        let cond_block = Block::new(&[]);
        cond_region.append_block(cond_block);

        let operation = block.append_operation(
            OperationBuilder::new("sol.do", self.unknown_location)
                .add_regions([body_region, cond_region])
                .build()
                .expect("sol.do operation is well-formed"),
        );

        let body_ref = operation
            .region(0)
            .expect("sol.do has body region")
            .first_block()
            .expect("body region has a block");
        let cond_ref = operation
            .region(1)
            .expect("sol.do has cond region")
            .first_block()
            .expect("cond region has a block");
        (body_ref, cond_ref)
    }

    /// Emits a `sol.for` with condition, body, and step regions.
    ///
    /// Returns `(cond_block, body_block, step_block)`. Condition terminates
    /// with `sol.condition`, body and step with `sol.yield`.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_for<'block>(
        &self,
        block: &BlockRef<'context, 'block>,
    ) -> (
        BlockRef<'context, 'block>,
        BlockRef<'context, 'block>,
        BlockRef<'context, 'block>,
    ) {
        let cond_region = Region::new();
        let cond_block = Block::new(&[]);
        cond_region.append_block(cond_block);

        let body_region = Region::new();
        let body_block = Block::new(&[]);
        body_region.append_block(body_block);

        let step_region = Region::new();
        let step_block = Block::new(&[]);
        step_region.append_block(step_block);

        let operation = block.append_operation(
            OperationBuilder::new("sol.for", self.unknown_location)
                .add_regions([cond_region, body_region, step_region])
                .build()
                .expect("sol.for operation is well-formed"),
        );

        let cond_ref = operation
            .region(0)
            .expect("sol.for has cond region")
            .first_block()
            .expect("cond region has a block");
        let body_ref = operation
            .region(1)
            .expect("sol.for has body region")
            .first_block()
            .expect("body region has a block");
        let step_ref = operation
            .region(2)
            .expect("sol.for has step region")
            .first_block()
            .expect("step region has a block");
        (cond_ref, body_ref, step_ref)
    }

    /// Emits a `sol.yield` region terminator.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_yield<'block, B>(&self, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            OperationBuilder::new("sol.yield", self.unknown_location)
                .build()
                .expect("sol.yield operation is well-formed"),
        );
    }

    /// Emits a `sol.condition` loop condition terminator.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_condition<'block, B>(&self, condition: Value<'context, 'block>, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            OperationBuilder::new("sol.condition", self.unknown_location)
                .add_operands(&[condition])
                .build()
                .expect("sol.condition operation is well-formed"),
        );
    }

    /// Emits a `sol.break` terminator.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_break<'block, B>(&self, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            OperationBuilder::new("sol.break", self.unknown_location)
                .build()
                .expect("sol.break operation is well-formed"),
        );
    }

    /// Emits a `sol.continue` terminator.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_continue<'block, B>(&self, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            OperationBuilder::new("sol.continue", self.unknown_location)
                .build()
                .expect("sol.continue operation is well-formed"),
        );
    }

    // ==== Memory ====

    /// Emits a `sol.alloca` for a local variable, returning the pointer.
    ///
    /// Emits a `sol.alloca` for a local variable of the given element type.
    ///
    /// Returns a `!sol.ptr<{element_type}, Stack>` pointer. Use this when
    /// the declared Solidity type is known (e.g. `uint64` → `ui64`).
    ///
    /// # Panics
    ///
    /// Panics if the MLIR type or operation cannot be constructed, indicating
    /// a bug in the builder.
    pub fn emit_sol_alloca<'block, B>(
        &self,
        element_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let ptr_type = self.create_pointer_type(element_type, solx_utils::DataLocation::Stack);
        block
            .append_operation(
                OperationBuilder::new(Self::SOL_ALLOCA, self.unknown_location)
                    .add_attributes(&[(
                        Identifier::new(self.context, "alloc_type"),
                        TypeAttribute::new(element_type).into(),
                    )])
                    .add_results(&[ptr_type])
                    .build()
                    .expect("sol.alloca operation is well-formed"),
            )
            .result(0)
            .expect("sol.alloca always produces one result")
            .into()
    }

    /// Emits a `sol.load` from a pointer with an explicit result type.
    ///
    /// Use this when the pointer element type is known at emission time.
    ///
    /// # Errors
    ///
    /// Returns an error if the load operation result cannot be extracted.
    pub fn emit_sol_load<'block, B>(
        &self,
        pointer: Value<'context, 'block>,
        result_type: Type<'context>,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Ok(block
            .append_operation(
                OperationBuilder::new(Self::SOL_LOAD, self.unknown_location)
                    .add_operands(&[pointer])
                    .add_results(&[result_type])
                    .build()
                    .expect("sol.load operation is well-formed"),
            )
            .result(0)?
            .into())
    }

    /// Emits a `sol.store` to a pointer.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug
    /// in the builder.
    pub fn emit_sol_store<'block, B>(
        &self,
        value: Value<'context, 'block>,
        pointer: Value<'context, 'block>,
        block: &B,
    ) where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            OperationBuilder::new(Self::SOL_STORE, self.unknown_location)
                .add_operands(&[value, pointer])
                .build()
                .expect("sol.store operation is well-formed"),
        );
    }

    // ==== Calls ====

    /// Emits a `sol.call` operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the call operation result cannot be extracted.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_call<'block, B>(
        &self,
        callee: &str,
        operands: &[Value<'context, 'block>],
        result_types: &[Type<'context>],
        block: &B,
    ) -> anyhow::Result<Option<Value<'context, 'block>>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let operation = block.append_operation(
            OperationBuilder::new(Self::SOL_CALL, self.unknown_location)
                .add_operands(operands)
                .add_attributes(&[(
                    Identifier::new(self.context, "callee"),
                    FlatSymbolRefAttribute::new(self.context, callee).into(),
                )])
                .add_results(result_types)
                .build()
                .expect("sol.call operation is well-formed"),
        );
        if result_types.is_empty() {
            Ok(None)
        } else {
            // TODO: return all results for multi-return functions
            Ok(Some(operation.result(0)?.into()))
        }
    }

    // ==== Comparisons ====

    /// Emits a `sol.cmp` comparison returning `i1`.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_cmp<'block, B>(
        &self,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        predicate: CmpPredicate,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                OperationBuilder::new(Self::SOL_CMP, self.unknown_location)
                    .add_operands(&[lhs, rhs])
                    .add_attributes(&[(
                        Identifier::new(self.context, "predicate"),
                        IntegerAttribute::new(
                            IntegerType::new(self.context, solx_utils::BIT_LENGTH_X64 as u32)
                                .into(),
                            predicate as i64,
                        )
                        .into(),
                    )])
                    .add_results(&[self.get_type(Self::I1)])
                    .build()
                    .expect("sol.cmp operation is well-formed"),
            )
            .result(0)
            .expect("sol.cmp always produces one result")
            .into()
    }

    /// Emits a `sol.cast` to an arbitrary target type.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_cast<'block, B>(
        &self,
        value: Value<'context, 'block>,
        to_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if value.r#type() == to_type {
            return value;
        }
        block
            .append_operation(
                OperationBuilder::new(Self::SOL_CAST, self.unknown_location)
                    .add_operands(&[value])
                    .add_results(&[to_type])
                    .build()
                    .expect("sol.cast operation is well-formed"),
            )
            .result(0)
            .expect("sol.cast always produces one result")
            .into()
    }

    /// Emits a `sol.address_cast` to convert between address and integer types.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_address_cast<'block, B>(
        &self,
        value: Value<'context, 'block>,
        to_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                OperationBuilder::new(Self::SOL_ADDRESS_CAST, self.unknown_location)
                    .add_operands(&[value])
                    .add_results(&[to_type])
                    .build()
                    .expect("sol.address_cast operation is well-formed"),
            )
            .result(0)
            .expect("sol.address_cast always produces one result")
            .into()
    }

    // ==== State variables ====

    /// Emits a `sol.state_var` declaration inside a contract body.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_state_var<'block, B>(&self, name: &str, slot: u64, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            OperationBuilder::new(Self::SOL_STATE_VAR, self.unknown_location)
                .add_attributes(&[
                    (
                        Identifier::new(self.context, "sym_name"),
                        StringAttribute::new(self.context, name).into(),
                    ),
                    (
                        Identifier::new(self.context, "type"),
                        TypeAttribute::new(self.get_type(Self::UI256)).into(),
                    ),
                    (
                        Identifier::new(self.context, "slot"),
                        Attribute::parse(self.context, &format!("{slot} : i256"))
                            .expect("valid slot literal"),
                    ),
                    (
                        Identifier::new(self.context, "byte_offset"),
                        IntegerAttribute::new(
                            IntegerType::new(self.context, solx_utils::BIT_LENGTH_X32 as u32)
                                .into(),
                            0,
                        )
                        .into(),
                    ),
                ])
                .build()
                .expect("sol.state_var operation is well-formed"),
        );
    }

    /// Emits a `sol.addr_of` returning a `!sol.ptr<ui256, Storage>`.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_addr_of<'block, B>(
        &self,
        name: &str,
        result_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                OperationBuilder::new(Self::SOL_ADDR_OF, self.unknown_location)
                    .add_attributes(&[(
                        Identifier::new(self.context, "var"),
                        FlatSymbolRefAttribute::new(self.context, name).into(),
                    )])
                    .add_results(&[result_type])
                    .build()
                    .expect("sol.addr_of operation is well-formed"),
            )
            .result(0)
            .expect("sol.addr_of always produces one result")
            .into()
    }

    // ==== EVM context intrinsics ====

    /// Emits a Sol dialect EVM context intrinsic (e.g. `sol.caller`, `sol.timestamp`).
    ///
    /// These are zero-operand operations that return a single value of
    /// `result_type` (e.g. `ui256` for numeric intrinsics, `!sol.address`
    /// for address-returning intrinsics like `sol.caller`).
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_intrinsic<'block, B>(
        &self,
        name: &str,
        result_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                OperationBuilder::new(name, self.unknown_location)
                    .add_results(&[result_type])
                    .build()
                    .expect("sol intrinsic operation is well-formed"),
            )
            .result(0)
            .expect("sol intrinsic always produces one result")
            .into()
    }

    // ==== Shared helpers ====

    /// Shared helper for emitting a two-operand operation with one result.
    ///
    /// # Errors
    ///
    /// Returns an error if the MLIR operation cannot be constructed.
    pub fn emit_binary_operation<'block, B>(
        &self,
        operation_name: &str,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        result_type: Type<'context>,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Ok(block
            .append_operation(
                OperationBuilder::new(operation_name, self.unknown_location)
                    .add_operands(&[lhs, rhs])
                    .add_results(&[result_type])
                    .build()?,
            )
            .result(0)
            .expect("binary operation always produces one result")
            .into())
    }

    /// Shared helper for emitting a constant operation with an attribute.
    ///
    /// # Errors
    ///
    /// Returns an error if the MLIR operation cannot be constructed.
    fn emit_constant_operation<'block, B>(
        &self,
        operation_name: &str,
        attribute: Attribute<'context>,
        result_type: Type<'context>,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Ok(block
            .append_operation(
                OperationBuilder::new(operation_name, self.unknown_location)
                    .add_attributes(&[(Identifier::new(self.context, "value"), attribute)])
                    .add_results(&[result_type])
                    .build()
                    .expect("constant operation is well-formed"),
            )
            .result(0)?
            .into())
    }
}
