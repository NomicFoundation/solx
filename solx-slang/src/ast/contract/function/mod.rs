//!
//! Function definition emission to Sol dialect MLIR.
//!

use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
pub mod body_kind;
pub mod expression;
pub mod function_scope;
pub mod mlir_symbol_name;
pub mod modifier;
pub mod modifier_body_call;
pub mod modifier_parameter_binding;
pub mod signature;
pub mod statement;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;

use solx_mlir::Environment;
use solx_mlir::Function;
use solx_mlir::ods::sol::ReturnOperation;

use self::body_kind::BodyKind;
use self::modifier::ModifiedBody;
use self::signature::Signature;
use self::statement::StatementContext;
use crate::ast::EmitFunction;
use crate::ast::EmitStatement;
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
        } = Signature::resolve(self, symbol_override, body_kind, &scope.state.builder);

        // A regular function can be the target of an internal function pointer, so it carries a
        // unique dispatch tag (modifier bodies and synthetic dispatchers are excluded).
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
        for (index, parameter) in self.parameters().iter().enumerate() {
            environment.bind_block_argument(
                parameter.node_id(),
                signature.parameter_types[index],
                index,
                &function_entry_block,
                &scope.state.builder,
            );
        }

        // A stack slot per named return (`None` for unnamed). A modifier body instead seeds every slot
        // from the trailing block arguments at the `parameter_count` offset.
        let mut return_slots: Vec<Option<Value<'context, '_>>> = Vec::new();
        if let Some(returns) = self.returns() {
            for (index, parameter) in returns.iter().enumerate() {
                let return_type = AstType::new(signature.return_types[index]);
                if body_kind == BodyKind::ModifierBody {
                    let pointer = Pointer::from_argument(
                        return_type,
                        parameter_count + index,
                        &function_entry_block,
                        &scope.state.builder,
                    );
                    if parameter.name().is_some() {
                        environment.define_variable(parameter.node_id(), pointer.into_mlir());
                    }
                    return_slots.push(Some(pointer.into_mlir()));
                } else if parameter.name().is_none() {
                    return_slots.push(None);
                } else {
                    let pointer = Pointer::default_initialized(
                        return_type,
                        &scope.state.builder,
                        &function_entry_block,
                    )
                    .into_mlir();
                    environment.define_variable(parameter.node_id(), pointer);
                    return_slots.push(Some(pointer));
                }
            }
        }

        let region = function_entry_block
            .parent_region()
            .expect("entry block belongs to a region");
        let mut current_block = function_entry_block;

        // State-variable initializers run at the top of the constructor body, but not in a `$body`
        // emission (the wrapping modified function already ran them).
        if matches!(self.kind(), FunctionKind::Constructor) && body_kind == BodyKind::Function {
            current_block = scope
                .contract
                .expect("a constructor is emitted only within a contract")
                .emit_state_var_initializers(scope, current_block);
        }

        // Collect the modifier bodies wrapping this function (none in `$body` mode — the wrapping
        // call emits the stages).
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
        // Named returns load from their slot; an unnamed return materialises its type's own default
        // (a type-correct one, not an ill-typed integer zero).
        let returns: Vec<_> = self
            .returns()
            .map(|params| params.iter().collect::<Vec<_>>())
            .unwrap_or_default();
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
                        AstValue::type_default(slang_type.as_ref(), return_type, builder, block)
                            .into_mlir()
                    }
                },
            )
            .collect();
        mlir_op_void!(builder, block, ReturnOperation.operands(&values));
    }
}
