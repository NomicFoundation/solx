//!
//! LLVM dialect emission methods and operation name constants.
//!
//! Contains methods on [`Context`](crate::context::Context) that emit
//! LLVM dialect MLIR operations: constants, loads, stores, casts,
//! branches, and calls.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::Identifier;
use melior::ir::Operation;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::DenseI32ArrayAttribute;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::operation::OperationBuilder;
use melior::ir::r#type::IntegerType;

use crate::ICmpPredicate;
use crate::context::builder::Builder;

impl<'context> Builder<'context> {
    /// Bit width for ICmp predicate attributes.
    const PREDICATE_ATTRIBUTE_BIT_WIDTH: u32 = 64;

    // ---- LLVM dialect operation names ----

    /// `llvm.mlir.constant` — materializes a compile-time constant.
    pub const MLIR_CONSTANT: &'static str = "llvm.mlir.constant";
    /// `llvm.return` — returns from a function.
    pub const RETURN: &'static str = "llvm.return";
    /// `llvm.br` — unconditional branch.
    pub const BR: &'static str = "llvm.br";
    /// `llvm.cond_br` — conditional branch.
    pub const COND_BR: &'static str = "llvm.cond_br";
    /// `llvm.icmp` — integer comparison.
    pub const ICMP: &'static str = "llvm.icmp";
    /// `llvm.zext` — zero extension.
    pub const ZEXT: &'static str = "llvm.zext";
    /// `llvm.alloca` — stack allocation.
    pub const ALLOCA: &'static str = "llvm.alloca";
    /// `llvm.inttoptr` — integer to pointer cast.
    pub const INTTOPTR: &'static str = "llvm.inttoptr";
    /// `llvm.call` — function call.
    pub const CALL: &'static str = "llvm.call";
    /// `llvm.add` — integer addition.
    pub const ADD: &'static str = "llvm.add";
    /// `llvm.sub` — integer subtraction.
    pub const SUB: &'static str = "llvm.sub";
    /// `llvm.mul` — integer multiplication.
    pub const MUL: &'static str = "llvm.mul";
    /// `llvm.udiv` — unsigned integer division.
    pub const UDIV: &'static str = "llvm.udiv";
    /// `llvm.urem` — unsigned integer remainder.
    pub const UREM: &'static str = "llvm.urem";
    /// `llvm.and` — bitwise AND.
    pub const AND: &'static str = "llvm.and";
    /// `llvm.or` — bitwise OR.
    pub const OR: &'static str = "llvm.or";
    /// `llvm.xor` — bitwise XOR.
    pub const XOR: &'static str = "llvm.xor";
    /// `llvm.shl` — shift left.
    pub const SHL: &'static str = "llvm.shl";
    /// `llvm.lshr` — logical shift right.
    pub const LSHR: &'static str = "llvm.lshr";
    /// `llvm.ashr` — arithmetic shift right.
    pub const ASHR: &'static str = "llvm.ashr";
    /// `llvm.sdiv` — signed integer division.
    pub const SDIV: &'static str = "llvm.sdiv";
    /// `llvm.srem` — signed integer remainder.
    pub const SREM: &'static str = "llvm.srem";
    /// `llvm.evm.return` — halt execution and return data.
    pub const EVM_RETURN: &'static str = "llvm.evm.return";
    /// `llvm.evm.revert` — halt execution and revert state.
    pub const EVM_REVERT: &'static str = "llvm.evm.revert";
    /// `llvm.evm.calldataload` — load 32 bytes from calldata.
    pub const EVM_CALLDATALOAD: &'static str = "llvm.evm.calldataload";
    /// `llvm.evm.origin` — get execution originator.
    pub const EVM_ORIGIN: &'static str = "llvm.evm.origin";
    /// `llvm.evm.gasprice` — get gas price.
    pub const EVM_GASPRICE: &'static str = "llvm.evm.gasprice";
    /// `llvm.evm.caller` — get caller address.
    pub const EVM_CALLER: &'static str = "llvm.evm.caller";
    /// `llvm.evm.callvalue` — get deposited value.
    pub const EVM_CALLVALUE: &'static str = "llvm.evm.callvalue";
    /// `llvm.evm.timestamp` — get block timestamp.
    pub const EVM_TIMESTAMP: &'static str = "llvm.evm.timestamp";
    /// `llvm.evm.number` — get block number.
    pub const EVM_NUMBER: &'static str = "llvm.evm.number";
    /// `llvm.evm.coinbase` — get block coinbase.
    pub const EVM_COINBASE: &'static str = "llvm.evm.coinbase";
    /// `llvm.evm.chainid` — get chain ID.
    pub const EVM_CHAINID: &'static str = "llvm.evm.chainid";
    /// `llvm.evm.basefee` — get block base fee.
    pub const EVM_BASEFEE: &'static str = "llvm.evm.basefee";
    /// `llvm.evm.gaslimit` — get block gas limit.
    pub const EVM_GASLIMIT: &'static str = "llvm.evm.gaslimit";
    /// `llvm.evm.call` — message call into an account.
    pub const EVM_CALL: &'static str = "llvm.evm.call";

