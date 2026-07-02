//!
//! Function definition lowering to Sol dialect MLIR.
//!

pub mod expression;
pub mod modifier;
pub mod statement;

use std::collections::HashMap;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::abi::AbiEntry;
use slang_solidity_v2::ast::ContractBase;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::ElementaryType;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::ModifierInvocation;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Type as SlangType;
use slang_solidity_v2::ast::TypeName;

use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::Function;
use solx_mlir::Pointer;
use solx_mlir::StateMutability;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::ReturnOperation;

use crate::ast::analysis::query::base_constructor_arguments::BaseConstructorArguments;
use crate::ast::analysis::query::base_constructor_chain::BaseConstructorChain;
use crate::ast::analysis::query::modifier_resolution::ModifierResolution;
use crate::ast::analysis::query::storage_layout::StorageSlot;
use crate::ast::block_and::BlockAnd;
use crate::ast::contract::contract_dispatch::ContractDispatch;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::emit::emit_constructor::EmitConstructor;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_function::EmitFunction;
use crate::ast::emit::emit_modifier_calls::EmitModifierCalls;
use crate::ast::emit::emit_statement::EmitStatement;

/// Lowers a Solidity function definition to a `sol.func` operation.
pub struct FunctionEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state Context<'context>,
    /// Containing contract, absent for a deployable library's own functions.
    contract: Option<&'state ContractDefinition>,
    /// State variable node ID to `(slot, byte_offset)` mapping. The byte
    /// offset is zero for unpacked variables and non-zero for variables
    /// packed into a shared slot.
    storage_layout: &'state HashMap<NodeId, StorageSlot>,
    /// Contract-local super/base and virtual dispatch maps, threaded into every emitted expression
    /// so a `super.f()` / `Base.f()` call and a virtual internal call resolve their C3-linearised
    /// target.
    dispatch: &'state ContractDispatch,
}

impl<'state, 'context> FunctionEmitter<'state, 'context> {
    /// Creates a new function emitter.
    pub fn new(
        state: &'state Context<'context>,
        contract: Option<&'state ContractDefinition>,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
        dispatch: &'state ContractDispatch,
    ) -> Self {
        Self {
            state,
            contract,
            storage_layout,
            dispatch,
        }
    }

    /// The shared MLIR context threaded through emission.
    pub fn state(&self) -> &'state Context<'context> {
        self.state
    }

