//!
//! LLVM dialect operations for the MLIR builder.
//!
//! Contains methods on [`MlirContext`] that emit LLVM dialect MLIR
//! operations: constants, loads, stores, casts, branches, and calls.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::Identifier;
use melior::ir::Operation;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::DenseI32ArrayAttribute;
use melior::ir::attribute::FlatSymbolRefAttribute;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::operation::OperationBuilder;
use melior::ir::r#type::IntegerType;

use crate::ICmpPredicate;
use crate::builder::MlirContext;

impl<'context> MlirContext<'context> {
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
                OperationBuilder::new(crate::ops::MLIR_CONSTANT, self.unknown_location)
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
    pub fn emit_i256_from_u64<'block, B>(&self, value: u64, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if value <= i64::MAX as u64 {
            return self.emit_i256_constant(value as i64, block);
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
        self.emit_constant_from_attribute(attribute, block)
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
        self.emit_constant_from_attribute(attribute, block)
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
                OperationBuilder::new(crate::ops::INTTOPTR, self.unknown_location)
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
                OperationBuilder::new(crate::ops::ZEXT, self.unknown_location)
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

    /// Emits a generic two-operand LLVM operation (e.g. `add`, `sub`, `lshr`).
    ///
    /// # Errors
    ///
    /// Returns an error if the operation cannot be constructed.
    ///
    /// # Panics
    ///
    /// Panics if the constructed operation produces no results, indicating a
    /// bug in the caller's operation name.
    pub fn emit_llvm_operation<'block, B>(
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
            .expect("operation always produces one result")
            .into())
    }

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
                OperationBuilder::new(crate::ops::ICMP, self.unknown_location)
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
            .expect("icmp always produces one result")
            .into()
    }

    // ---- Calls ----

    /// Emits an `llvm.call` operation.
    ///
    /// Returns `Some(value)` when `result_types` is non-empty, `None` for void calls.
    ///
    /// # Errors
    ///
    /// Returns an error if the call operation cannot be constructed or
    /// the result cannot be extracted.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_call<'block, B>(
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
            OperationBuilder::new(crate::ops::CALL, self.unknown_location)
                .add_operands(operands)
                .add_attributes(&[
                    (
                        Identifier::new(self.context, "callee"),
                        FlatSymbolRefAttribute::new(self.context, callee).into(),
                    ),
                    (
                        Identifier::new(self.context, "operandSegmentSizes"),
                        DenseI32ArrayAttribute::new(self.context, &[operands.len() as i32, 0])
                            .into(),
                    ),
                    (
                        Identifier::new(self.context, "op_bundle_sizes"),
                        DenseI32ArrayAttribute::new(self.context, &[]).into(),
                    ),
                ])
                .add_results(result_types)
                .build()
                .expect("llvm.call operation is well-formed"),
        );
        if result_types.is_empty() {
            Ok(None)
        } else {
            Ok(Some(operation.result(0)?.into()))
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
        OperationBuilder::new(crate::ops::BR, self.unknown_location)
            .add_operands(destination_arguments)
            .add_attributes(&[(
                Identifier::new(self.context, "operandSegmentSizes"),
                DenseI32ArrayAttribute::new(self.context, &[destination_arguments.len() as i32])
                    .into(),
            )])
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
        OperationBuilder::new(crate::ops::COND_BR, self.unknown_location)
            .add_operands(&operands)
            .add_attributes(&[(
                Identifier::new(self.context, "operandSegmentSizes"),
                DenseI32ArrayAttribute::new(
                    self.context,
                    &[1, true_arguments.len() as i32, false_arguments.len() as i32],
                )
                .into(),
            )])
            .add_successors(&[true_destination, false_destination])
            .build()
            .expect("llvm.cond_br operation is well-formed")
    }
}
