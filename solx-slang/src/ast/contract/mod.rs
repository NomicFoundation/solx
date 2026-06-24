//!
//! Contract definition emission to Sol dialect MLIR.
//!

pub mod constructor;
pub mod function;
pub mod getter;
pub mod object_scope;
pub mod storage_layout;

pub use self::object_scope::ObjectScope;

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::FunctionMutability;

use solx_mlir::Context;
use solx_mlir::ods::sol::StateVarOperation;

use self::function::FunctionScope;
use self::function::mlir_symbol_name::MlirSymbolName;
use crate::ast::EmitFunction;
use crate::ast::EmitObject;
use crate::ast::Type as AstType;
use crate::ast::emit::EmitConstructor;
use crate::ast::analysis::query::StorageLayout;

impl EmitObject for ContractDefinition {
    fn emit(&self, context: &mut Context, _scope: &ObjectScope) {
        let contract_name = self.name().name();

        // Pre-register the contract's own functions so a call by bare name resolves to its
        // registered symbol before any body is emitted.
        self.register_signatures(
            context,
            self.functions()
                .into_iter()
                .filter(|function| !matches!(function.kind(), FunctionKind::Modifier))
                .map(|function| {
                    let symbol = function.mlir_function_name();
                    (function, symbol)
                }),
        );

        let storage_layout = self.storage_layout();
        let is_payable = self.functions().iter().any(|function| {
            matches!(function.kind(), FunctionKind::Receive)
                || (matches!(function.kind(), FunctionKind::Fallback)
                    && matches!(function.mutability(), FunctionMutability::Payable))
        });
        let contract_type =
            AstType::contract(context.builder.context, &contract_name, is_payable).into_mlir();

        let module_body = context.module.body();
        let contract_body = self.emit_contract_shell(
            context,
            &contract_name,
            solx_mlir::ContractKind::Contract,
            &module_body,
        );

        // Declare the contract's own state variables; each body / getter addresses its slot by symbol.
        for state_variable in self.state_variables() {
            let Some(slot) = storage_layout.get(&state_variable.node_id()) else {
                continue;
            };
            let element_type = AstType::resolve_state_variable(
                &state_variable.get_type().expect("slang validated"),
                &context.builder,
            );
            let builder = &context.builder;
            let slot_attribute: IntegerAttribute =
                Attribute::parse(builder.context, &format!("{} : i256", slot.slot))
                    .expect("valid slot literal")
                    .try_into()
                    .expect("slot literal is an integer attribute");
            let byte_offset_attribute = IntegerAttribute::new(
                IntegerType::new(builder.context, solx_utils::BIT_LENGTH_X32 as u32).into(),
                slot.byte_offset.into(),
            );
            let operation =
                StateVarOperation::builder(builder.context, builder.unknown_location)
                    .sym_name(StringAttribute::new(builder.context, &slot.name))
                    .r#type(TypeAttribute::new(element_type))
                    .slot(slot_attribute)
                    .byte_offset(byte_offset_attribute);
            contract_body.append_operation(operation.build().into());
        }

        context.current_contract_type = Some(contract_type);
        self.emit_constructor(
            &FunctionScope::new(context, Some(self), &storage_layout),
            &contract_body,
        );
        context.current_contract_type = None;

        // Slang's `functions()` filters out Constructor and Modifier kinds.
        for function in self.functions() {
            context.current_contract_type = Some(contract_type);
            function.emit(
                &FunctionScope::new(context, Some(self), &storage_layout),
                &contract_body,
            );
            context.current_contract_type = None;
        }
    }
}
