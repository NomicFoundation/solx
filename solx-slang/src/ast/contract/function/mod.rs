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
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ElementaryType;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::TypeName;

use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::Function;
use solx_mlir::Pointer;
use solx_mlir::StateMutability;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::ReturnOperation;

use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::contract::function::storage_slot::StorageSlot;
use crate::ast::emit::emit_constructor::EmitConstructor;
use crate::ast::emit::emit_function::EmitFunction;
use crate::ast::emit::emit_statement::EmitStatement;

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
                Some(pointer) => Pointer::new(pointer)
                    .load(AstType::new(*result_type), self.state, block)
                    .into_mlir(),
                None => {
                    AstValue::constant(0, AstType::new(*result_type), self.state, block).into_mlir()
                }
            };
            values.push(value);
        }
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
        contract_body: &BlockRef<'context, '_>,
    ) -> String {
        let mlir_name = FunctionEmitter::mlir_function_name(self);
        let Some(ref body) = self.body() else {
            return mlir_name;
        };

        let parameters = self.parameters();

        let (mlir_parameter_types, result_types) =
            TypeConversion::resolve_function_types(self, emitter.state);

        let selector = self.compute_selector();

        let state_mutability = FunctionEmitter::map_state_mutability(self);

        let mlir_kind = match self.kind() {
            FunctionKind::Constructor => Some(solx_mlir::FunctionKind::Constructor),
            FunctionKind::Fallback => Some(solx_mlir::FunctionKind::Fallback),
            FunctionKind::Receive => Some(solx_mlir::FunctionKind::Receive),
            FunctionKind::Regular => None,
            FunctionKind::Modifier => unreachable!("modifiers are filtered before emission"),
        };

        let function_entry_block = Function::new(
            mlir_name.clone(),
            mlir_parameter_types.clone(),
            result_types.clone(),
        )
        .define(selector, state_mutability, mlir_kind, emitter.state, contract_body);

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

        let mut return_slots: Vec<Option<Value<'context, '_>>> = Vec::new();
        if let Some(returns) = self.returns() {
            for (index, parameter) in returns.iter().enumerate() {
                let Some(identifier) = parameter.name() else {
                    return_slots.push(None);
                    continue;
                };
                let return_type = result_types[index];
                let pointer =
                    Pointer::stack(AstType::new(return_type), emitter.state, &function_entry_block);
                // TODO: replace with a typed-zero helper covering address, fixed-bytes, and
                // memory-resident types (e.g. `0x60` for empty `string`/`bytes` memory).
                if IntegerType::try_from(return_type).is_ok() {
                    let zero = AstValue::constant(
                        0,
                        AstType::new(return_type),
                        emitter.state,
                        &function_entry_block,
                    );
                    pointer.store(zero, emitter.state, &function_entry_block);
                } else {
                    unimplemented!(
                        "zero-initialization for non-integer named return: {return_type}"
                    );
                }
                let pointer = pointer.into_mlir();
                environment.define_variable(identifier.name(), pointer, return_type);
                return_slots.push(Some(pointer));
            }
        }

        let region = function_entry_block
            .parent_region()
            .expect("entry block belongs to a region");
        let mut current_block = function_entry_block;

        if matches!(self.kind(), FunctionKind::Constructor) {
            let expression_context = ExpressionContext::new(
                emitter.state,
                &environment,
                emitter.storage_layout,
                true,
            );
            current_block =
                expression_context.emit_state_var_initializers(emitter.contract, current_block);
        }

        let mut terminated = false;
        for statement in body.statements().iter() {
            let mut statement_context = StatementContext::new(
                emitter.state,
                &mut environment,
                &region,
                emitter.storage_layout,
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
            emitter.emit_default_return(&result_types, &return_slots, &current_block);
        }

        mlir_name
    }
}

impl EmitConstructor for ContractDefinition {
    /// Emits the contract's `constructor()` `sol.func`, threaded via the shared [`FunctionEmitter`].
    fn emit_constructor<'context>(
        &self,
        emitter: &FunctionEmitter<'_, 'context>,
        contract_body: &BlockRef<'context, '_>,
    ) {
        if let Some(constructor) = self.constructor() {
            constructor.emit(emitter, contract_body);
            return;
        }
        let entry = Function::new("constructor()".to_owned(), Vec::new(), Vec::new()).define(
            None,
            StateMutability::NonPayable,
            Some(solx_mlir::FunctionKind::Constructor),
            emitter.state,
            contract_body,
        );
        let environment = Environment::new();
        let expression_context =
            ExpressionContext::new(emitter.state, &environment, emitter.storage_layout, true);
        let block = expression_context.emit_state_var_initializers(emitter.contract, entry);
        mlir_op_void!(emitter.state, &block, ReturnOperation.operands(&[]));
    }
}
