//!
//! Function definition lowering to Sol dialect MLIR.
//!

pub mod expression;
pub mod statement;
pub mod storage_slot;

use std::collections::HashMap;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::r#type::IntegerType;
use ruint::aliases::U256;
use slang_solidity_v2::abi::AbiEntry;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ElementaryType;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::TypeName;

use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::StateMutability;

use self::expression::ExpressionEmitter;
use self::expression::call::type_conversion::TypeConversion;
use self::statement::StatementEmitter;
use self::storage_slot::StorageSlot;

/// Lowers a Solidity function definition to a `sol.func` operation.
pub struct FunctionEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state Context<'context>,
    /// Containing contract.
    contract: &'state ContractDefinition,
    /// State variable node ID to storage slot mapping.
    storage_layout: &'state HashMap<NodeId, (U256, u32, solx_utils::DataLocation)>,
}

impl<'state, 'context> FunctionEmitter<'state, 'context> {
    /// Creates a new function emitter.
    pub fn new(
        state: &'state Context<'context>,
        contract: &'state ContractDefinition,
        storage_layout: &'state HashMap<NodeId, (U256, u32, solx_utils::DataLocation)>,
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
        self.emit_sol_inner(function, None, contract_body)
    }

    /// Emits `function` under the contract-qualified `symbol` with no public
    /// selector. Used for shadowed base overrides reached only through `super`
    /// (e.g. `B`'s `f()` when `D is B` overrides it): they must coexist with
    /// the most-derived `f()` in the same module, so they cannot share its
    /// symbol, and they are internal-only (no dispatch entry).
    pub fn emit_sol_with_symbol(
        &self,
        function: &FunctionDefinition,
        symbol: &str,
        contract_body: &BlockRef<'context, '_>,
    ) -> anyhow::Result<String> {
        self.emit_sol_inner(function, Some(symbol), contract_body)
    }

    fn emit_sol_inner(
        &self,
        function: &FunctionDefinition,
        symbol_override: Option<&str>,
        contract_body: &BlockRef<'context, '_>,
    ) -> anyhow::Result<String> {
        let Some(ref body) = function.body() else {
            // Abstract or interface function — no codegen needed.
            return Ok(symbol_override
                .map(str::to_owned)
                .unwrap_or_else(|| Self::mlir_function_name(function)));
        };

        let parameters = function.parameters();
        let mlir_name = symbol_override
            .map(str::to_owned)
            .unwrap_or_else(|| Self::mlir_function_name(function));

        let (mlir_parameter_types, result_types) =
            TypeConversion::resolve_function_types(function, &self.state.builder);

        // Shadowed base overrides share the most-derived function's selector;
        // emitting it twice would duplicate the dispatch entry. They are only
        // reachable internally through `super`, so they carry no selector.
        let selector = if symbol_override.is_some() {
            None
        } else {
            function.compute_selector()
        };

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
            let parameter_name = parameter
                .name()
                .map(|id| id.name())
                .unwrap_or_else(|| "_".to_owned());
            let parameter_type = mlir_parameter_types[index];
            let parameter_value: Value<'context, '_> = function_entry_block.argument(index)?.into();
            let pointer = self
                .state
                .builder
                .emit_sol_alloca(parameter_type, &function_entry_block);
            self.state
                .builder
                .emit_sol_store(parameter_value, pointer, &function_entry_block);

            environment.define_variable(parameter_name, pointer, parameter_type);
        }

        let mut return_slots: Vec<Option<Value<'context, '_>>> = Vec::new();
        if let Some(returns) = function.returns() {
            for (index, parameter) in returns.iter().enumerate() {
                let Some(identifier) = parameter.name() else {
                    return_slots.push(None);
                    continue;
                };
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
                } else if matches!(
                    parameter.get_type(),
                    Some(
                        slang_solidity_v2::ast::Type::FixedSizeArray(_)
                            | slang_solidity_v2::ast::Type::Struct(_)
                    )
                ) {
                    // A named return of a memory aggregate (`T[n] memory`,
                    // `S memory`) must point at a fresh zero-initialised
                    // allocation, otherwise writes through `result[..]` hit an
                    // uninitialised reference (and `return result` ABI-encodes
                    // garbage). Mirrors variable_declaration's `needs_memory_alloc`.
                    let allocated = self
                        .state
                        .builder
                        .emit_sol_malloc(return_type, &function_entry_block);
                    self.state
                        .builder
                        .emit_sol_store(allocated, pointer, &function_entry_block);
                }
                // Other non-integer named returns are left uninitialised; tests
                // that assign before reading still work.
                environment.define_variable(identifier.name(), pointer, return_type);
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
            let emitter =
                ExpressionEmitter::new(self.state, &environment, self.storage_layout, true);
            current_block = emitter.emit_state_var_initializers(self.contract, current_block)?;
        }

