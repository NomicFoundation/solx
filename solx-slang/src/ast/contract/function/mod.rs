//!
//! Function definition lowering to Sol dialect MLIR.
//!

pub mod body_kind;
pub mod expression;
pub mod modifier;
pub mod modifier_body_call;
pub mod modifier_parameter_binding;
pub mod signature;
pub mod statement;

use std::collections::HashMap;
use std::collections::HashSet;

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::abi::AbiEntry;
use slang_solidity_v2::ast::ContractBase;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::StateMutability;
use solx_mlir::ods::sol::MallocOperation;
use solx_mlir::ods::sol::ReturnOperation;
use solx_mlir::ods::sol::StoreOperation;

use self::body_kind::BodyKind;
use self::expression::ExpressionContext;
use self::expression::arithmetic_mode::ArithmeticMode;
use self::modifier::ModifiedBody;
use self::signature::InnerSignature;
use self::statement::StatementContext;
use crate::ast::Emit;
use crate::ast::contract::storage_layout::StorageSlot;
use crate::ast::type_conversion::LocationPolicy;
use crate::ast::type_conversion::ResolveSignature;
use crate::ast::type_conversion::TypeConversion;

/// Lowers a Solidity function definition to a `sol.func` operation.
pub struct FunctionEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state Context<'context>,
    /// Containing contract, when emitting a contract's functions. `None` for a
    /// library's functions — libraries have no constructor / state variables /
    /// inheritance, so the constructor-only uses of this field never run.
    contract: Option<&'state ContractDefinition>,
    /// State variable node ID to `(slot, byte_offset)` mapping. The byte
    /// offset is zero for unpacked variables and non-zero for variables
    /// packed into a shared slot.
    storage_layout: &'state HashMap<NodeId, StorageSlot>,
}

impl<'state, 'context> FunctionEmitter<'state, 'context> {
    /// Creates a new function emitter.
    pub fn new(
        state: &'state Context<'context>,
        contract: Option<&'state ContractDefinition>,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
    ) -> Self {
        Self {
            state,
            contract,
            storage_layout,
        }
    }

    /// Emits a `sol.func` for the given function definition into the given
    /// contract body block, under its canonical (dispatchable) symbol.
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

    /// Emits `function` under an explicit `symbol` with no public selector.
    ///
    /// Used for free and library functions emitted into a contract's module:
    /// they are never dispatched by selector, only resolved by node id, so each
    /// is given a node-id-qualified symbol that cannot collide with a same-named
    /// function reached together.
    ///
    /// # Errors
    ///
    /// Returns an error if the function body contains unsupported statements.
    pub fn emit_sol_with_symbol(
        &self,
        function: &FunctionDefinition,
        symbol: &str,
        contract_body: &BlockRef<'context, '_>,
    ) -> anyhow::Result<String> {
        self.emit_sol_inner(function, Some(symbol), contract_body, BodyKind::Function)
    }

