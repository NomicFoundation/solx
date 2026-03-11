//!
//! MLIR module builder for EVM code generation.
//!
//! Provides the [`MlirContext`] type that accumulates MLIR operations into a
//! module. Frontends (e.g. `solx-slang`) use this to emit LLVM dialect
//! operations without dealing with raw `melior` API details.
//!

use std::collections::HashMap;

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Identifier;
use melior::ir::Location;
use melior::ir::Module;
use melior::ir::Operation;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::DenseI32ArrayAttribute;
use melior::ir::attribute::FlatSymbolRefAttribute;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::operation::OperationBuilder;
use melior::ir::operation::OperationLike;
use melior::ir::r#type::IntegerType;

use solx_utils::AddressSpace;

use crate::ICmpPredicate;
use crate::function_entry::FunctionEntry;

/// Bit width of an EVM word (i256).
const EVM_WORD_BIT_WIDTH: u32 = 256;

/// Bit width for ICmp predicate attributes.
const PREDICATE_ATTRIBUTE_BIT_WIDTH: u32 = 64;

/// Bit width of each limb for wide constant decomposition.
const LIMB_BIT_WIDTH: i64 = 32;

/// Accumulated MLIR state threaded through the AST visitors.
///
/// Owns a `melior::ir::Module` being populated and provides helpers for
/// creating common MLIR types, SSA naming, and function registration.
pub struct MlirContext<'context> {
    /// The MLIR context with all dialects and translations registered.
    context: &'context melior::Context,
    /// The MLIR module being built.
    module: Module<'context>,
    /// Cached `i256` type (MLIR interns types, but avoids repeated lookups).
    i256_type: Type<'context>,
    /// Cached `i1` type.
    i1_type: Type<'context>,
    /// Cached unknown source location.
    unknown_location: Location<'context>,
    /// Registered external/public functions for selector dispatch.
    functions: Vec<FunctionEntry>,
    /// State variable name -> storage slot mapping.
    state_variables: HashMap<String, u64>,
    /// All function signatures for call resolution (bare name -> overloads).
    function_signatures: HashMap<String, Vec<FunctionSignature>>,
}

/// Function signature info for internal call resolution.
#[derive(Clone)]
struct FunctionSignature {
    /// The mangled MLIR function name.
    mlir_name: String,
    /// Number of parameters.
    parameter_count: usize,
    /// Whether the function returns a value.
    has_returns: bool,
}

impl<'context> MlirContext<'context> {
    /// Creates a new MLIR state with an empty module.
    pub fn new(context: &'context melior::Context) -> Self {
        let location = Location::unknown(context);

        Self {
            context,
            module: Module::new(location),
            i256_type: IntegerType::new(context, EVM_WORD_BIT_WIDTH).into(),
            i1_type: IntegerType::new(context, 1).into(),
            unknown_location: location,
            functions: Vec::new(),
            state_variables: HashMap::new(),
            function_signatures: HashMap::new(),
        }
    }

