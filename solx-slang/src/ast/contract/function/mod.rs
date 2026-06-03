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
use self::statement::ModifierBodyCall;
use self::statement::StatementEmitter;

/// One modifier stage's bound parameters: `(name, value, type)` for each
/// parameter the modifier binds before its body (and any `_;` placeholder)
/// runs.
type ModifierStageParams<'context, 'env> = Vec<(String, Value<'context, 'env>, Type<'context>)>;

/// The resolved MLIR signature of a function: its symbol name, parameter and
/// result types, the original parameter count, public selector, mutability,
/// and MLIR kind. In modifier-body mode `mlir_parameter_types` is extended with
/// the result types (the threaded-in shared return values); `parameter_count`
/// always holds the pre-extension count. A shadowed base override carries no
/// `selector` (it shares the most-derived function's dispatch entry).
struct InnerSignature<'context> {
    mlir_name: String,
    mlir_parameter_types: Vec<Type<'context>>,
    result_types: Vec<Type<'context>>,
    parameter_count: usize,
    selector: Option<u32>,
    state_mutability: StateMutability,
    mlir_kind: Option<solx_mlir::FunctionKind>,
}

/// The non-mutable inputs to emitting a modifier-wrapped function body: the
/// function, the resolved signature pieces it needs, and the contract / entry
/// blocks. The mutable emission state (environment, return slots, current
/// block) is threaded as separate `&mut` / by-value parameters.
#[derive(Clone, Copy)]
struct ModifiedBody<'a, 'context, 'block> {
    function: &'a FunctionDefinition,
    mlir_name: &'a str,
    mlir_parameter_types: &'a [Type<'context>],
    result_types: &'a [Type<'context>],
    contract_body: &'a BlockRef<'context, 'block>,
    function_entry_block: &'a BlockRef<'context, 'block>,
}

/// Which form of a function `emit_sol_inner` lowers.
#[derive(Clone, Copy, PartialEq, Eq)]
enum BodyKind {
    /// A normal function emission: public selector and modifier wrapping.
    Function,
    /// The unwrapped body of a modified function, emitted as a separate internal
    /// `sol.func` (the `$body` symbol) — no selector, no modifier wrapping, with
    /// the return values threaded in as trailing parameters.
    ModifierBody,
}

/// Lowers a Solidity function definition to a `sol.func` operation.
pub struct FunctionEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state Context<'context>,
    /// Containing contract, when emitting a contract's functions. `None` for a
    /// library's functions — libraries have no constructor / state variables /
    /// inheritance, so the constructor-only uses of this field never run.
    contract: Option<&'state ContractDefinition>,
    /// State variable node ID to storage slot mapping.
    storage_layout: &'state HashMap<NodeId, (U256, u32, solx_utils::DataLocation)>,
}

impl<'state, 'context> FunctionEmitter<'state, 'context> {
    /// Creates a new function emitter.
    pub fn new(
        state: &'state Context<'context>,
        contract: Option<&'state ContractDefinition>,
        storage_layout: &'state HashMap<NodeId, (U256, u32, solx_utils::DataLocation)>,
    ) -> Self {
        Self {
            state,
            contract,
            storage_layout,
        }
    }

