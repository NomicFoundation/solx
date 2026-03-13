//!
//! Contract definition lowering to MLIR.
//!

use melior::dialect::llvm;
use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;

use slang_solidity::backend::ir::ast::ContractDefinition;
use slang_solidity::backend::ir::ast::ContractMember;
use slang_solidity::backend::ir::ast::FunctionVisibility;
use slang_solidity::backend::ir::ast::StateVariableVisibility;

use solx_mlir::FunctionEntry;
use solx_mlir::ICmpPredicate;
use solx_utils::AddressSpace;

use crate::codegen::MlirContext;
use crate::codegen::function::FunctionEmitter;
use crate::codegen::selector::SelectorComputer;
use crate::codegen::types::TypeMapper;

/// Lowers a Solidity contract to MLIR, including function definitions
/// and the `@__entry` selector dispatch.
pub struct ContractEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state mut MlirContext<'context>,
}

/// Number of bits to right-shift to extract the 4-byte selector from a 256-bit word.
const SELECTOR_SHIFT_BITS: i64 = 224;

/// Size of the function selector in bytes.
const SELECTOR_SIZE_BYTES: i64 = 4;

/// Size of one ABI-encoded word in bytes.
const ABI_WORD_SIZE: i64 = 32;

impl<'state, 'context> ContractEmitter<'state, 'context> {
    /// Creates a new contract emitter.
    pub(crate) fn new(state: &'state mut MlirContext<'context>) -> Self {
        Self { state }
    }

    /// Emits all external/public functions and the `@__entry` dispatcher.
    ///
    /// # Errors
    ///
    /// Returns an error if any function body contains unsupported constructs.
    pub(crate) fn emit(&mut self, contract: &ContractDefinition) -> anyhow::Result<()> {
        self.emit_intrinsic_declarations();
        self.register_state_variables(contract);
        self.pre_register_functions(contract)?;
        self.emit_state_variable_getters(contract)?;
        self.emit_functions(contract)?;
        self.emit_entry()?;
        Ok(())
    }

    /// Declares EVM intrinsic functions used by the generated code.
    fn emit_intrinsic_declarations(&self) {
        let context = self.state.context();
        let location = self.state.location();
        let i256 = self.state.i256();
        let heap_ptr = self.state.ptr(AddressSpace::Heap);
        let void = llvm::r#type::void(context);

        let no_arguments_i256 = llvm::r#type::function(i256, &[], false);
        let intrinsics = [
            (
                solx_mlir::ops::EVM_RETURN,
                llvm::r#type::function(void, &[heap_ptr, i256], false),
            ),
            (
                solx_mlir::ops::EVM_REVERT,
                llvm::r#type::function(void, &[heap_ptr, i256], false),
            ),
            (
                solx_mlir::ops::EVM_CALLDATALOAD,
                llvm::r#type::function(i256, &[self.state.ptr(AddressSpace::Calldata)], false),
            ),
            (solx_mlir::ops::EVM_ORIGIN, no_arguments_i256),
            (solx_mlir::ops::EVM_GASPRICE, no_arguments_i256),
            (solx_mlir::ops::EVM_CALLER, no_arguments_i256),
            (solx_mlir::ops::EVM_CALLVALUE, no_arguments_i256),
            (solx_mlir::ops::EVM_TIMESTAMP, no_arguments_i256),
            (solx_mlir::ops::EVM_NUMBER, no_arguments_i256),
            (solx_mlir::ops::EVM_COINBASE, no_arguments_i256),
            (solx_mlir::ops::EVM_CHAINID, no_arguments_i256),
            (solx_mlir::ops::EVM_BASEFEE, no_arguments_i256),
            (solx_mlir::ops::EVM_GASLIMIT, no_arguments_i256),
            (
                solx_mlir::ops::EVM_CALL,
                llvm::r#type::function(
                    i256,
                    &[i256, i256, i256, heap_ptr, i256, heap_ptr, i256],
                    false,
                ),
            ),
        ];

