//!
//! Function definition emission to Sol dialect MLIR.
//!

pub mod expression;
pub mod function_scope;
pub mod mlir_symbol_name;
pub mod modifier;
pub mod signature;
pub mod statement;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::FunctionDefinition;

use solx_mlir::Environment;
use solx_mlir::Function;
use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::ReturnOperation;

use self::function_scope::FunctionScope;
use self::signature::Signature;
use self::statement::StatementContext;
use crate::ast::emit::emit_function::EmitFunction;
use crate::ast::emit::emit_modifier_calls::EmitModifierCalls;
use crate::ast::emit::emit_statement::EmitStatement;

impl EmitFunction for FunctionDefinition {
    fn emit<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        contract_body: &BlockRef<'context, '_>,
    ) {
        self.emit_inner(scope, None, contract_body);
    }

    fn emit_with_symbol<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        symbol: &str,
        contract_body: &BlockRef<'context, '_>,
    ) {
        self.emit_inner(scope, Some(symbol), contract_body);
    }

    fn emit_inner<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        symbol_override: Option<&str>,
        contract_body: &BlockRef<'context, '_>,
    ) {
        let Some(ref body) = self.body() else {
            return;
        };

        let Signature {
            mlir_name,
            mlir_parameter_types,
            mlir_result_types,
            selector,
            state_mutability,
            mlir_kind,
        } = Signature::resolve(self, symbol_override, scope.state);

        let function_id = mlir_kind
            .is_none()
            .then(|| scope.state.next_function_identifier());

        let signature = Function::new(mlir_name, mlir_parameter_types, mlir_result_types);
        let function_entry_block = signature.define(
            selector,
            state_mutability,
            mlir_kind,
            function_id,
            scope.state,
            contract_body,
        );

        let mut current_block = function_entry_block;

        self.emit_modifier_call_blocks(
            scope,
            &self.parameters().iter().collect::<Vec<_>>(),
            &signature.parameter_types,
            &current_block,
        );

        let mut environment = Environment::new();
        for (index, parameter) in self.parameters().iter().enumerate() {
            environment.bind_block_argument(
                parameter.node_id(),
                signature.parameter_types[index],
                index,
                &function_entry_block,
                scope.state,
            );
        }

        let mut return_slots: Vec<Option<Value<'context, '_>>> = Vec::new();
        if let Some(returns) = self.returns() {
            for (index, parameter) in returns.iter().enumerate() {
                let return_type = AstType::new(signature.return_types[index]);
                if parameter.name().is_none() {
                    return_slots.push(None);
                } else {
                    let pointer = Pointer::default_initialized(
                        return_type,
                        scope.state,
                        &function_entry_block,
                    )
                    .into_mlir();
                    environment.define_variable(parameter.node_id(), pointer);
                    return_slots.push(Some(pointer));
                }
            }
        }

        let mut terminated = false;
        for statement in body.statements().iter() {
            let mut emitter = StatementContext::new(
                scope.state,
                &mut environment,
                scope.dispatch,
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

        if !terminated {
            let returns: Vec<_> = self
                .returns()
                .map(|parameters| parameters.iter().collect::<Vec<_>>())
                .unwrap_or_default();
            let state = scope.state;
            let values: Vec<_> = signature
                .return_types
                .iter()
                .enumerate()
                .map(|(index, &return_type)| match return_slots[index] {
                    Some(pointer) => Pointer::new(pointer)
                        .load(AstType::new(return_type), state, &current_block)
                        .into_mlir(),
                    None => {
                        let slang_type = returns
                            .get(index)
                            .and_then(|parameter| parameter.get_type());
                        AstValue::type_default(slang_type.as_ref(), return_type, state, &current_block)
                            .into_mlir()
                    }
                })
                .collect();
            mlir_op_void!(state, &current_block, ReturnOperation.operands(&values));
        }
    }
}
