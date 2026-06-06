//!
//! Function definition lowering to Sol dialect MLIR.
//!

pub mod expression;
pub mod statement;

use std::collections::HashMap;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::abi::AbiEntry;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::StateMutability;

use self::expression::ExpressionEmitter;
use self::expression::arithmetic_mode::ArithmeticMode;
use self::statement::StatementEmitter;
use crate::ast::contract::storage_layout::StorageSlot;
use crate::ast::type_conversion::TypeConversion;

/// Lowers a Solidity function definition to a `sol.func` operation.
pub struct FunctionEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state Context<'context>,
    /// Containing contract.
    contract: &'state ContractDefinition,
    /// State variable node ID to `(slot, byte_offset)` mapping. The byte
    /// offset is zero for unpacked variables and non-zero for variables
    /// packed into a shared slot.
    storage_layout: &'state HashMap<NodeId, StorageSlot>,
}

impl<'state, 'context> FunctionEmitter<'state, 'context> {
    /// Creates a new function emitter.
    pub fn new(
        state: &'state Context<'context>,
        contract: &'state ContractDefinition,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
    ) -> Self {
        Self {
            state,
            contract,
            storage_layout,
        }
    }

    /// Emits a `sol.func` for the given function definition into the given
    /// contract body block.
    ///
    /// # Errors
    ///
    /// Returns an error if the function body contains unsupported statements.
    ///
    /// # Panics
    ///
    /// Panics if an entry block is not attached to a region, which is
    /// unreachable because `emit_sol_func` always creates a region.
    pub fn emit_sol(
        &self,
        function: &FunctionDefinition,
        contract_body: &BlockRef<'context, '_>,
    ) -> anyhow::Result<String> {
        let Some(ref body) = function.body() else {
            // Abstract or interface function — no codegen needed.
            return Ok(Self::mlir_function_name(function));
        };

        let parameters = function.parameters();
        let mlir_name = Self::mlir_function_name(function);

        let (mlir_parameter_types, result_types) =
            TypeConversion::resolve_function_types(function, &self.state.builder);

        let selector = function.compute_selector();

        let state_mutability = Self::map_state_mutability(function);

        let mlir_kind = match function.kind() {
            FunctionKind::Constructor => Some(solx_mlir::FunctionKind::Constructor),
            FunctionKind::Fallback => Some(solx_mlir::FunctionKind::Fallback),
            FunctionKind::Receive => Some(solx_mlir::FunctionKind::Receive),
            FunctionKind::Regular => None,
            FunctionKind::Modifier => unreachable!("modifiers are filtered before emission"),
        };

        let function_entry_block = self.state.builder.emit_sol_func(
            &mlir_name,
            &mlir_parameter_types,
            &result_types,
            selector,
            state_mutability,
            mlir_kind,
            contract_body,
        );

        let mut environment = Environment::new();

        // Create allocas for parameters and bind to environment.
        for (index, parameter) in parameters.iter().enumerate() {
            let parameter_type = mlir_parameter_types[index];
            let parameter_value: Value<'context, '_> = function_entry_block.argument(index)?.into();
            let pointer = self
                .state
                .builder
                .emit_sol_alloca(parameter_type, &function_entry_block);
            self.state
                .builder
                .emit_sol_store(parameter_value, pointer, &function_entry_block);

            environment.define_variable(parameter.node_id(), pointer, parameter_type);
        }

        let mut return_slots: Vec<Option<Value<'context, '_>>> = Vec::new();
        if let Some(returns) = function.returns() {
            for (index, parameter) in returns.iter().enumerate() {
                if parameter.name().is_none() {
                    return_slots.push(None);
                    continue;
                }
                let return_type = result_types[index];
                let pointer = self
                    .state
                    .builder
                    .emit_sol_alloca(return_type, &function_entry_block);
                // TODO: replace with a typed-zero helper covering address, fixed-bytes, and
                // memory-resident types (e.g. `0x60` for empty `string`/`bytes` memory).
                if IntegerType::try_from(return_type).is_ok() {
                    let zero =
                        self.state
                            .builder
                            .emit_sol_constant(0, return_type, &function_entry_block);
                    self.state
                        .builder
                        .emit_sol_store(zero, pointer, &function_entry_block);
                } else {
                    unimplemented!(
                        "zero-initialization for non-integer named return: {return_type}"
                    );
                }
                environment.define_variable(parameter.node_id(), pointer, return_type);
                return_slots.push(Some(pointer));
            }
        }

        let region = function_entry_block
            .parent_region()
            .expect("entry block belongs to a region");
        let mut current_block = function_entry_block;

        // State variable initializers run at the top of the constructor body,
        // before any user-written statements.
        if matches!(function.kind(), FunctionKind::Constructor) {
            let emitter = ExpressionEmitter::new(
                self.state,
                &environment,
                self.storage_layout,
                ArithmeticMode::Checked,
            );
            current_block = emitter.emit_state_var_initializers(self.contract, current_block)?;
        }

        let mut terminated = false;
        for statement in body.statements().iter() {
            let mut emitter = StatementEmitter::new(
                self.state,
                &mut environment,
                &region,
                self.storage_layout,
                &result_types,
            );
            match emitter.emit(&statement, current_block)? {
                Some(next) => current_block = next,
                None => {
                    terminated = true;
                    break;
                }
            }
        }

        if !terminated {
            self.emit_default_return(&result_types, &return_slots, &current_block);
        }

        Ok(mlir_name)
    }

