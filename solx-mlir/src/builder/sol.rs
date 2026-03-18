//!
//! Sol dialect operations for the MLIR builder.
//!
//! Contains methods on [`Context`] that emit Sol dialect MLIR
//! operations: contracts, functions, constants, returns, and calls.
//!

use melior::ir::Attribute;
use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Identifier;
use melior::ir::Location;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::FlatSymbolRefAttribute;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::operation::OperationBuilder;
use melior::ir::operation::OperationLike;
use melior::ir::r#type::FunctionType;
use melior::ir::r#type::IntegerType;

use crate::StateMutability;
use crate::builder::Context;

impl<'context> Context<'context> {
    // ---- Structure ----

    /// Emits a `sol.contract` operation with a body region.
    ///
    /// Returns the body block inside the contract region for appending
    /// function definitions.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_contract<'block>(
        &'block self,
        name: &str,
        kind: crate::ContractKind,
        block: &BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let body_region = Region::new();
        let body_block = Block::new(&[]);
        body_region.append_block(body_block);

        let operation = block.append_operation(
            OperationBuilder::new(crate::ops::sol::CONTRACT, self.unknown_location)
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
        &'block self,
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
            // orig_fn_type is required when selector is present — the
            // conversion pass uses it for ABI encoding/decoding.
            attributes.push((
                Identifier::new(self.context, "orig_fn_type"),
                TypeAttribute::new(function_type.into()).into(),
            ));
        }

        let operation = block.append_operation(
            OperationBuilder::new(crate::ops::sol::FUNC, self.unknown_location)
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

    // ---- Constants ----

    /// Emits a `sol.constant` producing an `i256` value.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_constant<'block, B>(&self, value: i64, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let attribute = IntegerAttribute::new(self.i256_type, value);
        block
            .append_operation(
                OperationBuilder::new(crate::ops::sol::CONSTANT, self.unknown_location)
                    .add_attributes(&[(Identifier::new(self.context, "value"), attribute.into())])
                    .add_results(&[self.i256_type])
                    .build()
                    .expect("sol.constant operation is well-formed"),
            )
            .result(0)
            .expect("sol.constant always produces one result")
            .into()
    }

    /// Emits a `sol.constant` from a decimal string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string cannot be parsed as an MLIR integer attribute.
    pub fn emit_sol_constant_from_decimal_str<'block, B>(
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
        self.emit_sol_constant_from_parsed_attribute(attribute, block)
    }

    /// Emits a `sol.constant` from a hex string (without `0x` prefix).
    ///
    /// # Errors
    ///
    /// Returns an error if the string cannot be parsed as an MLIR integer attribute.
    pub fn emit_sol_constant_from_hex_str<'block, B>(
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
        self.emit_sol_constant_from_parsed_attribute(attribute, block)
    }

    // ---- Terminators ----

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
            OperationBuilder::new(crate::ops::sol::RETURN, self.unknown_location)
                .add_operands(operands)
                .build()
                .expect("sol.return operation is well-formed"),
        );
    }

    /// Emits a `sol.yield` region terminator.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_yield<'block, B>(&self, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            OperationBuilder::new(crate::ops::sol::YIELD, self.unknown_location)
                .build()
                .expect("sol.yield operation is well-formed"),
        );
    }

    // ---- Arithmetic ----

    /// Emits a generic two-operand Sol arithmetic operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation cannot be constructed.
    ///
    /// # Panics
    ///
    /// Panics if the constructed operation produces no results, indicating a
    /// bug in the caller's operation name.
    pub fn emit_sol_binary_operation<'block, B>(
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
        self.emit_binary_operation(operation_name, lhs, rhs, result_type, block)
    }

    // ---- Memory ----

    /// Emits a `sol.alloca` for a local variable, returning the pointer.
    ///
    /// Returns a `!sol.ptr<i256, Stack>` pointer type.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR type or operation cannot be constructed, indicating
    /// a bug in the builder.
    pub fn emit_sol_alloca<'block, B>(&self, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                OperationBuilder::new(crate::ops::sol::ALLOCA, self.unknown_location)
                    .add_attributes(&[(
                        Identifier::new(self.context, "alloc_type"),
                        TypeAttribute::new(self.i256_type).into(),
                    )])
                    .add_results(&[self.sol_ptr_type])
                    .build()
                    .expect("sol.alloca operation is well-formed"),
            )
            .result(0)
            .expect("sol.alloca always produces one result")
            .into()
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
            OperationBuilder::new(crate::ops::sol::STORE, self.unknown_location)
                .add_operands(&[value, pointer])
                .build()
                .expect("sol.store operation is well-formed"),
        );
    }

    /// Emits a `sol.load` from a pointer.
    ///
    /// # Errors
    ///
    /// Returns an error if the load operation result cannot be extracted.
    pub fn emit_sol_load<'block, B>(
        &self,
        pointer: Value<'context, 'block>,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Ok(block
            .append_operation(
                OperationBuilder::new(crate::ops::sol::LOAD, self.unknown_location)
                    .add_operands(&[pointer])
                    .add_results(&[self.i256_type])
                    .build()
                    .expect("sol.load operation is well-formed"),
            )
            .result(0)?
            .into())
    }

    // ---- Calls ----

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
            OperationBuilder::new(crate::ops::sol::CALL, self.unknown_location)
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
            Ok(Some(operation.result(0)?.into()))
        }
    }
}