    /// The containing contract; only valid when emitting a contract (not a
    /// library). Constructor / state-variable / base-linearisation code calls
    /// this — none of which runs for a library.
    fn contract(&self) -> &'state ContractDefinition {
        self.contract
            .expect("contract context required (constructor / state-variable emission)")
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
        self.emit_sol_inner(function, None, contract_body, BodyKind::Function)
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
        self.emit_sol_inner(function, Some(symbol), contract_body, BodyKind::Function)
    }

    /// Emits `function`'s lowering.
    ///
    /// [`BodyKind::ModifierBody`] emits just the function body (no selector, no
    /// modifier wrapping) under `symbol_override` — used to materialise the
    /// wrapped body of a modified function as a separate internal `sol.func`
    /// (see the modifier-chain handling below).
    fn emit_sol_inner(
        &self,
        function: &FunctionDefinition,
        symbol_override: Option<&str>,
        contract_body: &BlockRef<'context, '_>,
        body_kind: BodyKind,
    ) -> anyhow::Result<String> {
        let Some(ref body) = function.body() else {
            // Abstract or interface function — no codegen needed.
            return Ok(symbol_override
                .map(str::to_owned)
                .unwrap_or_else(|| Self::mlir_function_name(function)));
        };

        let InnerSignature {
            mlir_name,
            mlir_parameter_types,
            result_types,
            parameter_count,
            selector,
            state_mutability,
            mlir_kind,
        } = self.resolve_inner_signature(function, symbol_override, body_kind);

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
        self.bind_parameters(
            function,
            &mlir_parameter_types,
            &function_entry_block,
            &mut environment,
        )?;

        let mut return_slots = self.init_return_slots(
            function,
            &result_types,
            parameter_count,
            body_kind,
            &function_entry_block,
            &mut environment,
        )?;

        let region = function_entry_block
            .parent_region()
            .expect("entry block belongs to a region");
        let mut current_block = function_entry_block;

        // State variable initializers run at the top of the constructor body,
        // before any user-written statements. The wrapping modified function
        // already runs them, so a `$body` emission must not run them again.
        if matches!(function.kind(), FunctionKind::Constructor) && body_kind == BodyKind::Function {
            let emitter =
                ExpressionEmitter::new(self.state, &environment, self.storage_layout, true);
            current_block = emitter.emit_state_var_initializers(self.contract(), current_block)?;
        }

        // Collect modifier bodies that wrap this function (`function f() onlyOwner {...}`).
        // In modifier-body mode the modifier stages are emitted by the wrapping
        // call; this emission is just the raw body, so collect none.
        let (modifier_stages, modifier_stage_params) = if body_kind == BodyKind::ModifierBody {
            (Vec::new(), Vec::new())
        } else {
            let (stages, params, next_block) =
                self.collect_modifier_stages(function, &environment, current_block)?;
            current_block = next_block;
            (stages, params)
        };

        let mut terminated = false;
        if modifier_stages.is_empty() {
            for statement in body.statements().iter() {
                let mut emitter = StatementEmitter::new(
                    self.state,
                    &mut environment,
                    &region,
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
        } else {
            let frame = ModifiedBody {
                function,
                mlir_name: &mlir_name,
                mlir_parameter_types: &mlir_parameter_types,
                result_types: &result_types,
                contract_body,
                function_entry_block: &function_entry_block,
            };
            match self.emit_modified_body(
                &frame,
                &mut environment,
                &mut return_slots,
                modifier_stages,
                modifier_stage_params,
                current_block,
            )? {
                Some(next) => current_block = next,
                None => terminated = true,
            }
        }

        if !terminated {
            self.emit_default_return(&result_types, &return_slots, &current_block);
        }

        Ok(mlir_name)
    }

    /// Resolves the MLIR signature for `function` — symbol, types, selector,
    /// mutability, and kind. See [`InnerSignature`] for field meanings,
    /// including the modifier-body parameter extension and the shadowed-override
    /// selector suppression.
    fn resolve_inner_signature(
        &self,
        function: &FunctionDefinition,
        symbol_override: Option<&str>,
        body_kind: BodyKind,
    ) -> InnerSignature<'context> {
        let mlir_name = symbol_override
            .map(str::to_owned)
            .unwrap_or_else(|| Self::mlir_function_name(function));

        let (mlir_parameter_types, result_types) =
            TypeConversion::resolve_function_types(function, &self.state.builder);

        // A `$body` emission takes the wrapping function's current return values
        // as extra trailing parameters and threads them back out as its
        // results, so the named return variables are *shared* across the
        // modifier chain and repeated `_;` invocations (matching solc): a
        // modifier-argument side effect (`m(x = 2)`) and accumulation across
        // multiple `_` both survive into the final result.
        let parameter_count = mlir_parameter_types.len();
        let mlir_parameter_types: Vec<Type<'context>> = if body_kind == BodyKind::ModifierBody {
            mlir_parameter_types
                .iter()
                .chain(result_types.iter())
                .copied()
                .collect()
        } else {
            mlir_parameter_types
        };

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

        InnerSignature {
            mlir_name,
            mlir_parameter_types,
            result_types,
            parameter_count,
            selector,
            state_mutability,
            mlir_kind,
        }
    }

    /// Allocates a stack slot for each parameter, stores the incoming argument
    /// value into it, and binds the slot to the parameter name in `environment`.
    fn bind_parameters<'block>(
        &self,
        function: &FunctionDefinition,
        parameter_types: &[Type<'context>],
        entry_block: &BlockRef<'context, 'block>,
        environment: &mut Environment<'context, 'block>,
    ) -> anyhow::Result<()> {
        for (index, parameter) in function.parameters().iter().enumerate() {
            let parameter_name = parameter
                .name()
                .map(|id| id.name())
                .unwrap_or_else(|| "_".to_owned());
            let parameter_type = parameter_types[index];
            let parameter_value: Value<'context, 'block> = entry_block.argument(index)?.into();
            let pointer = self
                .state
                .builder
                .emit_sol_alloca(parameter_type, entry_block);
            self.state
                .builder
                .emit_sol_store(parameter_value, pointer, entry_block);
            environment.define_variable(parameter_name, pointer, parameter_type);
        }
        Ok(())
    }

    /// Allocates and binds a stack slot for each return value. In modifier-body
    /// mode every return (named or not) is initialised from the corresponding
    /// threaded-in block argument; otherwise a named return is zero-initialised
    /// (integers) or pointed at a fresh memory allocation (memory aggregates),
    /// and an unnamed return gets no slot. Returns the per-return slots (`None`
    /// for an unnamed return outside modifier-body mode).
    fn init_return_slots<'block>(
        &self,
        function: &FunctionDefinition,
        result_types: &[Type<'context>],
        parameter_count: usize,
        body_kind: BodyKind,
        entry_block: &BlockRef<'context, 'block>,
        environment: &mut Environment<'context, 'block>,
    ) -> anyhow::Result<Vec<Option<Value<'context, 'block>>>> {
        let mut return_slots: Vec<Option<Value<'context, 'block>>> = Vec::new();
        if let Some(returns) = function.returns() {
            for (index, parameter) in returns.iter().enumerate() {
                let return_type = result_types[index];
                // `$body` emission: every return (named or not) gets a slot
                // initialised from the threaded-in value (the block argument
                // after the regular parameters), so the shared return state is
                // observable and survives an empty body or a partial `_` reach.
                if body_kind == BodyKind::ModifierBody {
                    let pointer = self
                        .state
                        .builder
                        .emit_sol_alloca(return_type, entry_block);
                    let incoming: Value<'context, 'block> =
                        entry_block.argument(parameter_count + index)?.into();
                    self.state
                        .builder
                        .emit_sol_store(incoming, pointer, entry_block);
                    if let Some(identifier) = parameter.name() {
                        environment.define_variable(identifier.name(), pointer, return_type);
                    }
                    return_slots.push(Some(pointer));
                    continue;
                }
                let Some(identifier) = parameter.name() else {
                    return_slots.push(None);
                    continue;
                };
                let pointer = self
                    .state
                    .builder
                    .emit_sol_alloca(return_type, entry_block);
                // TODO: replace with a typed-zero helper covering address, fixed-bytes, and
                // memory-resident types (e.g. `0x60` for empty `string`/`bytes` memory).
                if IntegerType::try_from(return_type).is_ok() {
                    let zero =
                        self.state
                            .builder
                            .emit_sol_constant(0, return_type, entry_block);
                    self.state
                        .builder
                        .emit_sol_store(zero, pointer, entry_block);
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
                        .emit_sol_malloc(return_type, entry_block);
                    self.state
                        .builder
                        .emit_sol_store(allocated, pointer, entry_block);
                }
                // Other non-integer named returns are left uninitialised; tests
                // that assign before reading still work.
                environment.define_variable(identifier.name(), pointer, return_type);
                return_slots.push(Some(pointer));
            }
        }
        Ok(return_slots)
    }

    /// Emits a modifier-wrapped function body. The wrapped body is materialised
    /// as a separate internal `sol.func` (so a `return` resumes the modifier
    /// tail rather than exiting the whole function); the innermost `_;`
    /// placeholder calls it and captures its results into the return slots,
    /// while the modifier stages stay inlined. Returns the block execution
    /// falls through to, or `None` if the chain terminated the block.
    fn emit_modified_body<'a, 'block>(
        &self,
        frame: &ModifiedBody<'a, 'context, 'block>,
        environment: &mut Environment<'context, 'block>,
        return_slots: &mut Vec<Option<Value<'context, 'block>>>,
        modifier_stages: Vec<slang_solidity_v2::ast::Statements>,
        modifier_stage_params: Vec<ModifierStageParams<'context, 'block>>,
        current_block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let ModifiedBody {
            function,
            mlir_name,
            mlir_parameter_types,
            result_types,
            contract_body,
            function_entry_block,
        } = *frame;
        let region = function_entry_block
            .parent_region()
            .expect("entry block belongs to a region");

        let body_symbol = format!("{mlir_name}$body");
        self.emit_sol_inner(function, Some(body_symbol.as_str()), contract_body, BodyKind::ModifierBody)?;

        // Unnamed returns carry no slot, but the body call's results must be
        // captured somewhere the epilogue can read them back — allocate a
        // (zero-initialised) slot for each so `return X` in the body
        // propagates X out, and a never-reached `_` yields the zero default.
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

        let forward_params: Vec<Value<'context, 'block>> = (0..mlir_parameter_types.len())
            .map(|index| function_entry_block.argument(index).map(Into::into))
            .collect::<Result<_, _>>()?;

        let mut emitter = StatementEmitter::new(
            self.state,
            environment,
            &region,
            self.storage_layout,
            result_types,
            return_slots.as_slice(),
        );
        emitter.modifier_stages = modifier_stages;
        emitter.modifier_stage_params = modifier_stage_params;
        emitter.modifier_body_call = Some(ModifierBodyCall {
            symbol: body_symbol,
            forward_params,
            return_slots: return_slots.clone(),
        });
        emitter.emit_modifier_chain(current_block)
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
        let derived_constructor = self.contract().constructor();

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

        // Per-contract constructor scopes, keyed by contract node id. Each holds
        // that contract's constructor parameters (and, while its body is
        // emitted, that body's local variables). Base constructors routinely
        // reuse the same parameter name as the derived contract (diamond tests
        // name them all `newI` / `newK`), so a single flat scope would clobber
        // them — every contract in the hierarchy gets its own scope.
        let mut root_environment = Environment::new();

        // The most-derived contract's parameters come from the deployed
        // `constructor()` arguments.
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
                root_environment.define_variable(parameter_name, pointer, parameter_type);
            }
        }

        // Run all (linearised) state-variable initializers first. State-variable
        // initializers cannot reference constructor parameters, so the choice of
        // scope only matters for the (shared) storage layout.
        let mut current_block = {
            let emitter =
                ExpressionEmitter::new(self.state, &root_environment, self.storage_layout, true);
            emitter.emit_state_var_initializers(self.contract(), entry)?
        };

        let mut scopes: HashMap<NodeId, Environment<'context, '_>> = HashMap::new();
        scopes.insert(self.contract().node_id(), root_environment);

        // Node ids whose constructor parameters are actually available: the
        // most-derived contract (bound from the deployed arguments) and every
        // base whose arguments were successfully evaluated below. A base that
        // takes parameters but whose arguments could not be matched/bound is
        // left out, and its body is skipped during emission rather than run
        // against an empty scope (which would reference unbound parameters).
        let mut bound_scopes: std::collections::HashSet<NodeId> = std::collections::HashSet::new();
        bound_scopes.insert(self.contract().node_id());

        // C3 linearisation, most-derived first (includes the contract itself).
        let mro: Vec<ContractDefinition> = self
            .contract()
            .compute_linearised_bases()
            .into_iter()
            .filter_map(|base| match base {
                slang_solidity_v2::ast::ContractBase::Contract(contract) => Some(contract),
                slang_solidity_v2::ast::ContractBase::Interface(_) => None,
            })
            .collect();

        // Bind every base constructor's parameters by evaluating the argument
        // expressions supplied for it. Walk most-derived first so the scope an
        // argument is evaluated in is already populated: a base's arguments are
        // written by a more-derived contract and may reference that contract's
        // own constructor parameters (diamond `C(newI, newK + 1)` where `newI`
        // is `D`'s parameter; then `A(newI, newK)` where `newI` is `C`'s, itself
        // just bound from `D`'s invocation).
        // Index the authoritative C3 linearisation by node id so a base
        // invocation can be matched back to its linearised contract — and so
        // base scopes key identically to the body-emission walk below, which
        // also iterates `mro`.
        let mro_node_ids: std::collections::HashSet<NodeId> =
            mro.iter().map(|contract| contract.node_id()).collect();

        current_block = self.bind_base_constructor_scopes(
            &mro,
            &mro_node_ids,
            &mut scopes,
            &mut bound_scopes,
            current_block,
        )?;
        self.emit_constructor_bodies(&mro, &mut scopes, &bound_scopes, &entry, current_block)
    }

    /// Binds every base constructor's parameters into per-contract scopes by
    /// evaluating the base-argument expressions (`Base(args)` / `is Base(args)`)
    /// in C3-linearisation order, populating `scopes` and recording in
    /// `bound_scopes` which bases were successfully bound. Returns the block
    /// reached after evaluating all (possibly side-effecting) arguments.
    fn bind_base_constructor_scopes<'block>(
        &self,
        mro: &[ContractDefinition],
        mro_node_ids: &std::collections::HashSet<NodeId>,
        scopes: &mut HashMap<NodeId, Environment<'context, 'block>>,
        bound_scopes: &mut std::collections::HashSet<NodeId>,
        mut current_block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        for contract in mro.iter() {
            // A contract whose constructor takes no externally-supplied
            // parameters (the common case — most inheritance passes no
            // arguments) was never bound by a more-derived contract, so it
            // evaluates its own base-argument expressions in a fresh empty
            // scope. A contract with parameters was already bound above, walking
            // most-derived first, so this leaves that scope untouched.
            scopes.entry(contract.node_id()).or_default();

            // Gather (linearised base contract, argument expressions) for this
            // contract's base invocations: its constructor's modifier-style list
            // (`constructor() Base(args)`) and its inheritance specifiers
            // (`is Base(args)`). Each base is matched back to its linearised
            // entry (see `match_linearised_base`).
            let mut base_argument_specs: Vec<(ContractDefinition, Vec<Expression>)> = Vec::new();
            if let Some(constructor) = contract.constructor() {
                for invocation in constructor.modifier_invocations().iter() {
                    let Some(arguments) = Self::positional_arguments(invocation.arguments()) else {
                        continue;
                    };
                    if let Some(base_contract) =
                        Self::match_linearised_base(&invocation.name(), mro, mro_node_ids)
                    {
                        base_argument_specs.push((base_contract, arguments));
                    }
                }
            }
            for inheritance in contract.inheritance_types().iter() {
                let Some(arguments) = Self::positional_arguments(inheritance.arguments()) else {
                    continue;
                };
                if let Some(base_contract) =
                    Self::match_linearised_base(&inheritance.type_name(), mro, mro_node_ids)
                {
                    base_argument_specs.push((base_contract, arguments));
                }
            }

            // solc evaluates base-constructor arguments in C3-linearisation
            // order (most-derived base first), not source order: `D is A, B, C`
            // with `B(f(2)) C(f(4)) A(f(6))` evaluates C, B, A (4, 2, 6). Sort
            // the specs by each base's MRO index so a side-effecting argument
            // (see modifiers/evaluation_order) runs in the right order. Pure
            // arguments are order-insensitive, so this is invisible to the
            // value-only base-ctor tests.
            base_argument_specs.sort_by_key(|(base, _)| {
                mro.iter()
                    .position(|contract| contract.node_id() == base.node_id())
                    .unwrap_or(usize::MAX)
            });

            // Evaluate the arguments in this contract's scope and build each
            // base's parameter scope. The immutable borrow of the evaluating
            // scope must end before the new scopes are inserted, so collect
            // them first.
            let evaluated: Vec<(NodeId, Environment<'context, '_>)> = {
                let evaluating_scope = scopes
                    .get(&contract.node_id())
                    .expect("scope ensured at the top of this iteration");
                let mut evaluated = Vec::new();
                for (base_contract, arguments) in base_argument_specs {
                    let base_id = base_contract.node_id();
                    // A more-derived contract already supplied this base's
                    // arguments (most-derived wins).
                    if scopes.contains_key(&base_id) {
                        continue;
                    }
                    let Some(base_constructor) = base_contract.constructor() else {
                        continue;
                    };
                    let mut base_environment = Environment::new();
                    for (parameter, argument) in
                        base_constructor.parameters().iter().zip(arguments.iter())
                    {
                        // Evaluate the argument even when the parameter is
                        // unnamed (`constructor(uint)`) — the evaluation may
                        // have side effects (`Base(f(x))`) that must still run,
                        // in base-linearisation order (modifiers/evaluation_order).
                        let (value, next_block) = {
                            let emitter = ExpressionEmitter::new(
                                self.state,
                                evaluating_scope,
                                self.storage_layout,
                                true,
                            );
                            emitter.emit_value(argument, current_block)?
                        };
                        current_block = next_block;
                        let Some(identifier) = parameter.name() else {
                            continue;
                        };
                        let parameter_type = parameter
                            .get_type()
                            .map(|slang_type| {
                                TypeConversion::resolve_slang_type(
                                    &slang_type,
                                    None,
                                    &self.state.builder,
                                )
                            })
                            .unwrap_or_else(|| self.state.builder.types.ui256);
                        let cast = TypeConversion::from_target_type(
                            parameter_type,
                            &self.state.builder,
                        )
                        .emit(value, &self.state.builder, &current_block);
                        let pointer = self
                            .state
                            .builder
                            .emit_sol_alloca(parameter_type, &current_block);
                        self.state.builder.emit_sol_store(cast, pointer, &current_block);
                        base_environment.define_variable(identifier.name(), pointer, parameter_type);
                    }
                    evaluated.push((base_id, base_environment));
                }
                evaluated
            };
            for (base_id, base_environment) in evaluated {
                bound_scopes.insert(base_id);
                scopes.entry(base_id).or_insert(base_environment);
            }
        }
        Ok(current_block)
    }

    /// Emits each (linearised) constructor body base-first, each in its own
    /// parameter scope and dispatching any constructor modifiers, then finishes
    /// the constructor with a `sol.return` unless a body already terminated the
    /// block.
    fn emit_constructor_bodies<'block>(
        &self,
        mro: &[ContractDefinition],
        scopes: &mut HashMap<NodeId, Environment<'context, 'block>>,
        bound_scopes: &std::collections::HashSet<NodeId>,
        entry: &BlockRef<'context, 'block>,
        mut current_block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        // Run constructor bodies base-first (reverse of most-derived order),
        // each in its own parameter scope. A base whose constructor takes no
        // arguments was never bound above, so it gets a fresh empty scope.
        let region = entry.parent_region().expect("entry block has a region");
        let return_types: [Type<'context>; 0] = [];
        let mut terminated = false;
        for contract in mro.iter().rev() {
            let Some(base_constructor) = contract.constructor() else {
                continue;
            };
            let Some(body) = base_constructor.body() else {
                continue;
            };
            // Skip a base whose constructor takes parameters that were never
            // bound (its arguments could not be matched to this hierarchy). Its
            // body would reference unbound parameters; skipping preserves the
            // pre-existing behaviour instead of panicking.
            if !base_constructor.parameters().is_empty()
                && !bound_scopes.contains(&contract.node_id())
            {
                continue;
            }
            let environment = scopes.entry(contract.node_id()).or_default();
            environment.enter_scope();

            // A constructor may carry modifiers (`constructor() mod1`). They are
            // virtually dispatched against the *deployed* contract (`self`), so
            // an overridden modifier runs its most-derived body even while a base
            // constructor executes. Base-constructor invocations in the same
            // list (`Base(args)`) do not resolve to a modifier and are dropped by
            // `collect_modifier_stages` (their arguments were already bound above).
            let (mut modifier_stages, mut modifier_stage_params, next_block) =
                self.collect_modifier_stages(&base_constructor, environment, current_block)?;
            current_block = next_block;

            if modifier_stages.is_empty() {
                for statement in body.statements().iter() {
                    let mut emitter = StatementEmitter::new(
                        self.state,
                        environment,
                        &region,
                        self.storage_layout,
                        &return_types,
                        &[],
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
                // The wrapped constructor body is the final stage: the innermost
                // modifier's `_;` placeholder runs it inline. A constructor has
                // no return value, so — unlike a modified regular function — the
                // body need not be a separate `sol.func` to carry results back.
                // The body stage carries no modifier parameters of its own.
                modifier_stages.push(body.statements());
                modifier_stage_params.push(Vec::new());
                let mut emitter = StatementEmitter::new(
                    self.state,
                    environment,
                    &region,
                    self.storage_layout,
                    &return_types,
                    &[],
                );
                emitter.modifier_stages = modifier_stages;
                emitter.modifier_stage_params = modifier_stage_params;
                match emitter.emit_modifier_chain(current_block)? {
                    Some(next) => current_block = next,
                    None => terminated = true,
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

    /// Matches a base-contract reference — the path in `is Base` or in a base
    /// constructor invocation `Base(args)` — to its entry in the contract's C3
    /// linearisation.
    ///
    /// Resolves the path to a definition first and, if that lands on a contract
    /// in the linearisation, returns that linearised entry. An import-aliased
    /// path (`M.C`) does not resolve to a definition on its own, so it falls
    /// back to matching the final path segment's name against the linearised
    /// contracts (the alias only renames the namespace, not the contract).
    /// Returns the entry from `mro` in either case, so callers key scopes
    /// consistently with the linearisation-driven body walk.
    fn match_linearised_base(
        path: &slang_solidity_v2::ast::IdentifierPath,
        mro: &[ContractDefinition],
        mro_node_ids: &std::collections::HashSet<NodeId>,
    ) -> Option<ContractDefinition> {
        if let Some(slang_solidity_v2::ast::Definition::Contract(base_contract)) =
            path.resolve_to_definition()
            && mro_node_ids.contains(&base_contract.node_id())
        {
            return mro
                .iter()
                .find(|contract| contract.node_id() == base_contract.node_id())
                .cloned();
        }
        let last_segment = path.iter().last()?;
        let name = last_segment.unparse();
        mro.iter()
            .find(|contract| contract.name().unparse() == name)
            .cloned()
    }

    /// Extracts positional base-constructor argument expressions from an
    /// argument-declaration node (`Base(a, b)`), returning `None` when there are
    /// no arguments. Base constructors only accept positional arguments.
    fn positional_arguments(
        arguments: Option<slang_solidity_v2::ast::ArgumentsDeclaration>,
    ) -> Option<Vec<Expression>> {
        match arguments {
            Some(slang_solidity_v2::ast::ArgumentsDeclaration::PositionalArguments(positional)) => {
                let expressions: Vec<Expression> = positional.iter().collect();
                if expressions.is_empty() {
                    None
                } else {
                    Some(expressions)
                }
            }
            _ => None,
        }
    }

    /// Builds the MLIR function name as `{name}({types})`.
    ///
    /// Uses slang's ABI canonical types when available (external functions),
    /// falls back to AST-based type names for internal/private functions.
    /// Resolves a modifier to its most-derived override in the contract's C3
    /// linearisation, by name and requiring a body (mirroring virtual function
    /// dispatch). A `virtual` modifier may be declared abstract in a base and
    /// `override`-n in a derived contract; the lexical resolution of an
    /// invocation picks the base declaration, so re-resolve against the
    /// linearised bases (most-derived first).
    ///
    /// Returns `None` (keep the lexical resolution) when:
    /// - the invocation is qualified (`Base.m`), which explicitly names a
    ///   specific modifier and bypasses virtual dispatch; or
    /// - the resolved modifier is not part of this contract's hierarchy — e.g.
    ///   a library modifier reached through `using L for *`, which must not be
    ///   virtual-dispatched against the using contract's own same-named modifier.
    fn resolve_modifier_override(
        &self,
        invocation: &slang_solidity_v2::ast::ModifierInvocation,
        resolved: &FunctionDefinition,
    ) -> Option<FunctionDefinition> {
        if invocation.name().len() > 1 {
            return None;
        }
        let name = resolved.name()?.unparse().to_owned();
        let resolved_id = resolved.node_id();
        let mro_modifiers: Vec<FunctionDefinition> = self
            .contract()
            .compute_linearised_bases()
            .into_iter()
            .filter_map(|base| match base {
                slang_solidity_v2::ast::ContractBase::Contract(contract) => Some(contract),
                slang_solidity_v2::ast::ContractBase::Interface(_) => None,
            })
            .flat_map(|contract| contract.modifiers())
            .collect();
        if !mro_modifiers
            .iter()
            .any(|modifier| modifier.node_id() == resolved_id)
        {
            return None;
        }
        mro_modifiers.into_iter().find(|modifier| {
            modifier.body().is_some()
                && modifier.name().is_some_and(|n| n.unparse() == name.as_str())
        })
    }

    /// Resolves a namespace-qualified modifier invocation (`M.M.C.m`) to its
    /// modifier definition. Such paths do not resolve to a definition directly
    /// (like qualified base paths — see [`Self::match_linearised_base`]), so the
    /// final path segment (the modifier name) is matched against the contract's
    /// C3-linearised modifiers, preferring the most-derived one with a body.
    ///
    /// Returns `None` when no modifier of that name exists — in particular for a
    /// base-constructor invocation (whose final segment is a contract name), so
    /// the caller correctly leaves it to `emit_constructor`.
    fn resolve_qualified_modifier(
        &self,
        invocation: &slang_solidity_v2::ast::ModifierInvocation,
    ) -> Option<FunctionDefinition> {
        let last_segment = invocation.name().iter().last()?;
        let modifier_name = last_segment.unparse();
        self.contract()
            .compute_linearised_bases()
            .into_iter()
            .filter_map(|base| match base {
                slang_solidity_v2::ast::ContractBase::Contract(contract) => Some(contract),
                slang_solidity_v2::ast::ContractBase::Interface(_) => None,
            })
            .flat_map(|contract| contract.modifiers())
            .find(|modifier| {
                modifier.body().is_some()
                    && modifier.name().is_some_and(|n| n.unparse() == modifier_name)
            })
    }

    /// Resolves the modifier invocations on `function` to their (virtually
    /// dispatched) modifier bodies, evaluating each invocation's arguments in
    /// `environment` (the clean function scope) and storing them into fresh
    /// per-invocation allocas.
    ///
    /// Returns, in source order (outermost modifier first): the modifier-body
    /// statement stages to inline around the wrapped function body, the parallel
    /// per-stage parameter bindings (`(name, pointer, type)`, to be bound in a
    /// scope local to each stage — see [`StatementEmitter::modifier_stage_params`]),
    /// and the block after the argument evaluations.
    ///
    /// Arguments are evaluated against `environment` *without* registering any
    /// modifier parameter, so an argument referencing a name (`mod(x)`) resolves
    /// to the function's variable rather than a sibling modifier's parameter, and
    /// the same modifier applied repeatedly keeps a distinct binding per use.
    ///
    /// A plain modifier resolves directly; a namespace-qualified path
    /// (`M.M.C.m`) resolves via its final segment (see
    /// [`Self::resolve_qualified_modifier`]). Each is then re-dispatched to its
    /// most-derived override (see [`Self::resolve_modifier_override`]).
    /// Invocations that do not resolve to a modifier — notably base-constructor
    /// calls `constructor() Base(args)` — are skipped, so this is safe to call
    /// on a constructor whose invocation list mixes modifiers and base calls.
    ///
    /// # Errors
    ///
    /// Returns an error if a modifier-argument expression cannot be lowered.
    fn collect_modifier_stages<'env>(
        &self,
        function: &FunctionDefinition,
        environment: &Environment<'context, 'env>,
        mut block: BlockRef<'context, 'env>,
    ) -> anyhow::Result<(
        Vec<slang_solidity_v2::ast::Statements>,
        Vec<ModifierStageParams<'context, 'env>>,
        BlockRef<'context, 'env>,
    )> {
        let mut modifier_stages: Vec<slang_solidity_v2::ast::Statements> = Vec::new();
        let mut modifier_params: Vec<ModifierStageParams<'context, 'env>> = Vec::new();
        for invocation in function.modifier_invocations().iter() {
            let resolved_modifier = match invocation.name().resolve_to_definition() {
                Some(slang_solidity_v2::ast::Definition::Modifier(modifier)) => modifier,
                _ => match self.resolve_qualified_modifier(&invocation) {
                    Some(modifier) => modifier,
                    None => continue,
                },
            };
            let modifier_definition = self
                .resolve_modifier_override(&invocation, &resolved_modifier)
                .unwrap_or(resolved_modifier);
            let Some(modifier_body) = modifier_definition.body() else {
                continue;
            };
            let argument_expressions: Vec<Expression> = match invocation.arguments() {
                Some(slang_solidity_v2::ast::ArgumentsDeclaration::PositionalArguments(
                    positional,
                )) => positional.iter().collect(),
                _ => Vec::new(),
            };
            let mut stage_params: ModifierStageParams<'context, 'env> = Vec::new();
            for (parameter, argument) in modifier_definition
                .parameters()
                .iter()
                .zip(argument_expressions)
            {
                // Evaluate the argument even when the parameter is unnamed
                // (`modifier m(uint) {...}`) — the evaluation may have side
                // effects (`m(f(x))`) that must still run.
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

    /// Returns the unique MLIR symbol name for a function — its base name plus a
    /// parenthesised parameter-type list. ABI-callable functions use ABI type
    /// names; internal/private functions use slang's internal signature;
    /// constructor / fallback / receive and untypeable callees fall back to AST
    /// type text. Every definition and call site routes through this, so the
    /// symbol stays consistent.
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