    /// Returns a reference to the melior context.
    pub fn context(&self) -> &'context melior::Context {
        self.context
    }

    /// Returns the module body block for appending top-level operations.
    pub fn body(&self) -> BlockRef<'context, '_> {
        self.module.body()
    }

    /// Returns an unknown source location.
    pub fn location(&self) -> Location<'context> {
        self.unknown_location
    }

    /// Returns the EVM word type (`i256`).
    pub fn i256(&self) -> Type<'context> {
        self.i256_type
    }

    /// Returns the EVM boolean type (`i1`).
    pub fn i1(&self) -> Type<'context> {
        self.i1_type
    }

    /// Emits an `llvm.mlir.constant` producing an `i256` value.
    ///
    /// Works with both owned `Block` and borrowed `BlockRef`.
    pub fn emit_i256_constant<'block, B>(
        &self,
        value: i64,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let attribute = IntegerAttribute::new(self.i256_type, value);
        block
            .append_operation(
                OperationBuilder::new(crate::ops::MLIR_CONSTANT, self.unknown_location)
                    .add_attributes(&[(
                        Identifier::new(self.context, "value"),
                        attribute.into(),
                    )])
                    .add_results(&[self.i256_type])
                    .build()
                    .expect("llvm.mlir.constant operation is well-formed"),
            )
            .result(0)
            .expect("mlir.constant always produces one result")
            .into()
    }

    /// Emits an `llvm.store` to a pointer.
    pub fn emit_store<'block, B>(
        &self,
        value: Value<'context, 'block>,
        ptr: Value<'context, 'block>,
        block: &B,
    ) where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(melior::dialect::llvm::store(
            self.context,
            value,
            ptr,
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
        ptr: Value<'context, 'block>,
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
                ptr,
                result_type,
                self.unknown_location,
                melior::dialect::llvm::LoadStoreOptions::default(),
            ))
            .result(0)?
            .into())
    }

    /// Emits an `llvm.inttoptr` cast.
    pub fn emit_inttoptr<'block, B>(
        &self,
        value: Value<'context, 'block>,
        ptr_type: Type<'context>,
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
                    .add_results(&[ptr_type])
                    .build()
                    .expect("llvm.inttoptr operation is well-formed"),
            )
            .result(0)
            .expect("inttoptr always produces one result")
            .into()
    }

    /// Emits a generic two-operand LLVM operation (e.g. `add`, `sub`, `lshr`).
    ///
    /// # Errors
    ///
    /// Returns an error if the operation cannot be constructed.
    pub fn emit_llvm_op<'block, B>(
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
                            IntegerType::new(self.context, PREDICATE_ATTRIBUTE_BIT_WIDTH).into(),
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

    /// Emits an `llvm.zext` from `i1` to `i256`.
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

    /// Emits an `llvm.call` operation.
    ///
    /// Returns `Some(value)` when `result_types` is non-empty, `None` for void calls.
    ///
    /// # Errors
    ///
    /// Returns an error if the call operation cannot be constructed or
    /// the result cannot be extracted.
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
                        DenseI32ArrayAttribute::new(
                            self.context,
                            &[operands.len() as i32, 0],
                        )
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

    /// Returns an LLVM pointer type with the given address space.
    pub fn ptr(&self, address_space: AddressSpace) -> Type<'context> {
        melior::dialect::llvm::r#type::pointer(self.context, address_space as u32)
    }

    /// Registers a function for entry-point selector dispatch.
    pub fn register_function(&mut self, entry: FunctionEntry) {
        self.functions.push(entry);
    }

    /// Returns the registered functions.
    pub fn functions(&self) -> &[FunctionEntry] {
        &self.functions
    }

    /// Registers a state variable with its storage slot.
    pub fn register_state_variable(&mut self, name: String, slot: u64) {
        self.state_variables.insert(name, slot);
    }

    /// Returns the storage slot for a state variable, if it exists.
    pub fn state_variable_slot(&self, name: &str) -> Option<u64> {
        self.state_variables.get(name).copied()
    }

    /// Emits an `i256` constant from a `u64` value without truncation.
    pub fn emit_i256_from_u64<'block, B>(
        &self,
        value: u64,
        block: &B,
    ) -> Value<'context, 'block>
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
        hex: &str,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let attribute = Attribute::parse(self.context, &format!("0x{hex} : i256"))
            .ok_or_else(|| anyhow::anyhow!("invalid i256 hex literal: 0x{hex}"))?;
        self.emit_constant_from_attribute(attribute, block)
    }

    /// Emits an `llvm.mlir.constant` from a pre-built attribute.
    fn emit_constant_from_attribute<'block, B>(
        &self,
        attribute: Attribute<'context>,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Ok(block
            .append_operation(
                OperationBuilder::new(crate::ops::MLIR_CONSTANT, self.unknown_location)
                    .add_attributes(&[(Identifier::new(self.context, "value"), attribute)])
                    .add_results(&[self.i256_type])
                    .build()
                    .expect("llvm.mlir.constant operation is well-formed"),
            )
            .result(0)?
            .into())
    }

    /// Emits an `i256` constant from 32-bit limbs in little-endian order.
    ///
    /// Each limb is at most `u32::MAX`, so it fits in a positive `i64` without
    /// sign-extension issues.
    fn emit_i256_from_limbs<'block, B>(
        &self,
        limbs: &[u32],
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        debug_assert!(!limbs.is_empty() && limbs.len() <= 8);
        let mut result = self.emit_i256_constant(limbs[0] as i64, block);
        for (i, &limb) in limbs.iter().enumerate().skip(1) {
            if limb == 0 {
                continue;
            }
            let limb_val = self.emit_i256_constant(limb as i64, block);
            let shift = self.emit_i256_constant(i as i64 * LIMB_BIT_WIDTH, block);
            let shifted =
                self.emit_llvm_op(crate::ops::SHL, limb_val, shift, self.i256_type, block)
                    .expect("llvm.shl operation is well-formed");
            result =
                self.emit_llvm_op(crate::ops::OR, result, shifted, self.i256_type, block)
                    .expect("llvm.or operation is well-formed");
        }
        result
    }

    /// Registers a function signature for call resolution.
    pub fn register_function_signature(
        &mut self,
        bare_name: &str,
        mlir_name: String,
        parameter_count: usize,
        has_returns: bool,
    ) {
        self.function_signatures
            .entry(bare_name.to_owned())
            .or_default()
            .push(FunctionSignature {
                mlir_name,
                parameter_count,
                has_returns,
            });
    }

    /// Resolves a function call by bare name and argument count.
    ///
    /// Returns the mangled MLIR name and whether it has return values.
    ///
    /// # Errors
    ///
    /// Returns an error if the function is undefined or the call is ambiguous.
    pub fn resolve_function(
        &self,
        bare_name: &str,
        arg_count: usize,
    ) -> anyhow::Result<(&str, bool)> {
        let sigs = self.function_signatures.get(bare_name).ok_or_else(|| {
            anyhow::anyhow!("undefined function: {bare_name}")
        })?;
        let matches: Vec<_> = sigs
            .iter()
            .filter(|s| s.parameter_count == arg_count)
            .collect();
        match matches.len() {
            0 => anyhow::bail!(
                "no overload of '{bare_name}' takes {arg_count} arguments"
            ),
            1 => Ok((&matches[0].mlir_name, matches[0].has_returns)),
            _ => anyhow::bail!(
                "ambiguous call to overloaded function '{bare_name}'"
            ),
        }
    }

    /// Builds an `llvm.br` operation (unconditional branch).
    pub fn llvm_br<'block>(
        &self,
        dest: &melior::ir::Block<'context>,
        dest_args: &[Value<'context, 'block>],
    ) -> Operation<'context> {
        OperationBuilder::new(crate::ops::BR, self.unknown_location)
            .add_operands(dest_args)
            .add_attributes(&[(
                Identifier::new(self.context, "operandSegmentSizes"),
                DenseI32ArrayAttribute::new(self.context, &[dest_args.len() as i32]).into(),
            )])
            .add_successors(&[dest])
            .build()
            .expect("llvm.br operation is well-formed")
    }

    /// Builds an `llvm.cond_br` operation (conditional branch).
    pub fn llvm_cond_br<'block>(
        &self,
        condition: Value<'context, 'block>,
        true_dest: &melior::ir::Block<'context>,
        false_dest: &melior::ir::Block<'context>,
        true_args: &[Value<'context, 'block>],
        false_args: &[Value<'context, 'block>],
    ) -> Operation<'context> {
        let mut operands = vec![condition];
        operands.extend_from_slice(true_args);
        operands.extend_from_slice(false_args);
        OperationBuilder::new(crate::ops::COND_BR, self.unknown_location)
            .add_operands(&operands)
            .add_attributes(&[(
                Identifier::new(self.context, "operandSegmentSizes"),
                DenseI32ArrayAttribute::new(
                    self.context,
                    &[1, true_args.len() as i32, false_args.len() as i32],
                )
                .into(),
            )])
            .add_successors(&[true_dest, false_dest])
            .build()
            .expect("llvm.cond_br operation is well-formed")
    }

    /// Consumes the state and returns the MLIR module as text.
    ///
    /// Verifies the module before serialization. The text is passed to
    /// `solx_core::project::contract::ir::mlir::MLIR` which feeds it
    /// through the existing MLIR -> LLVM IR pipeline.
    pub fn into_mlir_source(self) -> anyhow::Result<String> {
        let text = self.module.as_operation().to_string();
        if !self.module.as_operation().verify() {
            anyhow::bail!("generated MLIR module failed verification:\n{text}");
        }

        Ok(text)
    }
}