    /// Emits the contract's constructor as a `sol.func`.
    ///
    /// Dispatches to [`Self::emit_sol`] when the contract declares one,
    /// otherwise emits a `constructor()` running just the state-variable
    /// initializers.
    ///
    /// # Errors
    ///
    /// Returns an error if a state-variable initializer has an unresolved
    /// type or contains unsupported constructs, or if the explicit
    /// constructor body contains unsupported statements.
    ///
    /// # Panics
    ///
    /// Panics if an entry block is not attached to a region, which is
    /// unreachable because `emit_sol_func` always creates a region.
    pub fn emit_constructor(&self, contract_body: &BlockRef<'context, '_>) -> anyhow::Result<()> {
        if let Some(constructor) = self.contract.constructor() {
            self.emit_sol(&constructor, contract_body)?;
            return Ok(());
        }
        let entry = self.state.builder.emit_sol_func(
            "constructor()",
            &[],
            &[],
            None,
            StateMutability::NonPayable,
            Some(solx_mlir::FunctionKind::Constructor),
            contract_body,
        );
        let environment = Environment::new();
        let emitter = ExpressionEmitter::new(
            self.state,
            &environment,
            self.storage_layout,
            ArithmeticMode::Checked,
        );
        let block = emitter.emit_state_var_initializers(self.contract, entry)?;
        self.state.builder.emit_sol_return(&[], &block);
        Ok(())
    }

    /// Returns the unique MLIR symbol name for a function.
    ///
    /// Externally-callable functions use slang's canonical ABI signature (a
    /// struct parameter expands to its component tuple, so overloads taking
    /// different structs do not collapse onto one symbol); internal/private
    /// functions use slang's internal signature. Constructor / fallback /
    /// receive have neither — they are not callable by name, so the base name
    /// alone is unique. Every definition and call site routes through this, so
    /// the symbol stays consistent.
    pub fn mlir_function_name(function: &FunctionDefinition) -> String {
        if let Some(AbiEntry::Function(abi_function)) = function.compute_abi_entry() {
            if let Some(signature) = function.compute_canonical_signature() {
                return signature;
            }
            let name = Self::mlir_base_name(function);
            let inputs = abi_function.inputs();
            let types: Vec<&str> = inputs.iter().map(|input| input.type_name()).collect();
            return format!("{name}({})", types.join(","));
        }

        if let Some(signature) = function.compute_internal_signature() {
            return signature;
        }

        format!("{}()", Self::mlir_base_name(function))
    }

    /// Returns the base name for a function's MLIR symbol, using its kind to
    /// generate names for special functions (fallback, receive) that have no
    /// Solidity-level identifier.
    pub fn mlir_base_name(function: &FunctionDefinition) -> String {
        match function.kind() {
            FunctionKind::Regular => function
                .name()
                .expect("regular functions have a name")
                .name(),
            FunctionKind::Fallback => "fallback".to_owned(),
            FunctionKind::Receive => "receive".to_owned(),
            FunctionKind::Constructor => "constructor".to_owned(),
            FunctionKind::Modifier => unreachable!("modifiers are not emitted as functions"),
        }
    }

    /// Emits a default `sol.return` if the block lacks a terminator.
    ///
    /// For each return position, loads the current value from the named-return
    /// slot when one was allocated, otherwise materializes a typed zero
    /// constant.
    fn emit_default_return(
        &self,
        result_types: &[Type<'context>],
        return_slots: &[Option<Value<'context, '_>>],
        block: &BlockRef<'context, '_>,
    ) {
        if block.terminator().is_some() {
            return;
        }
        self.state
            .builder
            .emit_return_from_slots(result_types, return_slots, block);
    }

    /// Maps Slang's `FunctionMutability` to the Sol dialect's `StateMutability`.
    ///
    /// Required because the Sol dialect defines its own mutability enum
    /// independently of the Slang AST representation.
    fn map_state_mutability(function: &FunctionDefinition) -> StateMutability {
        use slang_solidity_v2::ast::FunctionMutability;
        match function.mutability() {
            FunctionMutability::Pure => StateMutability::Pure,
            FunctionMutability::View => StateMutability::View,
            FunctionMutability::Payable => StateMutability::Payable,
            FunctionMutability::NonPayable => StateMutability::NonPayable,
        }
    }
}
