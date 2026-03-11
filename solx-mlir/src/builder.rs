//!
//! MLIR module builder for EVM code generation.
//!
//! Provides the [`MlirContext`] type that accumulates MLIR operations into a
//! module. Frontends (e.g. `solx-slang`) use this to emit LLVM dialect
//! operations without dealing with raw `melior` API details.
//!

use std::collections::HashMap;

use solx_utils::AddressSpace;

use melior::dialect::llvm;
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

use crate::function_entry::FunctionEntry;

/// MLIR LLVM dialect `llvm.icmp` predicate values.
///
/// Matches the LLVM `ICmpPredicate` encoding used by the MLIR LLVM dialect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum ICmpPredicate {
    /// Equal.
    Eq = 0,
    /// Not equal.
    Ne = 1,
    /// Signed less than.
    Slt = 2,
    /// Signed less than or equal.
    Sle = 3,
    /// Signed greater than.
    Sgt = 4,
    /// Signed greater than or equal.
    Sge = 5,
    /// Unsigned less than.
    Ult = 6,
    /// Unsigned less than or equal.
    Ule = 7,
    /// Unsigned greater than.
    Ugt = 8,
    /// Unsigned greater than or equal.
    Uge = 9,
}

/// Accumulated MLIR state threaded through the AST visitors.
///
/// Owns a `melior::ir::Module` being populated and provides helpers for
/// creating common MLIR types, SSA naming, and function registration.
pub struct MlirContext<'c> {
    /// The MLIR context with all dialects and translations registered.
    context: &'c melior::Context,
    /// The MLIR module being built.
    module: Module<'c>,
    /// Cached `i256` type (MLIR interns types, but avoids repeated lookups).
    i256_type: Type<'c>,
    /// Cached `i1` type.
    i1_type: Type<'c>,
    /// Cached unknown source location.
    unknown_location: Location<'c>,
    /// Registered external/public functions for selector dispatch.
    functions: Vec<FunctionEntry>,
    /// State variable name → storage slot mapping.
    state_variables: HashMap<String, u64>,
    /// All function signatures for call resolution (bare name → overloads).
    function_signatures: HashMap<String, Vec<FunctionSignature>>,
}

/// Function signature info for internal call resolution.
#[derive(Clone)]
struct FunctionSignature {
    /// The mangled MLIR function name.
    mlir_name: String,
    /// Number of parameters.
    param_count: usize,
    /// Whether the function returns a value.
    has_returns: bool,
}

impl<'c> MlirContext<'c> {
    /// Creates a new MLIR state with an empty module.
    pub fn new(context: &'c melior::Context) -> Self {
        let location = Location::unknown(context);
        let module = Module::new(location);

        Self {
            context,
            module,
            i256_type: IntegerType::new(context, 256).into(),
            i1_type: IntegerType::new(context, 1).into(),
            unknown_location: location,
            functions: Vec::new(),
            state_variables: HashMap::new(),
            function_signatures: HashMap::new(),
        }
    }