    // ---- Constants ----

    /// Emits an `llvm.mlir.constant` producing an `i256` value.
    ///
    /// Works with both owned `Block` and borrowed `BlockRef`.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_i256_constant<'block, B>(&self, value: i64, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let attribute = IntegerAttribute::new(self.i256_type, value);
        block
            .append_operation(
                OperationBuilder::new(Self::MLIR_CONSTANT, self.unknown_location)
                    .add_attributes(&[(Identifier::new(self.context, "value"), attribute.into())])
                    .add_results(&[self.i256_type])
                    .build()
                    .expect("llvm.mlir.constant operation is well-formed"),
            )
            .result(0)
            .expect("mlir.constant always produces one result")
            .into()
    }

    /// Emits an `i256` constant from a `u64` value without truncation.
    ///
    /// # Errors
    ///
    /// Returns an error if limb decomposition fails (should not happen for
    /// valid `u64` values).
    pub fn emit_i256_from_u64<'block, B>(
        &self,
        value: u64,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if value <= i64::MAX as u64 {
            return Ok(self.emit_i256_constant(value as i64, block));
        }
        let limbs = [value as u32, (value >> 32) as u32];
        self.emit_i256_from_limbs(&limbs, block)
    }

    /// Emits an `i256` constant from a decimal string of arbitrary size.
    ///
    /// # Errors
    ///
    /// Returns an error if the string cannot be parsed as an MLIR integer attribute.
    pub fn emit_i256_from_decimal_str<'block, B>(
        &self,
        value: &str,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let attribute = Attribute::parse(self.context, &format!("{value} : i256"))
            .ok_or_else(|| anyhow::anyhow!("invalid i256 decimal literal: {value}"))?;
        self.emit_constant_operation(Self::MLIR_CONSTANT, attribute, block)
    }

    /// Emits an `i256` constant from a hex string (without `0x` prefix) of arbitrary size.
    ///
    /// # Errors
    ///
    /// Returns an error if the string cannot be parsed as an MLIR integer attribute.
    pub fn emit_i256_from_hex_str<'block, B>(
        &self,
        hexadecimal: &str,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let attribute = Attribute::parse(self.context, &format!("0x{hexadecimal} : i256"))
            .ok_or_else(|| anyhow::anyhow!("invalid i256 hex literal: 0x{hexadecimal}"))?;
        self.emit_constant_operation(Self::MLIR_CONSTANT, attribute, block)
    }

    // ---- Memory ----

    /// Emits an `llvm.store` to a pointer.
    pub fn emit_store<'block, B>(
        &self,
        value: Value<'context, 'block>,
        pointer: Value<'context, 'block>,
        block: &B,
    ) where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(melior::dialect::llvm::store(
            self.context,
            value,
            pointer,
            self.unknown_location,
            melior::dialect::llvm::LoadStoreOptions::default(),
        ));
    }

    /// Emits an `llvm.load` from a pointer.
    ///
    /// # Errors
    ///
    /// Returns an error if the load operation result cannot be extracted.
    pub fn emit_load<'block, B>(
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
            .append_operation(melior::dialect::llvm::load(
                self.context,
                pointer,
                result_type,
                self.unknown_location,
                melior::dialect::llvm::LoadStoreOptions::default(),
            ))
            .result(0)?
            .into())
    }

    // ---- Casts ----

    /// Emits an `llvm.inttoptr` cast.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_inttoptr<'block, B>(
        &self,
        value: Value<'context, 'block>,
        pointer_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                OperationBuilder::new(Self::INTTOPTR, self.unknown_location)
                    .add_operands(&[value])
                    .add_results(&[pointer_type])
                    .build()
                    .expect("llvm.inttoptr operation is well-formed"),
            )
            .result(0)
            .expect("inttoptr always produces one result")
            .into()
    }

    /// Emits an `llvm.zext` from `i1` to `i256`.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_zext_to_i256<'block, B>(
        &self,
        value: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                OperationBuilder::new(Self::ZEXT, self.unknown_location)
                    .add_operands(&[value])
                    .add_results(&[self.i256_type])
                    .build()
                    .expect("llvm.zext operation is well-formed"),
            )
            .result(0)
            .expect("zext always produces one result")
            .into()
    }

    // ---- Arithmetic ----

    /// Emits an `llvm.icmp` comparison returning `i1`.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_icmp<'block, B>(
        &self,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        predicate: ICmpPredicate,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                OperationBuilder::new(Self::ICMP, self.unknown_location)
                    .add_operands(&[lhs, rhs])
                    .add_attributes(&[(
                        Identifier::new(self.context, "predicate"),
                        IntegerAttribute::new(
                            IntegerType::new(self.context, Self::PREDICATE_ATTRIBUTE_BIT_WIDTH)
                                .into(),
                            predicate as i64,
                        )
                        .into(),
                    )])
                    .add_results(&[self.i1_type])
                    .build()
                    .expect("llvm.icmp operation is well-formed"),
            )
            .result(0)
            .expect("llvm.icmp always produces one result")
            .into()
    }

    // ---- EVM Intrinsics ----

    /// Emits an EVM intrinsic as a direct MLIR operation.
    ///
    /// EVM intrinsics (`llvm.evm.caller`, `llvm.evm.call`, etc.) are MLIR
    /// operations, not function calls — they must be emitted via
    /// `OperationBuilder`, not `llvm.call`.
    ///
    /// Returns `Some(value)` when `has_result` is true, `None` for void
    /// intrinsics.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation cannot be constructed or the result
    /// cannot be extracted.
    pub fn emit_evm_intrinsic<'block, B>(
        &self,
        name: &str,
        operands: &[Value<'context, 'block>],
        has_result: bool,
        block: &B,
    ) -> anyhow::Result<Option<Value<'context, 'block>>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let result_types: Vec<Type<'context>> = if has_result {
            vec![self.i256_type]
        } else {
            vec![]
        };
        let operation = block.append_operation(
            OperationBuilder::new(name, self.unknown_location)
                .add_operands(operands)
                .add_results(&result_types)
                .build()?,
        );
        if has_result {
            Ok(Some(operation.result(0)?.into()))
        } else {
            Ok(None)
        }
    }

    // ---- Branches ----

    /// Builds an `llvm.br` operation (unconditional branch).
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn llvm_br<'block>(
        &self,
        destination: &melior::ir::Block<'context>,
        destination_arguments: &[Value<'context, 'block>],
    ) -> Operation<'context> {
        OperationBuilder::new(Self::BR, self.unknown_location)
            .add_operands(destination_arguments)
            .add_attributes(&[
                self.operand_segment_sizes_attribute(&[destination_arguments.len() as i32])
            ])
            .add_successors(&[destination])
            .build()
            .expect("llvm.br operation is well-formed")
    }

    /// Builds an `llvm.cond_br` operation (conditional branch).
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn llvm_cond_br<'block>(
        &self,
        condition: Value<'context, 'block>,
        true_destination: &melior::ir::Block<'context>,
        false_destination: &melior::ir::Block<'context>,
        true_arguments: &[Value<'context, 'block>],
        false_arguments: &[Value<'context, 'block>],
    ) -> Operation<'context> {
        let mut operands = vec![condition];
        operands.extend_from_slice(true_arguments);
        operands.extend_from_slice(false_arguments);
        OperationBuilder::new(Self::COND_BR, self.unknown_location)
            .add_operands(&operands)
            .add_attributes(&[self.operand_segment_sizes_attribute(&[
                1,
                true_arguments.len() as i32,
                false_arguments.len() as i32,
            ])])
            .add_successors(&[true_destination, false_destination])
            .build()
            .expect("llvm.cond_br operation is well-formed")
    }

    // ---- Attributes ----

    /// Builds the `operandSegmentSizes` dense `i32` array attribute.
    fn operand_segment_sizes_attribute(
        &self,
        segments: &[i32],
    ) -> (Identifier<'context>, Attribute<'context>) {
        (
            Identifier::new(self.context, "operandSegmentSizes"),
            DenseI32ArrayAttribute::new(self.context, segments).into(),
        )
    }
}