    /// The central function-emission driver: opens the `sol.func`, binds
    /// parameters and return slots, threads the body statements, and closes with
    /// the default return. `symbol_override` names the `sol.func` explicitly (a
    /// free/library function) and suppresses the public selector and special
    /// kind; otherwise the canonical [`Self::mlir_function_name`] and computed
    /// selector are used.
    ///
    /// # Errors
    ///
    /// Returns an error if the function body contains unsupported statements.
    ///
    /// # Panics
    ///
    /// Panics if an entry block is not attached to a region, which is
    /// unreachable because `emit_sol_func` always creates a region.
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
            parameter_count,
            result_types,
            selector,
            state_mutability,
            mlir_kind,
        } = self.resolve_inner_signature(function, symbol_override, body_kind);

        // A regular function (real body, not a constructor/fallback/receive, and
        // not a modifier-stage `$body`) is the target of an internal function
        // pointer, so it carries an `id` — the slang node id, matching solc. This
        // includes free and library functions, which are emitted under a node-id
        // symbol override but are still referenceable (`p(libFn)`); only modifier
        // bodies (`BodyKind::ModifierBody`) and synthetic dispatchers are excluded.
        let function_id = (body_kind == BodyKind::Function && mlir_kind.is_none()).then(|| {
            i64::try_from(usize::from(function.node_id())).expect("a function node id fits in i64")
        });

        let function_entry_block = self.state.builder.emit_sol_func(
            &mlir_name,
            &mlir_parameter_types,
            &result_types,
            selector,
            state_mutability,
            mlir_kind,
            function_id,
            contract_body,
        );

        let mut environment = Environment::new();
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
            let emitter = ExpressionContext::new(
                self.state,
                &environment,
                self.storage_layout,
                ArithmeticMode::Checked,
            );
            current_block = emitter.emit_state_var_initializers(
                self.contract
                    .expect("a constructor is emitted only within a contract"),
                current_block,
            )?;
        }

        // Collect the modifier bodies that wrap this function
        // (`function f() onlyOwner {...}`). In modifier-body mode the stages are
        // emitted by the wrapping call, so this raw `$body` emission collects
        // none.
        let (modifier_stages, modifier_stage_params) = if body_kind == BodyKind::ModifierBody {
            (Vec::new(), Vec::new())
        } else {
            let (stages, params, next_block) =
                self.build_modifier_stages(function, &environment, current_block)?;
            current_block = next_block;
            (stages, params)
        };

        let mut terminated = false;
        if modifier_stages.is_empty() {
            for statement in body.statements().iter() {
                let mut emitter = StatementContext::new(
                    self.state,
                    &mut environment,
                    &region,
                    self.storage_layout,
                    &result_types,
                    &return_slots,
                );
                match statement.emit(&mut emitter, current_block)? {
                    Some(next) => current_block = next,
                    None => {
                        terminated = true;
                        break;
                    }
                }
            }
        } else {
            let frame = ModifiedBody::new(
                function,
                &mlir_name,
                &mlir_parameter_types,
                &result_types,
                contract_body,
                &function_entry_block,
            );
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
            self.emit_default_return(function, &result_types, &return_slots, &current_block);
        }

        Ok(mlir_name)
    }

    /// Resolves the MLIR signature for `function` — symbol, parameter and result
    /// types, selector, mutability, and kind (see [`InnerSignature`]). A
    /// symbol-override emission (a free/library function) or a modifier body
    /// carries no public selector or special function kind.
    fn resolve_inner_signature(
        &self,
        function: &FunctionDefinition,
        symbol_override: Option<&str>,
        body_kind: BodyKind,
    ) -> InnerSignature<'context> {
        let mlir_name = symbol_override
            .map(str::to_owned)
            .unwrap_or_else(|| Self::mlir_function_name(function));

        let (mut mlir_parameter_types, result_types) =
            function.resolve_signature_types(LocationPolicy::Declared(None), &self.state.builder);

        // The function's own parameters, recorded before the modifier-body
        // extension below.
        let parameter_count = mlir_parameter_types.len();

        // A modifier body (`$body`) receives the wrapping function's return
        // values as trailing parameters, so its return slots can be seeded from
        // the body call and observed by the modifier tail and epilogue.
        if body_kind == BodyKind::ModifierBody {
            mlir_parameter_types.extend(result_types.iter().copied());
        }

        let state_mutability = Self::map_state_mutability(function);

        let (selector, mlir_kind) = match (symbol_override, body_kind) {
            (None, BodyKind::Function) => {
                let mlir_kind = match function.kind() {
                    FunctionKind::Constructor => Some(solx_mlir::FunctionKind::Constructor),
                    FunctionKind::Fallback => Some(solx_mlir::FunctionKind::Fallback),
                    FunctionKind::Receive => Some(solx_mlir::FunctionKind::Receive),
                    FunctionKind::Regular => None,
                    FunctionKind::Modifier => {
                        unreachable!("modifiers are filtered before emission")
                    }
                };
                (function.compute_selector(), mlir_kind)
            }
            _ => (None, None),
        };

        InnerSignature {
            mlir_name,
            mlir_parameter_types,
            parameter_count,
            result_types,
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
            let parameter_type = parameter_types[index];
            let parameter_value: Value<'context, 'block> = entry_block.argument(index)?.into();
            let pointer = crate::ast::Pointer::stack_slot(
                crate::ast::Type::new(parameter_type),
                &self.state.builder,
                entry_block,
            )
            .into_mlir();
            sol_op_void!(
                &self.state.builder,
                entry_block,
                StoreOperation.val(parameter_value).addr(pointer)
            );
            environment.define_variable(parameter.node_id(), pointer, parameter_type);
        }
        Ok(())
    }

    /// Allocates and binds a stack slot for each named return value (integers
    /// zero-initialised), and pushes `None` for an unnamed return. Returns the
    /// per-return slots, parallel to the declared returns.
    fn init_return_slots<'block>(
        &self,
        function: &FunctionDefinition,
        result_types: &[Type<'context>],
        parameter_count: usize,
        body_kind: BodyKind,
        entry_block: &BlockRef<'context, 'block>,
        environment: &mut Environment<'context, 'block>,
    ) -> anyhow::Result<Vec<Option<Value<'context, 'block>>>> {
        // A modifier body seeds every return slot (named or not) from the values
        // threaded in as trailing block arguments at the `parameter_count`
        // offset, rather than zero-initialising only the named ones, so the
        // shared return state is observable and survives an empty body or a
        // partial `_` reach.
        if body_kind == BodyKind::ModifierBody {
            let mut return_slots: Vec<Option<Value<'context, 'block>>> = Vec::new();
            if let Some(returns) = function.returns() {
                for (index, parameter) in returns.iter().enumerate() {
                    let return_type = result_types[index];
                    let pointer = crate::ast::Pointer::stack_slot(
                        crate::ast::Type::new(return_type),
                        &self.state.builder,
                        entry_block,
                    )
                    .into_mlir();
                    let incoming: Value<'context, 'block> =
                        entry_block.argument(parameter_count + index)?.into();
                    sol_op_void!(
                        &self.state.builder,
                        entry_block,
                        StoreOperation.val(incoming).addr(pointer)
                    );
                    if parameter.name().is_some() {
                        environment.define_variable(parameter.node_id(), pointer, return_type);
                    }
                    return_slots.push(Some(pointer));
                }
            }
            return Ok(return_slots);
        }
        let mut return_slots: Vec<Option<Value<'context, 'block>>> = Vec::new();
        if let Some(returns) = function.returns() {
            for (index, parameter) in returns.iter().enumerate() {
                if parameter.name().is_none() {
                    return_slots.push(None);
                    continue;
                }
                let return_type = result_types[index];
                let pointer = TypeConversion::emit_default_initialized_slot(
                    parameter.get_type().as_ref(),
                    return_type,
                    &self.state.builder,
                    entry_block,
                );
                environment.define_variable(parameter.node_id(), pointer, return_type);
                return_slots.push(Some(pointer));
            }
        }
        Ok(return_slots)
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
        let contract = self
            .contract
            .expect("the constructor emitter requires a contract");

        // C3 linearisation, most-derived (self) first. Interfaces have no
        // constructor, so only contracts contribute to the construction chain.
        let mro: Vec<ContractDefinition> = contract
            .linearised_bases()
            .into_iter()
            .filter_map(|base| match base {
                ContractBase::Contract(base_contract) => Some(base_contract),
                ContractBase::Interface(_) => None,
            })
            .collect();

        // When no base contributes a constructor, the contract's own constructor
        // (or an empty one running just the state-variable initializers) is the
        // entire construction, emitted via `emit_sol` (which wraps its own
        // modifiers as separate `sol.func`s). A chain that DOES have a base
        // constructor — with or without modifiers — takes the chain path below,
        // where `emit_constructor_bodies` runs every base body in C3 order and
        // applies each constructor's modifiers as an inline chain.
        let has_base_constructor = mro.iter().skip(1).any(|base| base.constructor().is_some());
        if !has_base_constructor {
            if let Some(constructor) = contract.constructor() {
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
                None,
                contract_body,
            );
            let environment = Environment::new();
            let emitter = ExpressionContext::new(
                self.state,
                &environment,
                self.storage_layout,
                ArithmeticMode::Checked,
            );
            let block = emitter.emit_state_var_initializers(contract, entry)?;
            sol_op_void!(&self.state.builder, &block, ReturnOperation.operands(&[]));
            return Ok(());
        }

        // Inheritance chain: one `constructor()` runs every base constructor
        // (base-first), each in its own parameter scope, after the linearised
        // state-variable initializers. The deployed constructor takes the
        // most-derived contract's own constructor parameters.
        let derived_constructor = contract.constructor();
        let (parameter_types, mutability) = match &derived_constructor {
            Some(constructor) => {
                let (parameter_types, _) = constructor
                    .resolve_signature_types(LocationPolicy::Declared(None), &self.state.builder);
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
            None,
            contract_body,
        );

        // Per-contract constructor scopes, keyed by contract node id. Each holds
        // that contract's constructor parameters (and, while its body is emitted,
        // its local variables). Base constructors routinely reuse the derived
        // contract's parameter names, so a single flat scope would clobber them.
        let mut root_environment = Environment::new();
        if let Some(constructor) = &derived_constructor {
            for (index, parameter) in constructor.parameters().iter().enumerate() {
                let parameter_type = parameter_types[index];
                let parameter_value: Value<'context, '_> = entry.argument(index)?.into();
                let pointer = crate::ast::Pointer::stack_slot(
                    crate::ast::Type::new(parameter_type),
                    &self.state.builder,
                    &entry,
                )
                .into_mlir();
                sol_op_void!(
                    &self.state.builder,
                    &entry,
                    StoreOperation.val(parameter_value).addr(pointer)
                );
                root_environment.define_variable(parameter.node_id(), pointer, parameter_type);
            }
        }

        // State-variable initializers (whole C3 hierarchy) run first; they cannot
        // reference constructor parameters, so the scope only matters for the
        // shared storage layout.
        let mut current_block = {
            let emitter = ExpressionContext::new(
                self.state,
                &root_environment,
                self.storage_layout,
                ArithmeticMode::Checked,
            );
            emitter.emit_state_var_initializers(contract, entry)?
        };

        let mut scopes: HashMap<NodeId, Environment<'context, '_>> = HashMap::new();
        scopes.insert(contract.node_id(), root_environment);
        let mut bound_scopes: HashSet<NodeId> = HashSet::new();
        bound_scopes.insert(contract.node_id());

        let mro_node_ids: HashSet<NodeId> = mro.iter().map(|base| base.node_id()).collect();
        current_block = self.bind_base_constructor_scopes(
            &mro,
            &mro_node_ids,
            &mut scopes,
            &mut bound_scopes,
            current_block,
        )?;
        self.emit_constructor_bodies(&mro, &mut scopes, &bound_scopes, &entry, current_block)
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
    fn emit_default_return<'block>(
        &self,
        function: &FunctionDefinition,
        result_types: &[Type<'context>],
        return_slots: &[Option<Value<'context, 'block>>],
        block: &BlockRef<'context, 'block>,
    ) {
        if block.terminator().is_some() {
            return;
        }
        // Named returns load from their slot; an unnamed return (no slot) reached
        // on this fall-through path materialises its type's own default. The
        // default must be type-correct, not `emit_sol_constant(0, r#type)` — a string
        // / bytes / aggregate / address / fixed-bytes type is not an integer, so
        // an integer-attribute zero of that type is an ill-typed op.
        let returns: Vec<_> = function
            .returns()
            .map(|params| params.iter().collect::<Vec<_>>())
            .unwrap_or_default();
        let builder = &self.state.builder;
        let values: Vec<Value<'context, 'block>> = result_types
            .iter()
            .enumerate()
            .map(
                |(index, &return_type)| match return_slots.get(index).copied().flatten() {
                    Some(pointer) => crate::ast::Pointer::new(pointer)
                        .load(crate::ast::Type::new(return_type), builder, block)
                        .into_mlir(),
                    None => {
                        let slang_type = returns
                            .get(index)
                            .and_then(|parameter| parameter.get_type());
                        self.default_return_value(slang_type.as_ref(), return_type, block)
                    }
                },
            )
            .collect();
        sol_op_void!(builder, block, ReturnOperation.operands(&values));
    }

    /// The default value of a return position reached without an explicit
    /// `return <value>` (a fall-through epilogue past a body that does not end in
    /// a return — e.g. after a `try` whose branches all return). Mirrors solc's
    /// default-initialised return: a fresh zeroed buffer for a memory aggregate,
    /// an empty buffer for dynamic `string` / `bytes`, the representation's own
    /// zero for the other scalar value types, and an integer-width zero for an
    /// integer / bool (or a dead-path storage reference / mapping).
    fn default_return_value<'block>(
        &self,
        slang_type: Option<&SlangType>,
        return_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let builder = &self.state.builder;
        let is_memory = |location| matches!(location, slang_solidity_v2::ast::DataLocation::Memory);
        match slang_type {
            Some(SlangType::FixedSizeArray(array)) if is_memory(array.location()) => {
                sol_op!(
                    builder,
                    block,
                    MallocOperation
                        .addr(return_type)
                        .zero_init(Attribute::unit(builder.context))
                )
            }
            Some(SlangType::Struct(structure)) if is_memory(structure.location()) => {
                sol_op!(
                    builder,
                    block,
                    MallocOperation
                        .addr(return_type)
                        .zero_init(Attribute::unit(builder.context))
                )
            }
            Some(SlangType::Array(array)) if is_memory(array.location()) => {
                sol_op!(
                    builder,
                    block,
                    MallocOperation
                        .addr(return_type)
                        .zero_init(Attribute::unit(builder.context))
                )
            }
            Some(SlangType::String(_) | SlangType::Bytes(_)) => {
                // A fresh zero-length buffer (plain `sol.malloc`, matching solc),
                // not a sized `new bytes(0)`.
                sol_op!(builder, block, MallocOperation.addr(return_type))
            }
            Some(
                scalar @ (SlangType::Address(_)
                | SlangType::ByteArray(_)
                | SlangType::Enum(_)
                | SlangType::UserDefinedValue(_)
                | SlangType::Function(_)
                | SlangType::Contract(_)
                | SlangType::Interface(_)),
            ) => TypeConversion::emit_scalar_zero(scalar, return_type, builder, block),
            _ => crate::ast::Value::constant(0, crate::ast::Type::new(return_type), builder, block)
                .into_mlir(),
        }
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