        // Collect modifier bodies that wrap this function (`function f() onlyOwner {...}`).
        // Base-constructor invocations (`constructor() Base(arg)`) also appear
        // here but resolve to a contract, not a modifier — skip those.
        let mut modifier_stages: Vec<slang_solidity_v2::ast::Statements> = Vec::new();
        for invocation in function.modifier_invocations().iter() {
            let Some(slang_solidity_v2::ast::Definition::Modifier(modifier_definition)) =
                invocation.name().resolve_to_definition()
            else {
                continue;
            };
            let Some(modifier_body) = modifier_definition.body() else {
                continue;
            };
            // Bind modifier parameters by evaluating the invocation arguments
            // in the function's entry scope.
            let argument_expressions: Vec<Expression> = match invocation.arguments() {
                Some(slang_solidity_v2::ast::ArgumentsDeclaration::PositionalArguments(
                    positional,
                )) => positional.iter().collect(),
                _ => Vec::new(),
            };
            for (parameter, argument) in modifier_definition
                .parameters()
                .iter()
                .zip(argument_expressions)
            {
                let Some(identifier) = parameter.name() else {
                    continue;
                };
                let parameter_type = parameter
                    .get_type()
                    .map(|slang_type| {
                        TypeConversion::resolve_slang_type(&slang_type, None, &self.state.builder)
                    })
                    .unwrap_or_else(|| self.state.builder.types.ui256);
                let (value, next_block) = {
                    let emitter = ExpressionEmitter::new(
                        self.state,
                        &environment,
                        self.storage_layout,
                        true,
                    );
                    emitter.emit_value(&argument, current_block)?
                };
                current_block = next_block;
                let cast = TypeConversion::from_target_type(parameter_type, &self.state.builder)
                    .emit(value, &self.state.builder, &current_block);
                let pointer = self
                    .state
                    .builder
                    .emit_sol_alloca(parameter_type, &current_block);
                self.state.builder.emit_sol_store(cast, pointer, &current_block);
                environment.define_variable(identifier.name(), pointer, parameter_type);
            }
            modifier_stages.push(modifier_body.statements());
        }

