//!
//! Contract constructor synthesis: the deploy-time `constructor()` `sol.func`.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;

use solx_mlir::Environment;
use solx_mlir::Function;
use solx_mlir::StateMutability;
use solx_mlir::ods::sol::ReturnOperation;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::EmitFunction;
use crate::ast::LocationPolicy;
use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::contract::function::FunctionScope;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::emit::EmitConstructor;

impl EmitConstructor for ContractDefinition {
    fn emit_constructor<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        contract_body: &BlockRef<'context, '_>,
    ) {
        if let Some(constructor) = self.constructor() {
            constructor.emit(scope, contract_body);
            return;
        }
        let entry = Function::new("constructor()".to_owned(), Vec::new(), Vec::new()).define(
            None,
            StateMutability::NonPayable,
            Some(solx_mlir::FunctionKind::Constructor),
            None,
            &scope.state.builder,
            contract_body,
        );
        let block = self.emit_state_var_initializers(scope, entry);
        mlir_op_void!(&scope.state.builder, &block, ReturnOperation.operands(&[]));
    }

    fn emit_state_var_initializers<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        mut block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        // Initializers cannot reference constructor parameters or locals, so they
        // run over an empty variable environment.
        let environment = Environment::new();
        let emitter = ExpressionContext::new(
            scope.state,
            &environment,
            scope.storage_layout,
            ArithmeticMode::Checked,
        );
        for member in self.members().iter() {
            let ContractMember::StateVariableDefinition(state_variable) = member else {
                continue;
            };
            let Some(slot) = scope.storage_layout.get(&state_variable.node_id()) else {
                continue;
            };
            let Some(initializer) = state_variable.value() else {
                continue;
            };
            let declared_type = state_variable.get_type().expect("slang validated");
            let builder = &scope.state.builder;
            let element_type =
                AstType::resolve(&declared_type, LocationPolicy::Declared(None), builder);
            let address_type =
                AstType::new(element_type).address_type(slot.location, builder.context);
            let storage_ref =
                Pointer::addr_of(&slot.name, address_type, builder, &block).into_mlir();
            let BlockAnd {
                value,
                block: next_block,
            } = initializer.emit(&emitter, block);
            block = next_block;
            if declared_type.is_reference_type() {
                Pointer::new(storage_ref).copy_from(value, builder, &block);
            } else {
                let stored_value = value.cast(AstType::new(element_type), builder, &block);
                Pointer::new(storage_ref).store(stored_value, builder, &block);
            }
        }
        block
    }
}