    /// Returns a reference to the melior context.
    pub fn context(&self) -> &'c melior::Context {
        self.context
    }

    /// Returns the module body block for appending top-level operations.
    pub fn body(&self) -> BlockRef<'c, '_> {
        self.module.body()
    }

    /// Returns an unknown source location.
    pub fn location(&self) -> Location<'c> {
        self.unknown_location
    }

    /// Returns the EVM word type (`i256`).
    pub fn i256(&self) -> Type<'c> {
        self.i256_type
    }

    /// Returns the EVM boolean type (`i1`).
    pub fn i1(&self) -> Type<'c> {
        self.i1_type
    }

    /// Emits an `llvm.mlir.constant` producing an `i256` value.
    ///
    /// Works with both owned `Block` and borrowed `BlockRef`.
    pub fn emit_i256_constant<'b>(
        &self,
        value: i64,
        block: &impl BlockLike<'c, 'b>,
    ) -> Value<'c, 'b>
    where
        'c: 'b,
    {
        let attr = IntegerAttribute::new(self.i256_type, value);
        block
            .append_operation(
                OperationBuilder::new(crate::ops::MLIR_CONSTANT, self.unknown_location)
                    .add_attributes(&[(Identifier::new(self.context, "value"), attr.into())])
                    .add_results(&[self.i256_type])
                    .build()
                    .expect("valid llvm.mlir.constant"),
            )
            .result(0)
            .expect("constant has one result")
            .into()
    }

    /// Emits an `llvm.store` to a pointer.
    ///
    /// # Errors
    ///
    /// Returns an error if the store operation cannot be constructed.
    pub fn emit_store<'b>(
        &self,
        value: Value<'c, 'b>,
        ptr: Value<'c, 'b>,
        block: &impl BlockLike<'c, 'b>,
    ) -> anyhow::Result<()>
    where
        'c: 'b,
    {
        block.append_operation(llvm::store(
            self.context,
            value,
            ptr,
            self.unknown_location,
            llvm::LoadStoreOptions::default(),
        ));
        Ok(())
    }

    /// Emits an `llvm.load` from a pointer.
    ///
    /// # Errors
    ///
    /// Returns an error if the load operation result cannot be extracted.
    pub fn emit_load<'b>(
        &self,
        ptr: Value<'c, 'b>,
        result_type: Type<'c>,
        block: &impl BlockLike<'c, 'b>,
    ) -> anyhow::Result<Value<'c, 'b>>
    where
        'c: 'b,
    {
        Ok(block
            .append_operation(llvm::load(
                self.context,
                ptr,
                result_type,
                self.unknown_location,
                llvm::LoadStoreOptions::default(),
            ))
            .result(0)?
            .into())
    }

    /// Emits an `llvm.inttoptr` cast.
    pub fn emit_inttoptr<'b>(
        &self,
        value: Value<'c, 'b>,
        ptr_type: Type<'c>,
        block: &impl BlockLike<'c, 'b>,
    ) -> Value<'c, 'b>
    where
        'c: 'b,
    {
        block
            .append_operation(
                OperationBuilder::new(crate::ops::INTTOPTR, self.unknown_location)
                    .add_operands(&[value])
                    .add_results(&[ptr_type])
                    .build()
                    .expect("valid llvm.inttoptr"),
            )
            .result(0)
            .expect("inttoptr has one result")
            .into()
    }

    /// Emits a generic two-operand LLVM operation (e.g. `add`, `sub`, `lshr`).
    pub fn emit_llvm_op<'b>(
        &self,
        op_name: &str,
        lhs: Value<'c, 'b>,
        rhs: Value<'c, 'b>,
        result_type: Type<'c>,
        block: &impl BlockLike<'c, 'b>,
    ) -> Value<'c, 'b>
    where
        'c: 'b,
    {
        block
            .append_operation(
                OperationBuilder::new(op_name, self.unknown_location)
                    .add_operands(&[lhs, rhs])
                    .add_results(&[result_type])
                    .build()
                    .unwrap_or_else(|error| panic!("{op_name}: {error}")),
            )
            .result(0)
            .expect("op has one result")
            .into()
    }

    /// Emits an `llvm.icmp` comparison returning `i1`.
    pub fn emit_icmp<'b>(
        &self,
        lhs: Value<'c, 'b>,
        rhs: Value<'c, 'b>,
        predicate: ICmpPredicate,
        block: &impl BlockLike<'c, 'b>,
    ) -> Value<'c, 'b>
    where
        'c: 'b,
    {
        block
            .append_operation(
                OperationBuilder::new(crate::ops::ICMP, self.unknown_location)
                    .add_operands(&[lhs, rhs])
                    .add_attributes(&[(
                        Identifier::new(self.context, "predicate"),
                        IntegerAttribute::new(
                            IntegerType::new(self.context, 64).into(),
                            predicate as i64,
                        )
                        .into(),
                    )])
                    .add_results(&[self.i1_type])
                    .build()
                    .expect("valid llvm.icmp"),
            )
            .result(0)
            .expect("icmp has one result")
            .into()
    }

    /// Emits an `llvm.zext` from `i1` to `i256`.
    pub fn emit_zext_to_i256<'b>(
        &self,
        value: Value<'c, 'b>,
        block: &impl BlockLike<'c, 'b>,
    ) -> Value<'c, 'b>
    where
        'c: 'b,
    {
        block
            .append_operation(
                OperationBuilder::new(crate::ops::ZEXT, self.unknown_location)
                    .add_operands(&[value])
                    .add_results(&[self.i256_type])
                    .build()
                    .expect("valid llvm.zext"),
            )
            .result(0)
            .expect("zext has one result")
            .into()
    }

    /// Emits an `llvm.call` operation.
    ///
    /// Returns `Some(value)` when `result_types` is non-empty, `None` for void calls.
    pub fn emit_call<'b>(
        &self,
        callee: &str,
        operands: &[Value<'c, 'b>],
        result_types: &[Type<'c>],
        block: &impl BlockLike<'c, 'b>,
    ) -> anyhow::Result<Option<Value<'c, 'b>>>
    where
        'c: 'b,
    {
        let num_args = operands.len() as i32;
        let op = block.append_operation(
            OperationBuilder::new(crate::ops::CALL, self.unknown_location)
                .add_operands(operands)
                .add_attributes(&[
                    (
                        Identifier::new(self.context, "callee"),
                        FlatSymbolRefAttribute::new(self.context, callee).into(),
                    ),
                    (
                        Identifier::new(self.context, "operandSegmentSizes"),
                        DenseI32ArrayAttribute::new(self.context, &[num_args, 0]).into(),
                    ),
                    (
                        Identifier::new(self.context, "op_bundle_sizes"),
                        DenseI32ArrayAttribute::new(self.context, &[]).into(),
                    ),
                ])
                .add_results(result_types)
                .build()
                .expect("valid llvm.call"),
        );
        if result_types.is_empty() {
            Ok(None)
        } else {
            Ok(Some(op.result(0)?.into()))
        }
    }

    /// Returns an LLVM pointer type with the given address space.
    pub fn ptr(&self, address_space: AddressSpace) -> Type<'c> {
        llvm::r#type::pointer(self.context, address_space as u32)
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
    pub fn emit_i256_from_u64<'b>(
        &self,
        value: u64,
        block: &impl BlockLike<'c, 'b>,
    ) -> Value<'c, 'b>
    where
        'c: 'b,
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
    pub fn emit_i256_from_decimal_str<'b>(
        &self,
        value: &str,
        block: &impl BlockLike<'c, 'b>,
    ) -> anyhow::Result<Value<'c, 'b>>
    where
        'c: 'b,
    {
        let attr_str = format!("{value} : i256");
        let attr = Attribute::parse(self.context, &attr_str)
            .ok_or_else(|| anyhow::anyhow!("invalid i256 decimal literal: {value}"))?;
        self.emit_constant_from_attr(attr, block)
    }

    /// Emits an `i256` constant from a hex string (without `0x` prefix) of arbitrary size.
    ///
    /// # Errors
    ///
    /// Returns an error if the string cannot be parsed as an MLIR integer attribute.
    pub fn emit_i256_from_hex_str<'b>(
        &self,
        hex: &str,
        block: &impl BlockLike<'c, 'b>,
    ) -> anyhow::Result<Value<'c, 'b>>
    where
        'c: 'b,
    {
        let attr_str = format!("0x{hex} : i256");
        let attr = Attribute::parse(self.context, &attr_str)
            .ok_or_else(|| anyhow::anyhow!("invalid i256 hex literal: 0x{hex}"))?;
        self.emit_constant_from_attr(attr, block)
    }

    /// Emits an `llvm.mlir.constant` from a pre-built attribute.
    fn emit_constant_from_attr<'b>(
        &self,
        attr: Attribute<'c>,
        block: &impl BlockLike<'c, 'b>,
    ) -> anyhow::Result<Value<'c, 'b>>
    where
        'c: 'b,
    {
        Ok(block
            .append_operation(
                OperationBuilder::new(crate::ops::MLIR_CONSTANT, self.unknown_location)
                    .add_attributes(&[(Identifier::new(self.context, "value"), attr)])
                    .add_results(&[self.i256_type])
                    .build()
                    .expect("valid llvm.mlir.constant"),
            )
            .result(0)?
            .into())
    }

    /// Emits an `i256` constant from 32-bit limbs in little-endian order.
    ///
    /// Each limb is at most `u32::MAX`, so it fits in a positive `i64` without
    /// sign-extension issues.
    fn emit_i256_from_limbs<'b>(
        &self,
        limbs: &[u32],
        block: &impl BlockLike<'c, 'b>,
    ) -> Value<'c, 'b>
    where
        'c: 'b,
    {
        debug_assert!(!limbs.is_empty() && limbs.len() <= 8);
        let mut result = self.emit_i256_constant(limbs[0] as i64, block);
        for (i, &limb) in limbs.iter().enumerate().skip(1) {
            if limb == 0 {
                continue;
            }
            let limb_val = self.emit_i256_constant(limb as i64, block);
            let shift = self.emit_i256_constant((i * 32) as i64, block);
            let shifted =
                self.emit_llvm_op(crate::ops::SHL, limb_val, shift, self.i256_type, block);
            result = self.emit_llvm_op(crate::ops::OR, result, shifted, self.i256_type, block);
        }
        result
    }

    /// Registers a function signature for call resolution.
    pub fn register_function_signature(
        &mut self,
        bare_name: &str,
        mlir_name: String,
        param_count: usize,
        has_returns: bool,
    ) {
        self.function_signatures
            .entry(bare_name.to_owned())
            .or_default()
            .push(FunctionSignature {
                mlir_name,
                param_count,
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
            .filter(|s| s.param_count == arg_count)
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
    pub fn llvm_br<'b>(
        &self,
        dest: &melior::ir::Block<'c>,
        dest_args: &[Value<'c, 'b>],
    ) -> Operation<'c> {
        let num_args = dest_args.len() as i32;
        OperationBuilder::new(crate::ops::BR, self.unknown_location)
            .add_operands(dest_args)
            .add_attributes(&[(
                Identifier::new(self.context, "operandSegmentSizes"),
                DenseI32ArrayAttribute::new(self.context, &[num_args]).into(),
            )])
            .add_successors(&[dest])
            .build()
            .expect("valid llvm.br")
    }

    /// Builds an `llvm.cond_br` operation (conditional branch).
    pub fn llvm_cond_br<'b>(
        &self,
        condition: Value<'c, 'b>,
        true_dest: &melior::ir::Block<'c>,
        false_dest: &melior::ir::Block<'c>,
        true_args: &[Value<'c, 'b>],
        false_args: &[Value<'c, 'b>],
    ) -> Operation<'c> {
        let mut operands = vec![condition];
        operands.extend_from_slice(true_args);
        operands.extend_from_slice(false_args);
        let n_true = true_args.len() as i32;
        let n_false = false_args.len() as i32;
        OperationBuilder::new(crate::ops::COND_BR, self.unknown_location)
            .add_operands(&operands)
            .add_attributes(&[(
                Identifier::new(self.context, "operandSegmentSizes"),
                DenseI32ArrayAttribute::new(self.context, &[1, n_true, n_false]).into(),
            )])
            .add_successors(&[true_dest, false_dest])
            .build()
            .expect("valid llvm.cond_br")
    }

    /// Consumes the state and returns the MLIR module as text.
    ///
    /// Verifies the module before serialization. The text is passed to
    /// `solx_core::project::contract::ir::mlir::MLIR` which feeds it
    /// through the existing MLIR → LLVM IR pipeline.
    pub fn into_mlir_source(self) -> anyhow::Result<String> {
        let text = self.module.as_operation().to_string();
        if !self.module.as_operation().verify() {
            anyhow::bail!("generated MLIR module failed verification:\n{text}");
        }

        Ok(text)
    }
}
