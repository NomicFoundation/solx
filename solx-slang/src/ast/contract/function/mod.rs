//!
//! Function definition lowering to Sol dialect MLIR.
//!

pub mod expression;
pub mod statement;
pub mod storage_slot;

use std::collections::HashMap;

use slang_solidity_v2::abi::AbiEntry;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ElementaryType;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::TypeName;

use solx_mlir::Block;
use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::Function;
use solx_mlir::Place;
use solx_mlir::StateMutability;
use solx_mlir::Type;
use solx_mlir::Value;

use self::expression::ExpressionEmitter;
use self::expression::call::type_conversion::TypeConversion;
use self::statement::StatementEmitter;
use self::storage_slot::StorageSlot;

/// Lowers a Solidity function definition to a `sol.func` operation.
pub struct FunctionEmitter<'state> {
    /// Containing contract.
    contract: &'state ContractDefinition,
    /// State variable node ID to `(slot, byte_offset)` mapping. The byte
    /// offset is zero for unpacked variables and non-zero for variables
    /// packed into a shared slot.
    storage_layout: &'state HashMap<NodeId, StorageSlot>,
}

impl<'state> FunctionEmitter<'state> {
    /// Creates a new function emitter.
    pub fn new(
        contract: &'state ContractDefinition,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
    ) -> Self {
        Self {
            contract,
            storage_layout,
        }
    }

    /// Emits a `sol.func` for the given function definition into the given
    /// contract body block.
    ///
    /// # Errors
    ///
    /// Returns an error if the function body contains unsupported statements.
    pub fn emit_sol<'context>(
        &self,
        function: &FunctionDefinition,
        contract_body: Block<'context>,
        context: &mut Context<'context>,
    ) -> anyhow::Result<String> {
        let Some(ref body) = function.body() else {
            return Ok(Self::mlir_function_name(function));
        };

        let parameters = function.parameters();
        let mlir_name = Self::mlir_function_name(function);

        let (mlir_parameter_types, result_types) =
            TypeConversion::resolve_function_types(function, context);

        let selector = function.compute_selector();

        let state_mutability = Self::map_state_mutability(function);

        let mlir_kind = match function.kind() {
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
        .define(
            selector,
            state_mutability,
            mlir_kind,
            context,
            contract_body,
        );
        context.current_block = Some(function_entry_block);

        let mut environment = Environment::new();

        for (index, parameter) in parameters.iter().enumerate() {
            let parameter_name = parameter
                .name()
                .map(|id| id.name())
                .unwrap_or_else(|| "_".to_owned());
            let parameter_type = mlir_parameter_types[index];
            let parameter_value = function_entry_block.argument(index);
            let pointer = Place::stack(parameter_type, context);
            pointer.store(parameter_value, context);

            environment.define_variable(parameter_name, pointer, parameter_type);
        }

        let mut return_slots: Vec<Option<Place<'context>>> = Vec::new();
        if let Some(returns) = function.returns() {
            for (index, parameter) in returns.iter().enumerate() {
                let Some(identifier) = parameter.name() else {
                    return_slots.push(None);
                    continue;
                };
                let return_type = result_types[index];
                let pointer = Place::stack(return_type, context);
                // TODO: replace with a typed-zero helper covering address, fixed-bytes, and
                // memory-resident types.
                if return_type.is_integer() {
                    let zero = Value::constant(0, return_type, context);
                    pointer.store(zero, context);
                } else {
                    unimplemented!(
                        "zero-initialization for non-integer named return: {return_type}"
                    );
                }
                environment.define_variable(identifier.name(), pointer, return_type);
                return_slots.push(Some(pointer));
            }
        }

        if matches!(function.kind(), FunctionKind::Constructor) {
            let emitter = ExpressionEmitter::new(&environment, self.storage_layout, true);
            emitter.emit_state_var_initializers(self.contract, context)?;
        }

        for statement in body.statements().iter() {
            let mut emitter =
                StatementEmitter::new(&mut environment, self.storage_layout, &result_types);
            emitter.emit(&statement, context)?;
            if context.current_block().is_terminated() {
                break;
            }
        }

        if !context.current_block().is_terminated() {
            Self::emit_default_return(&result_types, &return_slots, context);
        }

        Ok(mlir_name)
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
    pub fn emit_constructor<'context>(
        &self,
        contract_body: Block<'context>,
        context: &mut Context<'context>,
    ) -> anyhow::Result<()> {
        if let Some(constructor) = self.contract.constructor() {
            self.emit_sol(&constructor, contract_body, context)?;
            return Ok(());
        }
        let entry = Function::new("constructor()".to_owned(), Vec::new(), Vec::new()).define(
            None,
            StateMutability::NonPayable,
            Some(solx_mlir::FunctionKind::Constructor),
            context,
            contract_body,
        );
        context.current_block = Some(entry);
        let environment = Environment::new();
        let emitter = ExpressionEmitter::new(&environment, self.storage_layout, true);
        emitter.emit_state_var_initializers(self.contract, context)?;
        let block = context.current_block();
        block.r#return(&[], context);
        Ok(())
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

    /// Emits a default `sol.return`.
    ///
    /// For each return position, loads the current value from the named-return
    /// slot when one was allocated, otherwise materializes a typed zero
    /// constant.
    fn emit_default_return<'context>(
        result_types: &[Type<'context>],
        return_slots: &[Option<Place<'context>>],
        context: &mut Context<'context>,
    ) {
        let mut values: Vec<Value<'context>> = Vec::with_capacity(result_types.len());
        for (index, result_type) in result_types.iter().enumerate() {
            let value = match return_slots.get(index).copied().flatten() {
                Some(pointer) => pointer.load(*result_type, context),
                None => Value::constant(0, *result_type, context),
            };
            values.push(value);
        }
        let block = context.current_block();
        block.r#return(&values, context);
    }

    /// Maps Slang's `FunctionMutability` to the Sol dialect's `StateMutability`.
    ///
    /// Required because the Sol dialect defines its own mutability enum
    /// independently of the Slang AST representation.
    fn map_state_mutability(function: &FunctionDefinition) -> StateMutability {
        use slang_solidity_v2::ast::FunctionMutability;
        match function.attributes().mutability() {
            FunctionMutability::Pure => StateMutability::Pure,
            FunctionMutability::View => StateMutability::View,
            FunctionMutability::Payable => StateMutability::Payable,
            FunctionMutability::NonPayable => StateMutability::NonPayable,
        }
    }
}
