//!
//! Contract definition lowering to Sol dialect MLIR.
//!

/// Function definition lowering to Sol dialect MLIR.
pub mod function;

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::FunctionMutability;

use solx_mlir::Context;
use solx_mlir::Type as AstType;
use solx_mlir::ods::sol::ContractOperation;
use solx_mlir::ods::sol::StateVarOperation;

use crate::ast::analysis::query::storage_layout::StorageLayout;
use crate::ast::emit::emit_constructor::EmitConstructor;
use crate::ast::emit::emit_function::EmitFunction;
use crate::ast::emit::emit_object::EmitObject;

use self::function::FunctionEmitter;
use self::function::expression::call::type_conversion::TypeConversion;

/// Lowers a Solidity contract to Sol dialect MLIR.
///
/// Emits `sol.contract` wrapping `sol.func` definitions. The
/// `convert-sol-to-yul` pass generates the entry-point dispatcher
/// from the function selectors.
pub struct ContractEmitter;

impl ContractEmitter {
    /// Returns whether `contract` is payable (declares a `receive()` function or
    /// a `payable` `fallback()` function). Single source of truth for payability
    /// derivation — used both when emitting the `sol.contract` op and when
    /// resolving `SlangType::Contract` to a `Sol_ContractType`.
    pub fn is_contract_payable(contract: &ContractDefinition) -> bool {
        contract.functions().iter().any(|function| {
            matches!(function.kind(), FunctionKind::Receive)
                || (matches!(function.kind(), FunctionKind::Fallback)
                    && matches!(function.mutability(), FunctionMutability::Payable))
        })
    }

    /// Pre-registers all function signatures for call resolution before bodies
    /// are emitted.
    fn pre_register_functions(context: &mut Context, contract: &ContractDefinition) {
        for function in contract.functions() {
            if matches!(function.kind(), FunctionKind::Modifier) {
                continue;
            }
            let mlir_name = FunctionEmitter::mlir_function_name(&function);
            let (parameter_types, return_types) =
                TypeConversion::resolve_function_types(&function, context);

            context.register_function_signature(
                function.node_id(),
                mlir_name,
                parameter_types,
                return_types,
            );
        }
    }
}

impl EmitObject for ContractDefinition {
    /// Emits a `sol.contract` containing all function definitions.
    fn emit(&self, context: &mut Context) {
        let contract_name = self.name().name();

        ContractEmitter::pre_register_functions(context, self);
        let storage_layout = self.storage_layout();

        let contract_type = AstType::contract(
            context.mlir_context,
            &contract_name,
            ContractEmitter::is_contract_payable(self),
        )
        .into_mlir();

        let module_body = context.module.body();
        let contract_body = mlir_region_op!(
            context, &module_body,
            ContractOperation
                .sym_name(StringAttribute::new(context.mlir_context, &contract_name))
                .kind(solx_mlir::ContractKind::Contract.attribute(context.mlir_context));
            body_region
        );

        for member in self.members().iter() {
            let ContractMember::StateVariableDefinition(state_variable) = member else {
                continue;
            };
            let Some(slot) = storage_layout.get(&state_variable.node_id()) else {
                continue;
            };
            let element_type =
                TypeConversion::resolve_state_variable_type(&state_variable, context);
            let slot_attribute: IntegerAttribute =
                Attribute::parse(context.mlir_context, &format!("{} : i256", slot.slot))
                    .expect("valid slot literal")
                    .try_into()
                    .expect("slot literal is an integer attribute");
            let byte_offset_attribute = IntegerAttribute::new(
                IntegerType::new(context.mlir_context, solx_utils::BIT_LENGTH_X32 as u32).into(),
                slot.byte_offset.into(),
            );
            mlir_op_void!(
                context,
                &contract_body,
                StateVarOperation
                    .sym_name(StringAttribute::new(context.mlir_context, &slot.name))
                    .r#type(TypeAttribute::new(element_type))
                    .slot(slot_attribute)
                    .byte_offset(byte_offset_attribute)
            );
        }

        context.current_contract_type = Some(contract_type);
        self.emit_constructor(
            &FunctionEmitter::new(context, self, &storage_layout),
            &contract_body,
        );
        context.current_contract_type = None;

        for function in self.functions() {
            context.current_contract_type = Some(contract_type);
            function.emit(
                &FunctionEmitter::new(context, self, &storage_layout),
                &contract_body,
            );
            context.current_contract_type = None;
        }
    }
}
