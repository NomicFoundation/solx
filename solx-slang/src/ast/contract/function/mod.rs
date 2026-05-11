//!
//! Function definition lowering to Sol dialect MLIR.
//!

pub mod expression;
pub mod statement;

use std::collections::HashMap;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use slang_solidity::backend::abi::AbiEntry;
use slang_solidity::backend::ir::ast::ElementaryType;
use slang_solidity::backend::ir::ast::Expression;
use slang_solidity::backend::ir::ast::FunctionDefinition;
use slang_solidity::backend::ir::ast::FunctionKind;
use slang_solidity::backend::ir::ast::Type as SlangType;
use slang_solidity::backend::ir::ast::TypeName;
use slang_solidity::cst::NodeId;

use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::StateMutability;

use self::expression::ExpressionEmitter;
use self::expression::call::type_conversion::TypeConversion;
use self::statement::StatementEmitter;

/// State variable with an inline initializer, to be emitted at the top of the
/// constructor body before the user-written constructor statements run.
pub struct StateVarInitializer {
    /// Storage slot for the state variable.
    pub slot: u64,
    /// Slang semantic type of the state variable.
    pub slang_type: SlangType,
    /// Initializer expression from the source-level `T x = <expr>;` declaration.
    pub initializer: Expression,
}

/// Lowers a Solidity function definition to a `sol.func` operation.
pub struct FunctionEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state Context<'context>,
    /// State variable node ID to storage slot mapping.
    storage_layout: &'state HashMap<NodeId, u64>,
    /// State variable initializers, emitted at the top of the constructor.
    state_var_initializers: &'state [StateVarInitializer],
}

impl<'state, 'context> FunctionEmitter<'state, 'context> {
    /// Creates a new function emitter.
    pub fn new(
        state: &'state Context<'context>,
        storage_layout: &'state HashMap<NodeId, u64>,
        state_var_initializers: &'state [StateVarInitializer],
    ) -> Self {
        Self {
            state,
            storage_layout,
            state_var_initializers,
        }
    }

