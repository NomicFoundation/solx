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
use slang_solidity_v2::abi::AbiEntry;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::ElementaryType;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Statements;
use slang_solidity_v2::ast::TypeName;

use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::StateMutability;

use self::expression::ExpressionEmitter;
use self::expression::call::type_conversion::TypeConversion;
use self::statement::ModifierBodyCall;
use self::statement::StatementEmitter;
use self::storage_slot::StorageSlot;

/// One modifier stage's bound parameters: `(name, alloca pointer, element type)`
/// for each named parameter the modifier binds, its slot already holding the
/// argument value evaluated in the wrapping function's scope.
type ModifierStageParams<'context, 'block> = Vec<(String, Value<'context, 'block>, Type<'context>)>;

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
        self.emit_sol_inner(function, None, contract_body)
    }

    /// Emits a function under an explicit MLIR symbol, overriding the default
    /// signature-derived name. Used for free (file-level) functions, which are
    /// emitted under a node-id-qualified symbol so two same-name file-level
    /// functions do not collide on one symbol.
    pub fn emit_sol_with_symbol(
        &self,
        function: &FunctionDefinition,
        symbol: &str,
        contract_body: &BlockRef<'context, '_>,
    ) -> anyhow::Result<String> {
        self.emit_sol_inner(function, Some(symbol), contract_body)
    }

    /// Shared body of [`Self::emit_sol`] / [`Self::emit_sol_with_symbol`]: emits
    /// the function as a `sol.func` under `symbol_override` (or its
    /// signature-derived name when `None`), binding parameters and named returns.
    // TODO(rebuild): split when the function-emission domain is rebuilt to the bar.
    #[allow(clippy::too_many_lines)]
    fn emit_sol_inner(
        &self,
        function: &FunctionDefinition,
        symbol_override: Option<&str>,
        contract_body: &BlockRef<'context, '_>,
    ) -> anyhow::Result<String> {
        let mlir_name = symbol_override
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| Self::mlir_function_name(function));
        let Some(ref body) = function.body() else {
            // Abstract or interface function — no codegen needed.
            return Ok(mlir_name);
        };

        let parameters = function.parameters();

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
                    // garbage).
                    let allocated = self
                        .state
                        .builder
                        .emit_sol_malloc_zeroed(return_type, &function_entry_block);
                    self.state
                        .builder
                        .emit_sol_store(allocated, pointer, &function_entry_block);
                }
                // Other non-integer named returns (`address`, `bytesN`, `bool`,
                // `string` / `bytes`) are left uninitialised: a fresh alloca reads
                // back as zero, so an unassigned return yields the zero value.
                environment.define_variable(identifier.name(), pointer, return_type);
                return_slots.push(Some(pointer));
            }
        }

        let mut current_block = function_entry_block;

        // State variable initializers run at the top of the constructor body,
        // before any user-written statements.
        if matches!(function.kind(), FunctionKind::Constructor) {
            let emitter =
                ExpressionEmitter::new(self.state, &environment, self.storage_layout, true);
            current_block = emitter.emit_state_var_initializers(self.contract, current_block)?;
        }

        // A non-constructor function with modifiers (`function f() onlyOwner {…}`)
        // is emitted as a chain of internal `sol.func`s (`f$mod0`, …, `f$body`);
        // `f`'s body is just the call into that chain. Otherwise the body lowers
        // directly here. (Constructor modifiers defer to a later commit.)
        let has_modifiers = !matches!(function.kind(), FunctionKind::Constructor)
            && function.modifier_invocations().iter().next().is_some();
        let mut terminated = false;
        if has_modifiers {
            let (modifier_stages, modifier_stage_params, next_block) =
                self.collect_modifier_stages(function, &environment, current_block)?;
            current_block = next_block;
            match self.emit_modified_body(
                function,
                &mlir_name,
                &mlir_parameter_types,
                &result_types,
                &mut return_slots,
                &function_entry_block,
                modifier_stages,
                modifier_stage_params,
                contract_body,
                current_block,
            )? {
                Some(next) => current_block = next,
                None => terminated = true,
            }
        } else {
            for statement in body.statements().iter() {
                let mut emitter = StatementEmitter::new(
                    self.state,
                    &mut environment,
                    self.storage_layout,
                    &result_types,
                    &return_slots,
                );
                match emitter.emit(&statement, current_block)? {
                    Some(next) => current_block = next,
                    None => {
                        terminated = true;
                        break;
                    }
                }
            }
        }

        if !terminated {
            self.emit_default_return(&result_types, &return_slots, &current_block);
        }

        Ok(mlir_name)
    }

    /// Collects the modifier stages wrapping `function`: each modifier's body
    /// statements and its parameters bound (in `function`'s scope) to fresh
    /// stack slots holding the evaluated argument values, in source order. A
    /// modifier that does not resolve to a local `modifier` definition (an
    /// inherited / overridden one) is skipped for now.
    fn collect_modifier_stages<'block>(
        &self,
        function: &FunctionDefinition,
        environment: &Environment<'context, 'block>,
        mut block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        Vec<Statements>,
        Vec<ModifierStageParams<'context, 'block>>,
        BlockRef<'context, 'block>,
    )> {
        let mut modifier_stages: Vec<Statements> = Vec::new();
        let mut modifier_params: Vec<ModifierStageParams<'context, 'block>> = Vec::new();
        for invocation in function.modifier_invocations().iter() {
            let Some(Definition::Modifier(modifier_definition)) =
                invocation.name().resolve_to_definition()
            else {
                continue;
            };
            let Some(modifier_body) = modifier_definition.body() else {
                continue;
            };
            let argument_expressions: Vec<Expression> = match invocation.arguments() {
                Some(ArgumentsDeclaration::PositionalArguments(positional)) => {
                    positional.iter().collect()
                }
                _ => Vec::new(),
            };
            let mut stage_params: ModifierStageParams<'context, 'block> = Vec::new();
            for (parameter, argument) in modifier_definition
                .parameters()
                .iter()
                .zip(argument_expressions)
            {
                // Evaluate the argument even for an unnamed parameter — the
                // evaluation may have side effects that must still run.
                let (value, next_block) = {
                    let emitter =
                        ExpressionEmitter::new(self.state, environment, self.storage_layout, true);
                    emitter.emit_value(&argument, block)?
                };
                block = next_block;
                let Some(identifier) = parameter.name() else {
                    continue;
                };
                let parameter_type = parameter
                    .get_type()
                    .map(|slang_type| {
                        TypeConversion::resolve_slang_type(&slang_type, None, &self.state.builder)
                    })
                    .unwrap_or_else(|| self.state.builder.types.ui256);
                let cast = TypeConversion::from_target_type(parameter_type, &self.state.builder)
                    .emit(value, &self.state.builder, &block);
                let pointer = self.state.builder.emit_sol_alloca(parameter_type, &block);
                self.state.builder.emit_sol_store(cast, pointer, &block);
                stage_params.push((identifier.name(), pointer, parameter_type));
            }
            modifier_stages.push(modifier_body.statements());
            modifier_params.push(stage_params);
        }
        Ok((modifier_stages, modifier_params, block))
    }

    /// Emits a modified function as a chain of internal `sol.func`s — each
    /// modifier stage (`f$mod0`, …) and the wrapped body (`f$body`) — so a
    /// `return` inside a modifier resumes that stage's `_;` tail rather than
    /// exiting the whole function. `f`'s own body becomes the call into the
    /// chain: it passes `[all modifier arguments ++ f's parameters ++ current
    /// return values]` to the outermost stage and captures the results back into
    /// the shared return slots, which `f`'s epilogue then returns.
    #[allow(clippy::too_many_arguments)]
    fn emit_modified_body<'block>(
        &self,
        function: &FunctionDefinition,
        mlir_name: &str,
        function_parameter_types: &[Type<'context>],
        result_types: &[Type<'context>],
        return_slots: &mut Vec<Option<Value<'context, 'block>>>,
        function_entry_block: &BlockRef<'context, 'block>,
        modifier_stages: Vec<Statements>,
        modifier_stage_params: Vec<ModifierStageParams<'context, 'block>>,
        contract_body: &BlockRef<'context, '_>,
        current_block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        // The wrapped body is the innermost func, reached by the last stage's
        // `_;`; it takes `f`'s parameters plus the threaded-in return values.
        let body_symbol = format!("{mlir_name}$body");
        self.emit_modifier_body_func(
            function,
            &body_symbol,
            function_parameter_types,
            result_types,
            contract_body,
        )?;

        // Every return needs a slot so the chain's results thread through; an
        // unnamed return (no slot yet) gets a fresh zero-initialised one.
        for (index, slot) in return_slots.iter_mut().enumerate() {
            if slot.is_none() {
                let return_type = result_types[index];
                let pointer = self
                    .state
                    .builder
                    .emit_sol_alloca(return_type, function_entry_block);
                if IntegerType::try_from(return_type).is_ok() {
                    let zero =
                        self.state
                            .builder
                            .emit_sol_constant(0, return_type, function_entry_block);
                    self.state
                        .builder
                        .emit_sol_store(zero, pointer, function_entry_block);
                }
                *slot = Some(pointer);
            }
        }

        // `f`'s own parameters, forwarded unchanged down the chain.
        let function_parameters: Vec<Value<'context, 'block>> = (0..function_parameter_types.len())
            .map(|index| function_entry_block.argument(index).map(Into::into))
            .collect::<Result<_, _>>()?;

        // Emit each stage func. Stage `i`'s `_;` calls stage `i + 1` (or `$body`
        // for the last). Stage `i`'s downstream parameters are every later
        // stage's argument types followed by `f`'s parameter types — exactly
        // what the next stage binds.
        let stage_count = modifier_stages.len();
        let stage_symbols: Vec<String> = (0..stage_count)
            .map(|index| format!("{mlir_name}$mod{index}"))
            .collect();
        let stage_argument_types: Vec<Vec<Type<'context>>> = modifier_stage_params
            .iter()
            .map(|params| {
                params
                    .iter()
                    .map(|(_, _, parameter_type)| *parameter_type)
                    .collect()
            })
            .collect();
        for index in 0..stage_count {
            let next_symbol = if index + 1 < stage_count {
                stage_symbols[index + 1].as_str()
            } else {
                body_symbol.as_str()
            };
            let downstream_types: Vec<Type<'context>> = stage_argument_types[index + 1..]
                .iter()
                .flatten()
                .copied()
                .chain(function_parameter_types.iter().copied())
                .collect();
            self.emit_modifier_stage_func(
                stage_symbols[index].as_str(),
                &modifier_stages[index],
                &modifier_stage_params[index],
                &downstream_types,
                result_types,
                next_symbol,
                contract_body,
            )?;
        }

        // `f`'s body: call the outermost stage with [all modifier arguments ++
        // f's parameters ++ current return values], capturing the results back
        // into the shared return slots, then fall through to `f`'s epilogue.
        let mut call_arguments: Vec<Value<'context, 'block>> = Vec::new();
        for params in &modifier_stage_params {
            for (_, pointer, parameter_type) in params {
                call_arguments.push(self.state.builder.emit_sol_load(
                    *pointer,
                    *parameter_type,
                    &current_block,
                )?);
            }
        }
        call_arguments.extend(function_parameters);
        for (slot, &return_type) in return_slots.iter().zip(result_types) {
            if let Some(pointer) = slot {
                call_arguments.push(self.state.builder.emit_sol_load(
                    *pointer,
                    return_type,
                    &current_block,
                )?);
            }
        }
        let results = self.state.builder.emit_sol_call_results(
            stage_symbols[0].as_str(),
            &call_arguments,
            result_types,
            &current_block,
        )?;
        for (slot, value) in return_slots.iter().zip(results) {
            if let Some(pointer) = slot {
                self.state
                    .builder
                    .emit_sol_store(value, *pointer, &current_block);
            }
        }
        Ok(Some(current_block))
    }

    /// Emits the wrapped body of a modified function as an internal `sol.func`
    /// (`f$body`), taking `f`'s parameters plus the threaded-in return values and
    /// returning the (possibly modifier-updated) return values. Named returns are
    /// seeded from the threaded-in trailing arguments.
    fn emit_modifier_body_func(
        &self,
        function: &FunctionDefinition,
        body_symbol: &str,
        function_parameter_types: &[Type<'context>],
        result_types: &[Type<'context>],
        contract_body: &BlockRef<'context, '_>,
    ) -> anyhow::Result<()> {
        let Some(body) = function.body() else {
            return Ok(());
        };
        let mut parameter_types = function_parameter_types.to_vec();
        parameter_types.extend(result_types.iter().copied());
        let entry = self.state.builder.emit_sol_func(
            body_symbol,
            &parameter_types,
            result_types,
            None,
            StateMutability::NonPayable,
            None,
            contract_body,
        );

        let mut environment = Environment::new();
        // Bind `f`'s parameters from the leading arguments.
        for (index, parameter) in function.parameters().iter().enumerate() {
            let parameter_name = parameter
                .name()
                .map(|id| id.name())
                .unwrap_or_else(|| "_".to_owned());
            let parameter_type = function_parameter_types[index];
            let value: Value<'context, '_> = entry.argument(index)?.into();
            let pointer = self.state.builder.emit_sol_alloca(parameter_type, &entry);
            self.state.builder.emit_sol_store(value, pointer, &entry);
            environment.define_variable(parameter_name, pointer, parameter_type);
        }

        // Seed each return slot from the threaded-in trailing argument; bind the
        // named ones so the body reads / writes them.
        let return_offset = function_parameter_types.len();
        let returns: Vec<_> = function
            .returns()
            .map(|parameters| parameters.iter().collect())
            .unwrap_or_default();
        let mut return_slots: Vec<Option<Value<'context, '_>>> =
            Vec::with_capacity(result_types.len());
        for (index, &return_type) in result_types.iter().enumerate() {
            let pointer = self.state.builder.emit_sol_alloca(return_type, &entry);
            let incoming: Value<'context, '_> = entry.argument(return_offset + index)?.into();
            self.state.builder.emit_sol_store(incoming, pointer, &entry);
            if let Some(parameter) = returns.get(index)
                && let Some(identifier) = parameter.name()
            {
                environment.define_variable(identifier.name(), pointer, return_type);
            }
            return_slots.push(Some(pointer));
        }

        let mut emitter = StatementEmitter::new(
            self.state,
            &mut environment,
            self.storage_layout,
            result_types,
            return_slots.as_slice(),
        );
        let mut current_block = entry;
        let mut terminated = false;
        for statement in body.statements().iter() {
            match emitter.emit(&statement, current_block)? {
                Some(next) => current_block = next,
                None => {
                    terminated = true;
                    break;
                }
            }
        }
        if !terminated {
            self.emit_default_return(result_types, return_slots.as_slice(), &current_block);
        }
        Ok(())
    }

    /// Emits one modifier stage as an internal `sol.func`, parameterised by
    /// `[this modifier's arguments ++ downstream values ++ threaded return
    /// values]`. It binds the modifier's parameters, seeds the return slots from
    /// the trailing arguments, and runs the modifier body — whose `_;` calls
    /// `next_symbol` (forwarding the downstream values + current returns) and
    /// whose `return` returns from this frame, resuming the caller's `_;` tail.
    #[allow(clippy::too_many_arguments)]
    fn emit_modifier_stage_func(
        &self,
        stage_symbol: &str,
        modifier_body: &Statements,
        modifier_params: &ModifierStageParams<'context, '_>,
        downstream_types: &[Type<'context>],
        result_types: &[Type<'context>],
        next_symbol: &str,
        contract_body: &BlockRef<'context, '_>,
    ) -> anyhow::Result<()> {
        let parameter_types: Vec<Type<'context>> = modifier_params
            .iter()
            .map(|(_, _, parameter_type)| *parameter_type)
            .chain(downstream_types.iter().copied())
            .chain(result_types.iter().copied())
            .collect();
        let entry = self.state.builder.emit_sol_func(
            stage_symbol,
            &parameter_types,
            result_types,
            None,
            StateMutability::NonPayable,
            None,
            contract_body,
        );

        let mut environment = Environment::new();
        // Bind this modifier's parameters from the leading arguments.
        for (index, (name, _, parameter_type)) in modifier_params.iter().enumerate() {
            let value: Value<'context, '_> = entry.argument(index)?.into();
            let pointer = self.state.builder.emit_sol_alloca(*parameter_type, &entry);
            self.state.builder.emit_sol_store(value, pointer, &entry);
            environment.define_variable(name.clone(), pointer, *parameter_type);
        }

        // Downstream values (later stages' arguments ++ f's parameters) are
        // forwarded verbatim to the next stage at `_;`.
        let downstream_offset = modifier_params.len();
        let forward_params: Vec<Value<'context, '_>> = (0..downstream_types.len())
            .map(|index| entry.argument(downstream_offset + index).map(Into::into))
            .collect::<Result<_, _>>()?;

        // Return slots, seeded from the threaded-in trailing arguments.
        let return_offset = modifier_params.len() + downstream_types.len();
        let mut return_slots: Vec<Option<Value<'context, '_>>> =
            Vec::with_capacity(result_types.len());
        for (index, &return_type) in result_types.iter().enumerate() {
            let pointer = self.state.builder.emit_sol_alloca(return_type, &entry);
            let incoming: Value<'context, '_> = entry.argument(return_offset + index)?.into();
            self.state.builder.emit_sol_store(incoming, pointer, &entry);
            return_slots.push(Some(pointer));
        }

        let modifier_body_call = ModifierBodyCall {
            symbol: next_symbol.to_owned(),
            forward_params,
            return_slots: return_slots.clone(),
        };
        let mut emitter = StatementEmitter::new(
            self.state,
            &mut environment,
            self.storage_layout,
            result_types,
            return_slots.as_slice(),
        )
        .with_modifier_body_call(modifier_body_call);
        let mut current_block = entry;
        let mut terminated = false;
        for statement in modifier_body.iter() {
            match emitter.emit(&statement, current_block)? {
                Some(next) => current_block = next,
                None => {
                    terminated = true;
                    break;
                }
            }
        }
        if !terminated {
            self.emit_default_return(result_types, return_slots.as_slice(), &current_block);
        }
        Ok(())
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
        let emitter = ExpressionEmitter::new(self.state, &environment, self.storage_layout, true);
        let block = emitter.emit_state_var_initializers(self.contract, entry)?;
        self.state.builder.emit_sol_return(&[], &block);
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
                    Some(_) => format!("{base}[]"),
                    None => format!("{base}[]"),
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