    /// The state-variable storage layout of the contract being emitted.
    pub fn storage_layout(&self) -> &'state HashMap<NodeId, StorageSlot> {
        self.storage_layout
    }

    /// The contract-local super/base and virtual dispatch maps of the contract being emitted.
    pub fn dispatch(&self) -> &'state ContractDispatch {
        self.dispatch
    }

    /// Builds the MLIR function name as `{name}({types})`.
    ///
    /// Uses slang's ABI canonical types when available (external functions),
    /// falls back to AST-based type names for internal/private functions.
    pub fn mlir_function_name(function: &FunctionDefinition) -> String {
        let name = Self::mlir_base_name(function);

        if matches!(function.enclosing_definition(), Some(Definition::Library(_)))
            && let Some(signature) = function.compute_library_signature()
        {
            return signature;
        }

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

    /// The MLIR symbol of a reached free function: its signature suffixed with the bare AST node id,
    /// disambiguating two free functions of the same signature and never colliding with a contract
    /// method's ABI-selector symbol.
    pub fn free_function_symbol(function: &FunctionDefinition) -> String {
        format!("{}_{}", Self::mlir_function_name(function), function.node_id())
    }

    /// The MLIR symbol of a base constructor emitted as a plain internal `sol.func` the construction
    /// chain `sol.call`s into, distinct from the most-derived `constructor()` def. Suffixed with the
    /// constructor's node id so each base contract's constructor resolves to its own symbol.
    pub fn base_constructor_symbol(constructor: &FunctionDefinition) -> String {
        format!("constructor_{}", constructor.node_id())
    }

    /// The MLIR symbol of a modifier definition: its name suffixed with its node id, so two like-named
    /// modifiers in an inherited override chain resolve to distinct `sol.modifier` defs. The same
    /// authority names both the `sol.modifier` def and the invoking `sol.call`.
    pub fn modifier_symbol(modifier: &FunctionDefinition) -> String {
        let name = modifier
            .name()
            .map(|identifier| identifier.name())
            .expect("a modifier definition has a name");
        format!("{name}_{}", modifier.node_id())
    }

    /// Resolves a modifier invocation to the body-bearing definition to emit, or `None` to skip a
    /// non-modifier invocation or a bodyless modifier. Applies lexical resolution, then virtual
    /// override re-dispatch against the emitting contract's C3-linearised modifier set.
    pub fn resolve_modifier_invocation(
        &self,
        invocation: &ModifierInvocation,
    ) -> Option<FunctionDefinition> {
        let Some(Definition::Modifier(lexical)) = invocation.name().resolve_to_definition() else {
            return None;
        };
        let definition = self
            .contract
            .and_then(|contract| contract.resolve_modifier_override(invocation, &lexical))
            .unwrap_or(lexical);
        definition.body().is_some().then_some(definition)
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

    /// Emits a default `sol.return` if the block lacks a terminator. A named return loads its
    /// default-initialised slot; an unnamed return materialises the typed zero of its Solidity return
    /// type via [`AstValue::type_default`].
    fn emit_default_return(
        &self,
        result_types: &[Type<'context>],
        return_slang_types: &[SlangType],
        return_slots: &[Option<Value<'context, '_>>],
        block: &BlockRef<'context, '_>,
    ) {
        if block.terminator().is_some() {
            return;
        }
        let values: Vec<Value<'context, '_>> = result_types
            .iter()
            .zip(return_slang_types)
            .zip(return_slots)
            .map(|((result_type, slang_type), slot)| match slot {
                Some(pointer) => Pointer::new(*pointer)
                    .load(AstType::new(*result_type), self.state, block)
                    .into_mlir(),
                None => AstValue::type_default(
                    slang_type,
                    AstType::new(*result_type),
                    self.state,
                    block,
                )
                .into_mlir(),
            })
            .collect();
        mlir_op_void!(self.state, block, ReturnOperation.operands(&values));
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

impl EmitFunction for FunctionDefinition {
    /// Emits a `sol.func` for this function definition into the given contract body block.
    fn emit<'context>(
        &self,
        emitter: &FunctionEmitter<'_, 'context>,
        symbol_override: Option<&str>,
        contract_body: &BlockRef<'context, '_>,
    ) -> String {
        let mlir_name = symbol_override
            .map(str::to_owned)
            .unwrap_or_else(|| FunctionEmitter::mlir_function_name(self));
        let Some(ref body) = self.body() else {
            return mlir_name;
        };

        let parameters = self.parameters();

        let (mlir_parameter_types, result_types) =
            TypeConversion::resolve_function_types(self, emitter.state);

        let state_mutability = FunctionEmitter::map_state_mutability(self);

        let (selector, mlir_kind) = match symbol_override {
            Some(_) => (None, None),
            None => {
                let mlir_kind = match self.kind() {
                    FunctionKind::Fallback => Some(solx_mlir::FunctionKind::Fallback),
                    FunctionKind::Receive => Some(solx_mlir::FunctionKind::Receive),
                    FunctionKind::Regular => None,
                    FunctionKind::Constructor => {
                        unreachable!("constructors are emitted through emit_constructor")
                    }
                    FunctionKind::Modifier => {
                        unreachable!("modifiers are filtered before emission")
                    }
                };
                (self.compute_selector(), mlir_kind)
            }
        };

        let dispatch_identifier = mlir_kind.is_none().then(|| emitter.state.next_function_id());

        let function_entry_block = Function::new(
            mlir_name.clone(),
            mlir_parameter_types.clone(),
            result_types.clone(),
        )
        .define(
            selector,
            state_mutability,
            mlir_kind,
            dispatch_identifier,
            emitter.state,
            contract_body,
        );

        self.emit_modifier_call_blocks(
            emitter,
            &parameters.iter().collect::<Vec<_>>(),
            &mlir_parameter_types,
            &function_entry_block,
        );

        let mut environment = Environment::new();

        for (index, parameter) in parameters.iter().enumerate() {
            let parameter_name = parameter
                .name()
                .map(|id| id.name())
                .unwrap_or_else(|| "_".to_owned());
            let parameter_type = mlir_parameter_types[index];
            let parameter_value: Value<'context, '_> = function_entry_block
                .argument(index)
                .expect("function entry block has one argument per parameter")
                .into();
            let pointer =
                Pointer::stack(AstType::new(parameter_type), emitter.state, &function_entry_block);
            pointer.store(
                AstValue::new(parameter_value),
                emitter.state,
                &function_entry_block,
            );

            environment.define_variable(parameter_name, pointer.into_mlir(), parameter_type);
        }

        let mut return_slang_types: Vec<SlangType> = Vec::new();
        let mut return_slots: Vec<Option<Value<'context, '_>>> = Vec::new();
        if let Some(returns) = self.returns() {
            for (index, parameter) in returns.iter().enumerate() {
                let return_type = result_types[index];
                let slang_type = parameter.get_type().expect("slang types every return");
                let slot = parameter.name().map(|identifier| {
                    let pointer = Pointer::default_initialized(
                        &slang_type,
                        AstType::new(return_type),
                        emitter.state,
                        &function_entry_block,
                    )
                    .into_mlir();
                    environment.define_variable(identifier.name(), pointer, return_type);
                    pointer
                });
                return_slang_types.push(slang_type);
                return_slots.push(slot);
            }
        }

        let region = function_entry_block
            .parent_region()
            .expect("entry block belongs to a region");
        let mut current_block = function_entry_block;

        let mut terminated = false;
        for statement in body.statements().iter() {
            let mut statement_context = StatementContext::new(
                emitter.state,
                &mut environment,
                &region,
                emitter.storage_layout,
                emitter.dispatch,
                &result_types,
            );
            match statement.emit(&mut statement_context, current_block) {
                Some(next) => current_block = next,
                None => {
                    terminated = true;
                    break;
                }
            }
        }

        if !terminated {
            emitter.emit_default_return(
                &result_types,
                &return_slang_types,
                &return_slots,
                &current_block,
            );
        }

        mlir_name
    }
}

impl<'state, 'context> FunctionEmitter<'state, 'context> {
    /// Emits one constructor `sol.func` of the C3 chain: the most-derived `constructor()` when
    /// `is_most_derived`, else a plain internal `sol.func` the chain `sol.call`s into.
    ///
    /// The most-derived function runs the linearised state-variable initializers, then binds `owner`'s
    /// constructor parameters, calls the next base constructor in the chain, and runs `owner`'s
    /// constructor body.
    fn emit_constructor_func(
        &self,
        owner: &ContractDefinition,
        mro: &[ContractDefinition],
        base_arguments: &HashMap<NodeId, BaseConstructorArguments>,
        is_most_derived: bool,
        contract_body: &BlockRef<'context, '_>,
    ) {
        let derived = self
            .contract
            .expect("a constructor is emitted only for a contract");
        let constructor = owner.constructor();

        let (symbol, kind, dispatch_identifier) = if is_most_derived {
            (
                "constructor()".to_owned(),
                Some(solx_mlir::FunctionKind::Constructor),
                None,
            )
        } else {
            let constructor = constructor
                .as_ref()
                .expect("a base constructor func is emitted only for a contract with a constructor");
            (
                FunctionEmitter::base_constructor_symbol(constructor),
                None,
                Some(self.state.next_function_id()),
            )
        };

        let (parameter_types, state_mutability) = match &constructor {
            Some(constructor) => {
                let (parameter_types, _) =
                    TypeConversion::resolve_function_types(constructor, self.state);
                (parameter_types, FunctionEmitter::map_state_mutability(constructor))
            }
            None => (Vec::new(), StateMutability::NonPayable),
        };

        let entry = Function::new(symbol, parameter_types.clone(), Vec::new()).define(
            None,
            state_mutability,
            kind,
            dispatch_identifier,
            self.state,
            contract_body,
        );

        let mut environment = Environment::new();
        let mut current_block = if is_most_derived {
            let expression_context = ExpressionContext::new(
                self.state,
                &environment,
                self.storage_layout,
                self.dispatch,
                true,
            );
            expression_context.emit_state_var_initializers(derived, entry)
        } else {
            entry
        };

        if let Some(constructor) = &constructor {
            constructor.emit_modifier_call_blocks(
                self,
                &constructor.parameters().iter().collect::<Vec<_>>(),
                &parameter_types,
                &current_block,
            );
        }

        if let Some(constructor) = &constructor {
            for (index, parameter) in constructor.parameters().iter().enumerate() {
                let parameter_name = parameter
                    .name()
                    .map(|identifier| identifier.name())
                    .unwrap_or_else(|| "_".to_owned());
                let parameter_type = parameter_types[index];
                let parameter_value: Value<'context, '_> = entry
                    .argument(index)
                    .expect("constructor entry block has one argument per parameter")
                    .into();
                let pointer =
                    Pointer::stack(AstType::new(parameter_type), self.state, &entry);
                pointer.store(AstValue::new(parameter_value), self.state, &entry);
                environment.define_variable(parameter_name, pointer.into_mlir(), parameter_type);
            }
        }

        current_block =
            self.emit_next_constructor_call(owner, mro, base_arguments, &environment, current_block);

        let mut terminated = false;
        if let Some(body) = constructor
            .as_ref()
            .and_then(|constructor| constructor.body())
        {
            let region = current_block
                .parent_region()
                .expect("entry block belongs to a region");
            let return_types: [Type<'context>; 0] = [];
            for statement in body.statements().iter() {
                let mut statement_context = StatementContext::new(
                    self.state,
                    &mut environment,
                    &region,
                    self.storage_layout,
                    self.dispatch,
                    &return_types,
                );
                match statement.emit(&mut statement_context, current_block) {
                    Some(next) => current_block = next,
                    None => {
                        terminated = true;
                        break;
                    }
                }
            }
        }

        if !terminated {
            mlir_op_void!(self.state, &current_block, ReturnOperation.operands(&[]));
        }
    }

    /// Emits the `sol.call` to the next base constructor in the MRO chain. Each argument is passed at
    /// its own type; the emitted `sol.call` carries the implicit-castable operand/parameter mismatch,
    /// so no operand cast is emitted here.
    fn emit_next_constructor_call<'block>(
        &self,
        owner: &ContractDefinition,
        mro: &[ContractDefinition],
        base_arguments: &HashMap<NodeId, BaseConstructorArguments>,
        environment: &Environment<'context, 'block>,
        mut current_block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let derived = self
            .contract
            .expect("a constructor is emitted only for a contract");
        let Some(next_contract) = derived.next_constructor_contract(owner, mro) else {
            return current_block;
        };
        let next_constructor = next_contract
            .constructor()
            .expect("next_constructor_contract returns a contract with a constructor");

        let parameter_ids: Vec<NodeId> = next_constructor
            .parameters()
            .iter()
            .map(|parameter| parameter.node_id())
            .collect();
        let ordered_arguments = base_arguments
            .get(&next_contract.node_id())
            .map(|specification| {
                specification
                    .arguments
                    .ordered_by(&parameter_ids)
                    .expect("slang matches every base-constructor argument to a parameter")
            })
            .unwrap_or_default();

        let expression_context = ExpressionContext::new(
            self.state,
            environment,
            self.storage_layout,
            self.dispatch,
            true,
        );
        let mut operands: Vec<Value<'context, 'block>> = Vec::with_capacity(ordered_arguments.len());
        for argument in ordered_arguments.iter() {
            let BlockAnd { value, block } = argument.emit(&expression_context, current_block);
            current_block = block;
            operands.push(value);
        }

        Function::call(
            &FunctionEmitter::base_constructor_symbol(&next_constructor),
            &operands,
            &[],
            self.state,
            &current_block,
        )
        .expect("a base constructor resolves to its emitted symbol");
        current_block
    }
}

impl EmitConstructor for ContractDefinition {
    /// Emits the contract's construction as the C3-linearised base-constructor chain: the most-derived
    /// `constructor()` `sol.func` followed by one plain internal `sol.func` per base constructor.
    fn emit_constructor<'context>(
        &self,
        emitter: &FunctionEmitter<'_, 'context>,
        contract_body: &BlockRef<'context, '_>,
    ) {
        let mro: Vec<ContractDefinition> = self
            .linearised_bases()
            .into_iter()
            .filter_map(|base| match base {
                ContractBase::Contract(base_contract) => Some(base_contract),
                ContractBase::Interface(_) => None,
            })
            .collect();

        let base_arguments = self.base_constructor_arguments(&mro);

        emitter.emit_constructor_func(self, &mro, &base_arguments, true, contract_body);

        for base_contract in mro.iter().skip(1) {
            if base_contract.constructor().is_none() {
                continue;
            }
            emitter.emit_constructor_func(
                base_contract,
                &mro,
                &base_arguments,
                false,
                contract_body,
            );
        }
    }
}