    /// Emits a `sol.func` for the given function definition into the given
    /// contract body block.
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
        contract_body: &melior::ir::BlockRef<'context, '_>,
    ) -> anyhow::Result<String> {
        let Some(ref body) = function.body() else {
            // Abstract or interface function — no codegen needed.
            return Ok(Self::mlir_function_name(function));
        };

        let parameters = function.parameters();
        let mlir_name = Self::mlir_function_name(function);

        let (mlir_parameter_types, result_types) =
            TypeConversion::resolve_function_types(function, &self.state.builder);

        let selector = function.compute_selector();

        let state_mutability = Self::map_state_mutability(function);

        let mlir_kind = match function.kind() {
            FunctionKind::Constructor => Some(solx_mlir::FunctionKind::Constructor),
            FunctionKind::Fallback | FunctionKind::Unnamed => {
                Some(solx_mlir::FunctionKind::Fallback)
            }
            FunctionKind::Receive => Some(solx_mlir::FunctionKind::Receive),
            FunctionKind::Regular => None,
            FunctionKind::Modifier => unreachable!("modifiers are filtered before emission"),
        };

        let function_entry_block = self.state.builder.emit_sol_func(
            &mlir_name,
            &mlir_parameter_types,
            &result_types,
            selector,
            state_mutability,
            mlir_kind,
            contract_body,
        );

        let mut environment = Environment::new();

        // Create allocas for parameters and bind to environment.
        for (index, parameter) in parameters.iter().enumerate() {
            let parameter_name = parameter
                .name()
                .map(|id| id.name())
                .unwrap_or_else(|| "_".to_owned());
            let parameter_type = mlir_parameter_types[index];
            let parameter_value: melior::ir::Value<'context, '_> =
                function_entry_block.argument(index)?.into();
            let pointer = self
                .state
                .builder
                .emit_sol_alloca(parameter_type, &function_entry_block);
            self.state
                .builder
                .emit_sol_store(parameter_value, pointer, &function_entry_block);

            environment.define_variable(parameter_name, pointer, parameter_type);
        }

        let region = function_entry_block
            .parent_region()
            .expect("entry block belongs to a region");
        let mut current_block = function_entry_block;

        // State variable initializers run at the top of the constructor body,
        // before any user-written statements, mirroring solc's MLIR layout.
        if matches!(function.kind(), FunctionKind::Constructor) {
            for initializer in self.state_var_initializers {
                current_block =
                    self.emit_state_var_initializer(initializer, &environment, current_block)?;
            }
        }

        let mut terminated = false;
        for statement in body.statements().iter() {
            let mut emitter = StatementEmitter::new(
                self.state,
                &mut environment,
                &region,
                self.storage_layout,
                &result_types,
            );
            match emitter.emit(&statement, current_block)? {
                Some(next) => current_block = next,
                None => {
                    terminated = true;
                    break;
                }
            }
        }

        if !terminated {
            self.emit_default_return(&result_types, &current_block);
        }

        Ok(mlir_name)
    }

    /// Emits a single state variable initializer at the top of the constructor.
    ///
    /// Reference-typed slots (struct, array, mapping, bytes, string) are
    /// written via `sol.copy` from the evaluated value into the storage
    /// reference. Primitive slots use `sol.store` after casting to the
    /// declared element type.
    pub fn emit_state_var_initializer<'block>(
        &self,
        initializer: &StateVarInitializer,
        environment: &Environment<'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>>
    where
        'context: 'block,
    {
        let builder = &self.state.builder;
        let element_type =
            TypeConversion::resolve_slang_type(&initializer.slang_type, None, builder);
        let emitter = ExpressionEmitter::new(self.state, environment, self.storage_layout, true);
        let (value, block) = emitter.emit_value(&initializer.initializer, block)?;
        let slot_name = format!("slot_{}", initializer.slot);
        let is_ref = matches!(
            initializer.slang_type,
            SlangType::Struct(_)
                | SlangType::Array(_)
                | SlangType::FixedSizeArray(_)
                | SlangType::Mapping(_)
                | SlangType::Bytes(_)
                | SlangType::String(_)
        );
        if is_ref {
            let storage_addr = builder.emit_sol_addr_of(&slot_name, element_type, &block);
            builder.emit_sol_copy(value, storage_addr, &block);
        } else {
            let ptr_type = builder
                .types
                .pointer(element_type, solx_utils::DataLocation::Storage);
            let storage_addr = builder.emit_sol_addr_of(&slot_name, ptr_type, &block);
            let cast = TypeConversion::from_target_type(element_type, builder)
                .emit(value, builder, &block);
            builder.emit_sol_store(cast, storage_addr, &block);
        }
        Ok(block)
    }

    /// Builds the MLIR function name as `{name}({types})`.
    ///
    /// Uses slang's ABI canonical types when available (external functions),
    /// falls back to AST-based type names for internal/private functions.
    pub fn mlir_function_name(function: &FunctionDefinition) -> String {
        let name = Self::mlir_base_name(function);

        if let Some(AbiEntry::Function { inputs, .. }) = function.compute_abi_entry() {
            let types: Vec<&str> = inputs.iter().map(|input| input.r#type.as_str()).collect();
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
                        format!("{base}[{}]", decimal.literal().text)
                    }
                    Some(Expression::HexNumberExpression(hex)) => {
                        format!("{base}[{}]", hex.literal().text)
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
            ElementaryType::BoolKeyword => "bool".to_owned(),
            ElementaryType::ByteKeyword => "byte".to_owned(),
            ElementaryType::StringKeyword => "string".to_owned(),
            ElementaryType::UintKeyword(terminal)
            | ElementaryType::IntKeyword(terminal)
            | ElementaryType::BytesKeyword(terminal)
            | ElementaryType::FixedKeyword(terminal)
            | ElementaryType::UfixedKeyword(terminal) => terminal.text.clone(),
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
            FunctionKind::Fallback | FunctionKind::Unnamed => "fallback".to_owned(),
            FunctionKind::Receive => "receive".to_owned(),
            FunctionKind::Constructor => "constructor".to_owned(),
            FunctionKind::Modifier => unreachable!("modifiers are not emitted as functions"),
        }
    }

    /// Emits a default `sol.return` if the block lacks a terminator.
    ///
    /// Emits one typed zero constant per return type and terminates the block.
    fn emit_default_return(
        &self,
        result_types: &[Type<'context>],
        block: &melior::ir::BlockRef<'context, '_>,
    ) {
        if block.terminator().is_some() {
            return;
        }
        let zeros: Vec<_> = result_types
            .iter()
            .map(|ty| self.state.builder.emit_sol_constant(0, *ty, block))
            .collect();
        self.state.builder.emit_sol_return(&zeros, block);
    }

    /// Maps Slang's `FunctionMutability` to the Sol dialect's `StateMutability`.
    ///
    /// Required because the Sol dialect defines its own mutability enum
    /// independently of the Slang AST representation.
    fn map_state_mutability(function: &FunctionDefinition) -> StateMutability {
        use slang_solidity::backend::ir::ast::FunctionMutability;
        match function.mutability() {
            FunctionMutability::Pure => StateMutability::Pure,
            FunctionMutability::View => StateMutability::View,
            FunctionMutability::Payable => StateMutability::Payable,
            FunctionMutability::NonPayable => StateMutability::NonPayable,
        }
    }
}