        let mut terminated = false;
        if modifier_stages.is_empty() {
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
        } else {
            // The wrapped function body is the final stage; `_;` placeholders
            // step through `modifier_stages`.
            modifier_stages.push(body.statements());
            let mut emitter = StatementEmitter::new(
                self.state,
                &mut environment,
                &region,
                self.storage_layout,
                &result_types,
            );
            emitter.modifier_stages = modifier_stages;
            match emitter.emit_modifier_chain(current_block)? {
                Some(next) => current_block = next,
                None => terminated = true,
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
        let derived_constructor = self.contract.constructor();

        // The deployed constructor takes the derived contract's constructor
        // parameters (if any).
        let (parameter_types, mutability) = match &derived_constructor {
            Some(constructor) => {
                let (parameter_types, _) =
                    TypeConversion::resolve_function_types(constructor, &self.state.builder);
                (parameter_types, Self::map_state_mutability(constructor))
            }
            None => (Vec::new(), StateMutability::NonPayable),
        };

        let entry = self.state.builder.emit_sol_func(
            "constructor()",
            &parameter_types,
            &[],
            None,
            mutability,
            Some(solx_mlir::FunctionKind::Constructor),
            contract_body,
        );

        let mut environment = Environment::new();

        // Bind the derived constructor's parameters.
        if let Some(constructor) = &derived_constructor {
            for (index, parameter) in constructor.parameters().iter().enumerate() {
                let parameter_name = parameter
                    .name()
                    .map(|id| id.name())
                    .unwrap_or_else(|| "_".to_owned());
                let parameter_type = parameter_types[index];
                let parameter_value: Value<'context, '_> = entry.argument(index)?.into();
                let pointer = self.state.builder.emit_sol_alloca(parameter_type, &entry);
                self.state
                    .builder
                    .emit_sol_store(parameter_value, pointer, &entry);
                environment.define_variable(parameter_name, pointer, parameter_type);
            }
        }

        // Run all (linearised) state-variable initializers first.
        let mut current_block = {
            let emitter =
                ExpressionEmitter::new(self.state, &environment, self.storage_layout, true);
            emitter.emit_state_var_initializers(self.contract, entry)?
        };

        // Run base-contract constructor bodies in C3 order (most-base first),
        // then the derived constructor body. `compute_linearised_bases`
        // returns most-derived first and includes the contract itself, so we
        // reverse to get base-first ordering.
        let region = entry.parent_region().expect("entry block has a region");
        let return_types: [Type<'context>; 0] = [];
        let mut bases = self.contract.compute_linearised_bases();
        bases.reverse();
        let mut terminated = false;
        for base in bases.iter() {
            let slang_solidity_v2::ast::ContractBase::Contract(base_contract) = base else {
                continue;
            };
            let Some(base_constructor) = base_contract.constructor() else {
                continue;
            };
            // The derived constructor's parameters are already bound; base
            // constructors with their own parameters need argument values
            // supplied through inheritance specifiers, which we do not yet
            // thread through — skip running their bodies in that case.
            let is_self = base_contract.node_id() == self.contract.node_id();
            if !is_self && !base_constructor.parameters().is_empty() {
                continue;
            }
            let Some(body) = base_constructor.body() else {
                continue;
            };
            environment.enter_scope();
            for statement in body.statements().iter() {
                let mut emitter = StatementEmitter::new(
                    self.state,
                    &mut environment,
                    &region,
                    self.storage_layout,
                    &return_types,
                );
                match emitter.emit(&statement, current_block)? {
                    Some(next) => current_block = next,
                    None => {
                        terminated = true;
                        break;
                    }
                }
            }
            environment.exit_scope();
            if terminated {
                break;
            }
        }

        if !terminated {
            self.state.builder.emit_sol_return(&[], &current_block);
        }
        Ok(())
    }

    /// Builds the MLIR function name as `{name}({types})`.
    ///
    /// Uses slang's ABI canonical types when available (external functions),
    /// falls back to AST-based type names for internal/private functions.
    pub fn mlir_function_name(function: &FunctionDefinition) -> String {
        let name = Self::mlir_base_name(function);

        if let Some(AbiEntry::Function(abi_function)) = function.compute_abi_entry() {
            let inputs = abi_function.inputs();
            let types: Vec<&str> = inputs.iter().map(|input| input.type_name()).collect();
            return format!("{name}({})", types.join(","));
        }

        // Internal/private functions: prefer slang's internal signature, which uses
        // well-defined internal type names (`type_internal_name`) for parameters that
        // cannot be ABI-encoded. This avoids the AST-text fallback's mangling hazards
        // — qualified user types (`a.b.T` and `c.d.T` both collapsing to `T`) and
        // every `mapping(...)` collapsing to the literal `mapping` — which could
        // alias two distinct internal functions onto a single MLIR symbol. Both the
        // definition (`pre_register_functions` / `emit_sol`) and every call site
        // (`resolve_function` by node id) route through this function, so the symbol
        // stays consistent across the change.
        if let Some(signature) = function.compute_internal_signature() {
            return signature;
        }

        // Fallback for callees slang cannot type, and for constructor / fallback /
        // receive (which have no name, so `compute_internal_signature` is `None`).
        let types: Vec<String> = function
            .parameters()
            .iter()
            .map(|parameter| {
                let type_name = parameter.type_name();
                Self::type_name_text(&type_name)
            })
            .collect();
        format!("{name}({})", types.join(","))
    }

    /// Returns a textual representation of a Solidity type name from the AST.
    fn type_name_text(type_name: &TypeName) -> String {
        match type_name {
            TypeName::ElementaryType(elementary) => Self::elementary_type_text(elementary),
            TypeName::IdentifierPath(path) => path.name(),
            TypeName::ArrayTypeName(array) => {
                let base = Self::type_name_text(&array.operand());
                match array.index() {
                    Some(Expression::DecimalNumberExpression(decimal)) => {
                        format!("{base}[{}]", decimal.literal().unparse())
                    }
                    Some(Expression::HexNumberExpression(hex)) => {
                        format!("{base}[{}]", hex.literal().unparse())
                    }
                    Some(_) | None => format!("{base}[]"),
                }
            }
            TypeName::MappingType(_) => "mapping".to_owned(),
            TypeName::FunctionType(_) => "function".to_owned(),
        }
    }

    /// Returns the text for an elementary type from its AST node.
    fn elementary_type_text(elementary: &ElementaryType) -> String {
        match elementary {
            ElementaryType::AddressType(_) => "address".to_owned(),
            ElementaryType::BoolKeyword(_) => "bool".to_owned(),
            ElementaryType::StringKeyword(_) => "string".to_owned(),
            ElementaryType::UintKeyword(terminal) => terminal.unparse().to_string(),
            ElementaryType::IntKeyword(terminal) => terminal.unparse().to_string(),
            ElementaryType::BytesKeyword(terminal) => terminal.unparse().to_string(),
            ElementaryType::FixedKeyword(terminal) => terminal.unparse().to_string(),
            ElementaryType::UfixedKeyword(terminal) => terminal.unparse().to_string(),
        }
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
        let mut values: Vec<Value<'context, '_>> = Vec::with_capacity(result_types.len());
        for (index, result_type) in result_types.iter().enumerate() {
            let value = match return_slots.get(index).copied().flatten() {
                Some(pointer) => self
                    .state
                    .builder
                    .emit_sol_load(pointer, *result_type, block)
                    .expect("named return slot loads with the declared type"),
                None => self.state.builder.emit_sol_constant(0, *result_type, block),
            };
            values.push(value);
        }
        self.state.builder.emit_sol_return(&values, block);
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
