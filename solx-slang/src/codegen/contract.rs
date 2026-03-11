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

use slang_solidity::backend::ir::ir2_flat_contracts::ContractDefinition;
use slang_solidity::backend::ir::ir2_flat_contracts::ContractMember;
use slang_solidity::backend::ir::ir2_flat_contracts::FunctionVisibility;
use slang_solidity::backend::ir::ir2_flat_contracts::StateVariableVisibility;

use solx_mlir::FunctionEntry;
use solx_mlir::ICmpPredicate;
use solx_mlir::ops;
use solx_utils::AddressSpace;

use crate::codegen::MlirContext;
use crate::codegen::function::FunctionEmitter;
use crate::codegen::selector::SelectorComputer;
use crate::codegen::types::TypeMapper;

/// Lowers a Solidity contract to MLIR, including function definitions
/// and the `@__entry` selector dispatch.
pub struct ContractEmitter<'a, 'c> {
    /// The shared MLIR context.
    state: &'a mut MlirContext<'c>,
}

impl<'a, 'c> ContractEmitter<'a, 'c> {
    /// Creates a new contract emitter.
    pub fn new(state: &'a mut MlirContext<'c>) -> Self {
        Self { state }
    }

    /// Emits all external/public functions and the `@__entry` dispatcher.
    ///
    /// # Errors
    ///
    /// Returns an error if any function body contains unsupported constructs.
    pub fn emit(&mut self, contract: &ContractDefinition) -> anyhow::Result<()> {
        self.emit_intrinsic_declarations();
        self.register_state_variables(contract);
        self.pre_register_functions(contract);
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

        let no_args_i256 = llvm::r#type::function(i256, &[], false);
        let intrinsics = [
            (ops::EVM_RETURN, llvm::r#type::function(void, &[heap_ptr, i256], false)),
            (ops::EVM_REVERT, llvm::r#type::function(void, &[heap_ptr, i256], false)),
            (
                ops::EVM_CALLDATALOAD,
                llvm::r#type::function(i256, &[self.state.ptr(AddressSpace::Calldata)], false),
            ),
            (ops::EVM_ORIGIN, no_args_i256),
            (ops::EVM_GASPRICE, no_args_i256),
            (ops::EVM_CALLER, no_args_i256),
            (ops::EVM_CALLVALUE, no_args_i256),
            (ops::EVM_TIMESTAMP, no_args_i256),
            (ops::EVM_NUMBER, no_args_i256),
            (ops::EVM_COINBASE, no_args_i256),
            (ops::EVM_CHAINID, no_args_i256),
            (ops::EVM_BASEFEE, no_args_i256),
            (ops::EVM_GASLIMIT, no_args_i256),
            (
                ops::EVM_CALL,
                llvm::r#type::function(
                    i256,
                    &[i256, i256, i256, heap_ptr, i256, heap_ptr, i256],
                    false,
                ),
            ),
        ];

        for (name, func_type) in intrinsics {
            let region = Region::new();
            let func_op = llvm::func(
                context,
                StringAttribute::new(context, name),
                TypeAttribute::new(func_type),
                region,
                &[],
                location,
            );
            self.state.body().append_operation(func_op);
        }
    }

    /// Registers state variables with sequential storage slot assignments.
    fn register_state_variables(&mut self, contract: &ContractDefinition) {
        let mut slot = 0u64;
        for member in &contract.members {
            if let ContractMember::StateVariableDefinition(var) = member {
                self.state
                    .register_state_variable(var.name.text.to_string(), slot);
                slot += 1;
            }
        }
    }

    /// Pre-registers all function signatures for call resolution before bodies are emitted.
    ///
    /// This enables forward references: function A can call function B even if B
    /// is defined after A in the source.
    fn pre_register_functions(&mut self, contract: &ContractDefinition) {
        for member in &contract.members {
            let ContractMember::FunctionDefinition(func) = member else {
                continue;
            };
            let name = func
                .name
                .as_ref()
                .map(|t| t.text.as_str())
                .unwrap_or("unnamed");

            let param_types: Vec<String> = func
                .parameters
                .iter()
                .map(|p| TypeMapper::canonical_type(&p.type_name))
                .collect();
            let mlir_name = format!("solx.fn.{name}({})", param_types.join(","));

            let has_returns = func
                .returns
                .as_ref()
                .is_some_and(|r| !r.is_empty());

            self.state.register_function_signature(
                name,
                mlir_name,
                func.parameters.len(),
                has_returns,
            );
        }
    }

    /// Emits getter functions for public state variables.
    ///
    /// Each public state variable `x` at slot `s` gets a function
    /// `@solx.fn.x() -> i256` that loads from storage slot `s`.
    fn emit_state_variable_getters(
        &mut self,
        contract: &ContractDefinition,
    ) -> anyhow::Result<()> {
        let context = self.state.context();
        let location = self.state.location();
        let i256 = self.state.i256();
        let storage_ptr = self.state.ptr(AddressSpace::Storage);

        for member in &contract.members {
            let ContractMember::StateVariableDefinition(var) = member else {
                continue;
            };
            if !matches!(var.visibility, StateVariableVisibility::Public) {
                continue;
            }

            let name = var.name.text.as_str();
            let mlir_name = format!("solx.fn.{name}()");
            let slot = self
                .state
                .state_variable_slot(name)
                .expect("registered above");

            // Build getter function body: sload from storage slot.
            let region = Region::new();
            let entry_block = Block::new(&[]);

            let slot_val = self.state.emit_i256_from_u64(slot, &entry_block);
            let slot_ptr = self.state.emit_inttoptr(slot_val, storage_ptr, &entry_block);
            let loaded = self.state.emit_load(slot_ptr, i256, &entry_block)?;
            entry_block.append_operation(llvm::r#return(Some(loaded), location));

            region.append_block(entry_block);

            let func_type = llvm::r#type::function(i256, &[], false);
            let func_op = llvm::func(
                context,
                StringAttribute::new(context, &mlir_name),
                TypeAttribute::new(func_type),
                region,
                &[],
                location,
            );
            self.state.body().append_operation(func_op);

            // Compute selector for the getter (same as `functionName()` with no args).
            let selector_sig = format!("{name}()");
            let selector = SelectorComputer::selector_from_signature(&selector_sig);

            // Register for dispatch and call resolution.
            self.state.register_function(FunctionEntry::getter(
                mlir_name.clone(),
                selector,
            ));
            self.state
                .register_function_signature(name, mlir_name, 0, true);
        }
        Ok(())
    }

    /// Emits `llvm.func` for each function in the contract.
    ///
    /// All functions are emitted (public, external, internal, private) so that
    /// internal calls work. Only external/public functions are registered for
    /// selector dispatch.
    fn emit_functions(&mut self, contract: &ContractDefinition) -> anyhow::Result<()> {
        for member in &contract.members {
            let ContractMember::FunctionDefinition(func) = member else {
                continue;
            };

            let emitter = FunctionEmitter::new(self.state);
            let mlir_name = emitter.emit(func)?;

            let is_dispatched = matches!(
                func.visibility,
                FunctionVisibility::External | FunctionVisibility::Public
            );
            if is_dispatched {
                let (selector, _signature) = SelectorComputer::compute(func);
                self.state.register_function(FunctionEntry::new(
                    mlir_name,
                    selector,
                    func.parameters.len(),
                    func.returns.as_ref().is_some_and(|r| !r.is_empty()),
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
        let calldata_ptr = self.state.ptr(AddressSpace::Calldata);
        let cd_ptr = self.state.emit_inttoptr(c0, calldata_ptr, &entry_block);
        let raw_selector = self
            .state
            .emit_call(ops::EVM_CALLDATALOAD, &[cd_ptr], &[i256], &entry_block)?
            .expect("calldataload has result");
        let c224 = self.state.emit_i256_constant(224, &entry_block);
        let selector = self.state.emit_llvm_op(ops::LSHR, raw_selector, c224, i256, &entry_block);

        let revert_block = Block::new(&[]);
        let revert_ptr = self.state.emit_inttoptr(c0, heap_ptr, &revert_block);
        let revert_size = self.state.emit_i256_constant(0, &revert_block);
        self.state
            .emit_call(ops::EVM_REVERT, &[revert_ptr, revert_size], &[], &revert_block)?;
        let location = self.state.location();
        revert_block.append_operation(llvm::unreachable(location));

        let mut dispatch_blocks = Vec::new();

        // Build dispatch blocks for each function.
        let cd_ptr_type = self.state.ptr(AddressSpace::Calldata);
        for func_entry in self.state.functions() {
            let dispatch_block = Block::new(&[]);

            // Decode calldata arguments: each parameter is a 32-byte word
            // at offset 4 + 32*i (4 bytes for the selector).
            let mut args = Vec::new();
            for param_idx in 0..func_entry.param_count {
                let offset = (4 + 32 * param_idx) as i64;
                let offset_val = self.state.emit_i256_constant(offset, &dispatch_block);
                let cd_arg_ptr = self.state.emit_inttoptr(offset_val, cd_ptr_type, &dispatch_block);
                let arg = self
                    .state
                    .emit_call(ops::EVM_CALLDATALOAD, &[cd_arg_ptr], &[i256], &dispatch_block)?
                    .expect("calldataload has result");
                args.push(arg);
            }

            if func_entry.has_returns {
                let ret_val = self
                    .state
                    .emit_call(&func_entry.mlir_name, &args, &[i256], &dispatch_block)?
                    .expect("function has return value");
                let store_ptr = self.state.emit_inttoptr(c0, heap_ptr, &dispatch_block);
                self.state.emit_store(ret_val, store_ptr, &dispatch_block)?;
                let c32 = self.state.emit_i256_constant(32, &dispatch_block);
                self.state
                    .emit_call(ops::EVM_RETURN, &[store_ptr, c32], &[], &dispatch_block)?;
            } else {
                self.state
                    .emit_call(&func_entry.mlir_name, &args, &[], &dispatch_block)?;
                let ret_ptr = self.state.emit_inttoptr(c0, heap_ptr, &dispatch_block);
                let c0_size = self.state.emit_i256_constant(0, &dispatch_block);
                self.state
                    .emit_call(ops::EVM_RETURN, &[ret_ptr, c0_size], &[], &dispatch_block)?;
            }
            dispatch_block.append_operation(llvm::unreachable(location));
            dispatch_blocks.push(dispatch_block);
        }

        // Build a chain of check blocks: first comparison goes in entry_block,
        // remaining comparisons each get their own block. Each block has exactly
        // one terminator (cond_br or br).
        let num_functions = self.state.functions().len();
        let mut check_blocks: Vec<Block<'c>> = Vec::new();
        for _ in 1..num_functions {
            check_blocks.push(Block::new(&[]));
        }

        if num_functions == 0 {
            entry_block.append_operation(self.state.llvm_br(&revert_block, &[]));
        } else {
            for (i, func_entry) in self.state.functions().iter().enumerate() {
                let cmp_block: &Block<'c> = if i == 0 {
                    &entry_block
                } else {
                    &check_blocks[i - 1]
                };

                let sel_bytes = func_entry.selector;
                let sel_value = u32::from_be_bytes(sel_bytes) as i64;
                let sel_const = self.state.emit_i256_constant(sel_value, cmp_block);
                let cmp = self.state.emit_icmp(
                    selector,
                    sel_const,
                    ICmpPredicate::Eq,
                    cmp_block,
                );

                let fallthrough: &Block<'c> = if i + 1 < num_functions {
                    &check_blocks[i]
                } else {
                    &revert_block
                };

                cmp_block.append_operation(
                    self.state
                        .llvm_cond_br(cmp, &dispatch_blocks[i], fallthrough, &[], &[]),
                );
            }
        }

        // Assemble region: entry → check blocks → dispatch blocks → revert.
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
        let func_type = llvm::r#type::function(llvm::r#type::void(context), &[], false);
        let entry_func = llvm::func(
            context,
            StringAttribute::new(context, "__entry"),
            TypeAttribute::new(func_type),
            region,
            &[],
            location,
        );
        self.state.body().append_operation(entry_func);

        Ok(())
    }
}
