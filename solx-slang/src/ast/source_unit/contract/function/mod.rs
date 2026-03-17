//!
//! Function definition lowering to Sol dialect MLIR.
//!

/// Expression lowering to MLIR SSA values.
pub(crate) mod expression;
/// Statement lowering to MLIR operations.
pub(crate) mod statement;

use melior::ir::BlockLike;
use slang_solidity::backend::abi::AbiEntry;
use slang_solidity::backend::ir::ast::ElementaryType;
use slang_solidity::backend::ir::ast::Expression;
use slang_solidity::backend::ir::ast::FunctionDefinition;
use slang_solidity::backend::ir::ast::FunctionKind;
use slang_solidity::backend::ir::ast::TypeName;

use solx_mlir::Environment;
use solx_mlir::MlirContext;
use solx_mlir::StateMutability;

use self::expression::ExpressionEmitter;
use self::statement::StatementEmitter;

/// Lowers a Solidity function definition to a `sol.func` operation.
pub(crate) struct FunctionEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state MlirContext<'context>,
}

impl<'state, 'context> FunctionEmitter<'state, 'context> {
    /// Creates a new function emitter.
    pub(crate) fn new(state: &'state MlirContext<'context>) -> Self {
        Self { state }
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
    pub(crate) fn emit_sol(
        &self,
        function: &FunctionDefinition,
        contract_body: &melior::ir::BlockRef<'context, '_>,
    ) -> anyhow::Result<String> {
        let parameters = function.parameters();
        let mlir_name = Self::mlir_function_name(function);

        let i256 = self.state.i256();

        let has_returns = function
            .returns()
            .is_some_and(|returns| !returns.is_empty());

        let mlir_parameter_types: Vec<melior::ir::Type<'context>> =
            (0..parameters.len()).map(|_| i256).collect();
        let result_types: Vec<melior::ir::Type<'context>> =
            if has_returns { vec![i256] } else { vec![] };

        let selector = function.compute_selector();

        let state_mutability = Self::map_state_mutability(function);

        let function_entry_block = self.state.emit_sol_func(
            &mlir_name,
            &mlir_parameter_types,
            &result_types,
            selector,
            state_mutability,
            contract_body,
        );

        let mut environment = Environment::new();

        // Create allocas for parameters and bind to environment.
        for (i, parameter) in parameters.iter().enumerate() {
            let parameter_name = parameter
                .name()
                .map(|id| id.name())
                .unwrap_or_else(|| "_".to_owned());
            let parameter_value: melior::ir::Value<'context, '_> =
                function_entry_block.argument(i)?.into();

            let region = function_entry_block
                .parent_region()
                .expect("entry block belongs to a region");
            let expression_emitter = ExpressionEmitter::new(self.state, &environment, &region);
            let pointer = expression_emitter.emit_alloca(&function_entry_block);
            expression_emitter.emit_store(parameter_value, pointer, &function_entry_block);

            if matches!(
                parameter.type_name(),
                TypeName::ElementaryType(ElementaryType::IntKeyword(_))
            ) {
                environment.mark_signed(&parameter_name);
            }
            environment.define_variable(parameter_name, pointer);
        }

        if let Some(ref body) = function.body() {
            let region = function_entry_block
                .parent_region()
                .expect("entry block belongs to a region");
            let mut current_block = function_entry_block;
            let mut terminated = false;
            for statement in body.statements().iter() {
                let mut emitter = StatementEmitter::new(self.state, &mut environment, &region);
                match emitter.emit(&statement, current_block)? {
                    Some(next) => current_block = next,
                    None => {
                        terminated = true;
                        break;
                    }
                }
            }

            if !terminated {
                self.emit_default_return(has_returns, &current_block);
            }
        } else {
            self.emit_default_return(has_returns, &function_entry_block);
        }

        Ok(mlir_name)
    }

    /// Builds the mangled MLIR function name `solx.fn.{name}({types})`.
    ///
    /// Uses slang's ABI canonical types when available (external functions),
    /// falls back to AST-based type names for internal/private functions.
    pub(crate) fn mlir_function_name(function: &FunctionDefinition) -> String {
        let name = Self::mlir_base_name(function);

        if let Some(AbiEntry::Function { inputs, .. }) = function.compute_abi_entry() {
            let types: Vec<&str> = inputs.iter().map(|p| p.r#type.as_str()).collect();
            return format!("solx.fn.{name}({})", types.join(","));
        }

        let types: Vec<String> = function
            .parameters()
            .iter()
            .map(|p| Self::type_name_text(&p.type_name()))
            .collect();
        format!("solx.fn.{name}({})", types.join(","))
    }

    /// Returns the base name for a function's MLIR symbol, using its kind to
    /// generate names for special functions (fallback, receive) that have no
    /// Solidity-level identifier.
    pub(crate) fn mlir_base_name(function: &FunctionDefinition) -> String {
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

    /// Maps Solidity function state mutability to Sol dialect `StateMutability`.
    fn map_state_mutability(function: &FunctionDefinition) -> StateMutability {
        use slang_solidity::backend::ir::ast::FunctionMutability;
        match function.mutability() {
            FunctionMutability::Pure => StateMutability::Pure,
            FunctionMutability::View => StateMutability::View,
            FunctionMutability::Payable => StateMutability::Payable,
            FunctionMutability::NonPayable => StateMutability::NonPayable,
        }
    }

    /// Emits a default `sol.return` if the block lacks a terminator.
    fn emit_default_return(&self, has_returns: bool, block: &melior::ir::BlockRef<'context, '_>) {
        if block.terminator().is_some() {
            return;
        }
        if has_returns {
            let zero = self.state.emit_sol_constant(0, block);
            self.state.emit_sol_return(&[zero], block);
        } else {
            self.state.emit_sol_return(&[], block);
        }
    }
}