        for (name, function_type) in intrinsics {
            let region = Region::new();
            let function_operation = llvm::func(
                context,
                StringAttribute::new(context, name),
                TypeAttribute::new(function_type),
                region,
                &[],
                location,
            );
            self.state.body().append_operation(function_operation);
        }
    }

    /// Registers state variables with sequential storage slot assignments.
    fn register_state_variables(&mut self, contract: &ContractDefinition) {
        let mut slot = 0u64;
        for member in contract.members().iter() {
            if let ContractMember::StateVariableDefinition(variable) = member {
                self.state
                    .register_state_variable(variable.name().name(), slot);
                slot += 1;
            }
        }
    }

    /// Pre-registers all function signatures for call resolution before bodies are emitted.
    ///
    /// This enables forward references: function A can call function B even if B
    /// is defined after A in the source.
    fn pre_register_functions(&mut self, contract: &ContractDefinition) -> anyhow::Result<()> {
        for member in contract.members().iter() {
            let ContractMember::FunctionDefinition(function) = member else {
                continue;
            };
            let name = function
                .name()
                .map(|id| id.name())
                .unwrap_or_else(|| "unnamed".to_owned());

            let parameter_types: Vec<String> = function
                .parameters()
                .iter()
                .map(|p| TypeMapper::canonical_type(&p.type_name()))
                .collect::<anyhow::Result<_>>()?;
            let mlir_name = format!("solx.fn.{name}({})", parameter_types.join(","));

            let has_returns = function.returns().is_some_and(|r| !r.is_empty());

            self.state.register_function_signature(
                &name,
                mlir_name,
                parameter_types.len(),
                has_returns,
            );
        }
        Ok(())
    }

    /// Emits getter functions for public state variables.
    ///
    /// Each public state variable `x` at slot `s` gets a function
    /// `@solx.fn.x() -> i256` that loads from storage slot `s`.
    fn emit_state_variable_getters(&mut self, contract: &ContractDefinition) -> anyhow::Result<()> {
        let context = self.state.context();
        let location = self.state.location();
        let i256 = self.state.i256();
        let storage_ptr = self.state.ptr(AddressSpace::Storage);

        for member in contract.members().iter() {
            let ContractMember::StateVariableDefinition(variable) = member else {
                continue;
            };
            if !matches!(variable.visibility(), StateVariableVisibility::Public) {
                continue;
            }

            let name = variable.name().name();
            let mlir_name = format!("solx.fn.{name}()");
            let slot = self.state.state_variable_slot(&name).ok_or_else(|| {
                anyhow::anyhow!("state variable '{name}' has no assigned storage slot")
            })?;

            // Build getter function body: sload from storage slot.
            let region = Region::new();
            let entry_block = Block::new(&[]);

            let slot_value = self.state.emit_i256_from_u64(slot, &entry_block);
            let slot_pointer = self
                .state
                .emit_inttoptr(slot_value, storage_ptr, &entry_block);
            let loaded = self.state.emit_load(slot_pointer, i256, &entry_block)?;
            entry_block.append_operation(llvm::r#return(Some(loaded), location));

            region.append_block(entry_block);

            let function_type = llvm::r#type::function(i256, &[], false);
            let function_operation = llvm::func(
                context,
                StringAttribute::new(context, &mlir_name),
                TypeAttribute::new(function_type),
                region,
                &[],
                location,
            );
            self.state.body().append_operation(function_operation);

            // Compute selector for the getter (same as `functionName()` with no args).
            let selector_signature = format!("{name}()");
            let selector = SelectorComputer::selector_from_signature(&selector_signature);

            // Register for dispatch and call resolution.
            self.state
                .register_function(FunctionEntry::getter(mlir_name.clone(), selector));
            self.state
                .register_function_signature(&name, mlir_name, 0, true);
        }
        Ok(())
    }

    /// Emits `llvm.func` for each function in the contract.
    ///
    /// All functions are emitted (public, external, internal, private) so that
    /// internal calls work. Only external/public functions are registered for
    /// selector dispatch.
    fn emit_functions(&mut self, contract: &ContractDefinition) -> anyhow::Result<()> {
        for member in contract.members().iter() {
            let ContractMember::FunctionDefinition(function) = member else {
                continue;
            };

            let emitter = FunctionEmitter::new(self.state);
            let mlir_name = emitter.emit(&function)?;

            let is_dispatched = matches!(
                function.visibility(),
                FunctionVisibility::External | FunctionVisibility::Public
            );
            if is_dispatched {
                let (selector, _signature) = SelectorComputer::compute(&function)?;
                self.state.register_function(FunctionEntry::new(
                    mlir_name,
                    selector,
                    function.parameters().len(),
                    function.returns().is_some_and(|r| !r.is_empty()),
                ));
            }
        }
        Ok(())
    }

    /// Emits the `@__entry` function with selector dispatch.
    ///
    /// Loads the 4-byte selector from calldata, compares against registered
    /// functions, and dispatches to the matching one via a chain of check
    /// blocks. Falls through to revert for unknown selectors.
    fn emit_entry(&mut self) -> anyhow::Result<()> {
        let i256 = self.state.i256();
        let heap_ptr = self.state.ptr(AddressSpace::Heap);

        let entry_block = Block::new(&[]);

        let c0 = self.state.emit_i256_constant(0, &entry_block);
        let calldata_pointer_type = self.state.ptr(AddressSpace::Calldata);
        let calldata_pointer = self
            .state
            .emit_inttoptr(c0, calldata_pointer_type, &entry_block);
        let raw_selector = self
            .state
            .emit_call(
                solx_mlir::ops::EVM_CALLDATALOAD,
                &[calldata_pointer],
                &[i256],
                &entry_block,
            )?
            .expect("calldataload always produces one result");
        let shift_amount = self
            .state
            .emit_i256_constant(SELECTOR_SHIFT_BITS, &entry_block);
        let selector = self.state.emit_llvm_op(
            solx_mlir::ops::LSHR,
            raw_selector,
            shift_amount,
            i256,
            &entry_block,
        )?;

        let revert_block = Block::new(&[]);
        let revert_ptr = self.state.emit_inttoptr(c0, heap_ptr, &revert_block);
        let revert_size = self.state.emit_i256_constant(0, &revert_block);
        self.state.emit_call(
            solx_mlir::ops::EVM_REVERT,
            &[revert_ptr, revert_size],
            &[],
            &revert_block,
        )?;
        let location = self.state.location();
        revert_block.append_operation(llvm::unreachable(location));

        let mut dispatch_blocks = Vec::new();

        // Build dispatch blocks for each function.
        for function_entry in self.state.functions() {
            let dispatch_block = Block::new(&[]);

            // Decode calldata arguments: each parameter is a 32-byte word
            // at offset 4 + 32*i (4 bytes for the selector).
            let mut arguments = Vec::new();
            for parameter_index in 0..function_entry.parameter_count() {
                let offset = SELECTOR_SIZE_BYTES + ABI_WORD_SIZE * parameter_index as i64;
                let offset_value = self.state.emit_i256_constant(offset, &dispatch_block);
                let calldata_argument_pointer =
                    self.state
                        .emit_inttoptr(offset_value, calldata_pointer_type, &dispatch_block);
                let argument = self
                    .state
                    .emit_call(
                        solx_mlir::ops::EVM_CALLDATALOAD,
                        &[calldata_argument_pointer],
                        &[i256],
                        &dispatch_block,
                    )?
                    .expect("calldataload always produces one result");
                arguments.push(argument);
            }

            if function_entry.has_returns() {
                let return_value = self
                    .state
                    .emit_call(
                        function_entry.mlir_name(),
                        &arguments,
                        &[i256],
                        &dispatch_block,
                    )?
                    .expect("function call always produces one result");
                let store_ptr = self.state.emit_inttoptr(c0, heap_ptr, &dispatch_block);
                self.state
                    .emit_store(return_value, store_ptr, &dispatch_block);
                let c32 = self
                    .state
                    .emit_i256_constant(ABI_WORD_SIZE, &dispatch_block);
                self.state.emit_call(
                    solx_mlir::ops::EVM_RETURN,
                    &[store_ptr, c32],
                    &[],
                    &dispatch_block,
                )?;
            } else {
                self.state.emit_call(
                    function_entry.mlir_name(),
                    &arguments,
                    &[],
                    &dispatch_block,
                )?;
                let return_pointer = self.state.emit_inttoptr(c0, heap_ptr, &dispatch_block);
                let c0_size = self.state.emit_i256_constant(0, &dispatch_block);
                self.state.emit_call(
                    solx_mlir::ops::EVM_RETURN,
                    &[return_pointer, c0_size],
                    &[],
                    &dispatch_block,
                )?;
            }
            dispatch_block.append_operation(llvm::unreachable(location));
            dispatch_blocks.push(dispatch_block);
        }

        // Build a chain of check blocks: first comparison goes in entry_block,
        // remaining comparisons each get their own block. Each block has exactly
        // one terminator (cond_br or br).
        let function_count = self.state.functions().len();
        let mut check_blocks: Vec<Block<'context>> = Vec::new();
        for _ in 1..function_count {
            check_blocks.push(Block::new(&[]));
        }

        if function_count == 0 {
            entry_block.append_operation(self.state.llvm_br(&revert_block, &[]));
        } else {
            for (i, function_entry) in self.state.functions().iter().enumerate() {
                let cmp_block: &Block<'context> = if i == 0 {
                    &entry_block
                } else {
                    &check_blocks[i - 1]
                };

                let selector_bytes = function_entry.selector();
                let selector_value = u32::from_be_bytes(selector_bytes) as i64;
                let selector_constant = self.state.emit_i256_constant(selector_value, cmp_block);
                let cmp =
                    self.state
                        .emit_icmp(selector, selector_constant, ICmpPredicate::Eq, cmp_block);

                let fallthrough: &Block<'context> = if i + 1 < function_count {
                    &check_blocks[i]
                } else {
                    &revert_block
                };

                cmp_block.append_operation(self.state.llvm_cond_br(
                    cmp,
                    &dispatch_blocks[i],
                    fallthrough,
                    &[],
                    &[],
                ));
            }
        }

        // Assemble region: entry -> check blocks -> dispatch blocks -> revert.
        let region = Region::new();
        region.append_block(entry_block);
        for check_block in check_blocks {
            region.append_block(check_block);
        }
        for dispatch_block in dispatch_blocks {
            region.append_block(dispatch_block);
        }
        region.append_block(revert_block);

        let context = self.state.context();
        let function_type = llvm::r#type::function(llvm::r#type::void(context), &[], false);
        let entry_function = llvm::func(
            context,
            StringAttribute::new(context, "__entry"),
            TypeAttribute::new(function_type),
            region,
            &[],
            location,
        );
        self.state.body().append_operation(entry_function);

        Ok(())
    }
}
