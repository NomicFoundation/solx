//!
//! Function definition emission to Sol dialect MLIR.
//!

use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use slang_solidity_v2::ast::DataLocation;
pub mod body_kind;
pub mod expression;
pub mod function_scope;
pub mod mlir_symbol_name;
pub mod modifier;
pub mod modifier_body_call;
pub mod modifier_parameter_binding;
pub mod signature;
pub mod statement;

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Environment;
use solx_mlir::Function;
use solx_mlir::StateMutability;
use solx_mlir::ods::sol::MallocOperation;
use solx_mlir::ods::sol::ReturnOperation;

use self::body_kind::BodyKind;
use self::mlir_symbol_name::MlirSymbolName;
use self::modifier::ModifiedBody;
use self::signature::Signature;
use self::statement::StatementContext;
use crate::ast::EmitFunction;
use crate::ast::EmitStatement;
use crate::ast::LocationPolicy;
use crate::ast::emit::EmitConstructor;
use crate::ast::emit::EmitModifierChain;

pub use self::function_scope::FunctionScope;

impl EmitFunction for FunctionDefinition {
    fn emit<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        contract_body: &BlockRef<'context, '_>,
    ) {
        self.emit_inner(scope, None, contract_body, BodyKind::Function);
    }

    fn emit_with_symbol<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        symbol: &str,
        contract_body: &BlockRef<'context, '_>,
    ) {
        self.emit_inner(scope, Some(symbol), contract_body, BodyKind::Function);
    }

    fn emit_inner<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        symbol_override: Option<&str>,
        contract_body: &BlockRef<'context, '_>,
        body_kind: BodyKind,
    ) {
        let Some(ref body) = self.body() else {
            // Abstract or interface function — no codegen needed.
            return;
        };

        let Signature {
            mlir_name,
            mlir_parameter_types,
            parameter_count,
            result_types,
            selector,
            state_mutability,
            mlir_kind,
        } = self.resolve_signature(scope, symbol_override, body_kind);

        // A regular function (real body, not a constructor/fallback/receive, not a
        // modifier-stage `$body`) can be the target of an internal function pointer,
        // so it carries a unique dispatch tag. Includes free/library functions
        // (`p(libFn)`); only modifier bodies and synthetic dispatchers are excluded.
        let function_id = (body_kind == BodyKind::Function && mlir_kind.is_none())
            .then(|| scope.state.next_function_id());

        let signature = Function::new(mlir_name, mlir_parameter_types, result_types);
        let function_entry_block = signature.define(
            selector,
            state_mutability,
            mlir_kind,
            function_id,
            &scope.state.builder,
            contract_body,
        );

        let mut environment = Environment::new();
        self.bind_parameters(
            scope,
            &signature.parameter_types,
            &function_entry_block,
            &mut environment,
        );

        let mut return_slots = self.initialize_return_slots(
            scope,
            &signature.return_types,
            parameter_count,
            body_kind,
            &function_entry_block,
            &mut environment,
        );

        let region = function_entry_block
            .parent_region()
            .expect("entry block belongs to a region");
        let mut current_block = function_entry_block;

        // State variable initializers run at the top of the constructor body. The
        // wrapping modified function already runs them, so a `$body` emission must
        // not run them again.
        if matches!(self.kind(), FunctionKind::Constructor) && body_kind == BodyKind::Function {
            current_block = scope
                .contract
                .expect("a constructor is emitted only within a contract")
                .emit_state_var_initializers(scope, current_block);
        }

        // Collect the modifier bodies that wrap this function
        // (`function f() onlyOwner {...}`). In modifier-body mode the stages are
        // emitted by the wrapping call, so a raw `$body` emission collects none.
        let (modifier_stages, modifier_stage_params) = if body_kind == BodyKind::ModifierBody {
            (Vec::new(), Vec::new())
        } else {
            let (stages, params, next_block) =
                self.build_modifier_stages(scope, &environment, current_block);
            current_block = next_block;
            (stages, params)
        };

        let mut terminated = false;
        if modifier_stages.is_empty() {
            for statement in body.statements().iter() {
                let mut emitter = StatementContext::new(
                    scope.state,
                    &mut environment,
                    &region,
                    scope.storage_layout,
                    &signature.return_types,
                    &return_slots,
                );
                match statement.emit(&mut emitter, current_block) {
                    Some(next) => current_block = next,
                    None => {
                        terminated = true;
                        break;
                    }
                }
            }
        } else {
            let frame = ModifiedBody::new(
                &signature.mlir_name,
                &signature.parameter_types,
                &signature.return_types,
                contract_body,
                &function_entry_block,
            );
            match self.emit_modified_body(
                scope,
                &frame,
                &mut environment,
                &mut return_slots,
                modifier_stages,
                modifier_stage_params,
                current_block,
            ) {
                Some(next) => current_block = next,
                None => terminated = true,
            }
        }

        if !terminated {
            self.emit_default_return(
                scope,
                &signature.return_types,
                &return_slots,
                &current_block,
            );
        }
    }

    fn resolve_signature<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        symbol_override: Option<&str>,
        body_kind: BodyKind,
    ) -> Signature<'context> {
        let mlir_name = symbol_override
            .map(str::to_owned)
            .unwrap_or_else(|| self.mlir_function_name());

        let (mut mlir_parameter_types, result_types) =
            AstType::resolve_signature(self, LocationPolicy::Declared(None), &scope.state.builder);

        // Recorded before the modifier-body extension below.
        let parameter_count = mlir_parameter_types.len();

        // A modifier body (`$body`) receives the wrapping function's return values
        // as trailing parameters, so its return slots can be seeded from the body
        // call and observed by the modifier tail and epilogue.
        if body_kind == BodyKind::ModifierBody {
            mlir_parameter_types.extend(result_types.iter().copied());
        }

        let state_mutability = StateMutability::from(self.mutability());

        let (selector, mlir_kind) = match (symbol_override, body_kind) {
            (None, BodyKind::Function) => {
                let mlir_kind = match self.kind() {
                    FunctionKind::Constructor => Some(solx_mlir::FunctionKind::Constructor),
                    FunctionKind::Fallback => Some(solx_mlir::FunctionKind::Fallback),
                    FunctionKind::Receive => Some(solx_mlir::FunctionKind::Receive),
                    FunctionKind::Regular => None,
                    FunctionKind::Modifier => {
                        unreachable!("modifiers are filtered before emission")
                    }
                };
                (self.compute_selector(), mlir_kind)
            }
            _ => (None, None),
        };

        Signature {
            mlir_name,
            mlir_parameter_types,
            parameter_count,
            result_types,
            selector,
            state_mutability,
            mlir_kind,
        }
    }

    fn bind_parameters<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        parameter_types: &[Type<'context>],
        entry_block: &BlockRef<'context, 'block>,
        environment: &mut Environment<'context, 'block>,
    ) {
        for (index, parameter) in self.parameters().iter().enumerate() {
            let parameter_type = parameter_types[index];
            let parameter_value = AstValue::new(
                entry_block
                    .argument(index)
                    .expect("argument index is within the block signature")
                    .into(),
            );
            let pointer = Pointer::stack_slot(
                AstType::new(parameter_type),
                &scope.state.builder,
                entry_block,
            );
            pointer.store(parameter_value, &scope.state.builder, entry_block);
            environment.define_variable(parameter.node_id(), pointer.into_mlir());
        }
    }

    fn initialize_return_slots<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        result_types: &[Type<'context>],
        parameter_count: usize,
        body_kind: BodyKind,
        entry_block: &BlockRef<'context, 'block>,
        environment: &mut Environment<'context, 'block>,
    ) -> Vec<Option<Value<'context, 'block>>> {
        // A modifier body seeds every return slot (named or not) from the values
        // threaded in as trailing block arguments at the `parameter_count` offset,
        // rather than zero-initialising only the named ones, so the shared return
        // state survives an empty body or a partial `_` reach.
        if body_kind == BodyKind::ModifierBody {
            let mut return_slots: Vec<Option<Value<'context, 'block>>> = Vec::new();
            if let Some(returns) = self.returns() {
                for (index, parameter) in returns.iter().enumerate() {
                    let return_type = result_types[index];
                    let pointer = Pointer::stack_slot(
                        AstType::new(return_type),
                        &scope.state.builder,
                        entry_block,
                    );
                    let incoming = AstValue::new(
                        entry_block
                            .argument(parameter_count + index)
                            .expect("argument index is within the block signature")
                            .into(),
                    );
                    pointer.store(incoming, &scope.state.builder, entry_block);
                    if parameter.name().is_some() {
                        environment.define_variable(parameter.node_id(), pointer.into_mlir());
                    }
                    return_slots.push(Some(pointer.into_mlir()));
                }
            }
            return return_slots;
        }
        let mut return_slots: Vec<Option<Value<'context, 'block>>> = Vec::new();
        if let Some(returns) = self.returns() {
            for (index, parameter) in returns.iter().enumerate() {
                if parameter.name().is_none() {
                    return_slots.push(None);
                    continue;
                }
                let return_type = result_types[index];
                let pointer = Pointer::default_initialized(
                    AstType::new(return_type),
                    &scope.state.builder,
                    entry_block,
                )
                .into_mlir();
                environment.define_variable(parameter.node_id(), pointer);
                return_slots.push(Some(pointer));
            }
        }
        return_slots
    }

    fn emit_default_return<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        result_types: &[Type<'context>],
        return_slots: &[Option<Value<'context, 'block>>],
        block: &BlockRef<'context, 'block>,
    ) {
        if block.terminator().is_some() {
            return;
        }
        // Named returns load from their slot; an unnamed return (no slot) reached on
        // this fall-through path materialises its type's own default. The default
        // must be type-correct: a string/bytes/aggregate/address/fixed-bytes type is
        // not an integer, so an integer-attribute zero of that type is ill-typed.
        let returns: Vec<_> = self
            .returns()
            .map(|params| params.iter().collect::<Vec<_>>())
            .unwrap_or_default(); // recut-lint-allow: fail01 — a function may declare no returns
        let builder = &scope.state.builder;
        let values: Vec<Value<'context, 'block>> = result_types
            .iter()
            .enumerate()
            .map(
                |(index, &return_type)| match return_slots.get(index).copied().flatten() {
                    Some(pointer) => Pointer::new(pointer)
                        .load(AstType::new(return_type), builder, block)
                        .into_mlir(),
                    None => {
                        let slang_type = returns
                            .get(index)
                            .and_then(|parameter| parameter.get_type());
                        self.default_return_value(scope, slang_type.as_ref(), return_type, block)
                    }
                },
            )
            .collect();
        mlir_op_void!(builder, block, ReturnOperation.operands(&values));
    }

    fn default_return_value<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        slang_type: Option<&SlangType>,
        return_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let builder = &scope.state.builder;
        let is_memory = |location| matches!(location, DataLocation::Memory);
        match slang_type {
            Some(SlangType::FixedSizeArray(array)) if is_memory(array.location()) => {
                mlir_op!(
                    builder,
                    block,
                    MallocOperation
                        .addr(return_type)
                        .zero_init(Attribute::unit(builder.context))
                )
            }
            Some(SlangType::Struct(structure)) if is_memory(structure.location()) => {
                mlir_op!(
                    builder,
                    block,
                    MallocOperation
                        .addr(return_type)
                        .zero_init(Attribute::unit(builder.context))
                )
            }
            Some(SlangType::Array(array)) if is_memory(array.location()) => {
                mlir_op!(
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
                mlir_op!(builder, block, MallocOperation.addr(return_type))
            }
            Some(
                SlangType::Address(_)
                | SlangType::ByteArray(_)
                | SlangType::Enum(_)
                | SlangType::UserDefinedValue(_)
                | SlangType::Function(_)
                | SlangType::Contract(_)
                | SlangType::Interface(_),
            ) => AstValue::zero(AstType::new(return_type), builder, block).into_mlir(),
            _ => AstValue::constant(0, AstType::new(return_type), builder, block).into_mlir(),
        }
    }
}
