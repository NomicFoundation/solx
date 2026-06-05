//!
//! Local variable declaration statement lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MultiTypedDeclaration;
use slang_solidity_v2::ast::SingleTypedDeclaration;
use slang_solidity_v2::ast::Type as SlangType;
use slang_solidity_v2::ast::VariableDeclarationStatement;
use slang_solidity_v2::ast::VariableDeclarationTarget;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Lowers a local variable declaration.
    pub fn emit_variable_declaration(
        &mut self,
        declaration: &VariableDeclarationStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        match declaration.target() {
            VariableDeclarationTarget::SingleTypedDeclaration(single) => {
                self.emit_single_typed_declaration(&single, block)
            }
            VariableDeclarationTarget::MultiTypedDeclaration(multi) => {
                self.emit_multi_typed_declaration(&multi, block)
            }
        }
    }

    /// Lowers a tuple deconstruction declaration `(T a, T b) = rhs;`.
    ///
    /// The right-hand side — a tuple expression or a multi-result call — yields
    /// one value per slot; each named slot allocates a fresh stack slot, stores
    /// the coerced value, and binds the name, while an empty slot (`(, b)`)
    /// discards its value.
    fn emit_multi_typed_declaration(
        &mut self,
        declaration: &MultiTypedDeclaration,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let elements = declaration.elements();
        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let (values, block) = match declaration.value() {
            Expression::FunctionCallExpression(call) => {
                CallEmitter::new(&emitter).emit_function_call_results(&call, block)?
            }
            expression => emitter.emit_component_values(&expression, block)?,
        };

        for (element, value) in elements.iter().zip(values) {
            let Some(declared) = element.member() else {
                continue;
            };
            let name = declared.name().name();
            let builder = &self.state.builder;
            let declared_type = TypeConversion::resolve_slang_type(
                &declared
                    .get_type()
                    .expect("the binder types every deconstructed local"),
                None,
                builder,
            );
            let value = TypeConversion::from_target_type(declared_type, builder)
                .emit(value, builder, &block);
            let pointer = builder.emit_sol_alloca(declared_type, &block);
            builder.emit_sol_store(value, pointer, &block);
            self.environment
                .define_variable(name, pointer, declared_type);
        }
        Ok(Some(block))
    }

    /// Lowers a single `T name [= value];` declaration: allocate a stack slot,
    /// store the initializer (or a typed zero when omitted), and bind the name.
    fn emit_single_typed_declaration(
        &mut self,
        declaration: &SingleTypedDeclaration,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let name = declaration.declaration().name().name();
        let declared_slang_type = declaration
            .declaration()
            .get_type()
            .expect("binder types every variable declaration");
        let declared_type =
            TypeConversion::resolve_slang_type(&declared_slang_type, None, &self.state.builder);

        // An explicit initializer is evaluated and cast before the alloca, to
        // match solc's emission order (value → alloca → store).
        let (block, initial_value) = match declaration.value() {
            Some(initializer) => {
                let emitter = ExpressionEmitter::new(
                    self.state,
                    self.environment,
                    self.storage_layout,
                    self.checked,
                );
                let (value, block) = emitter.emit_value(&initializer, block)?;
                let value = TypeConversion::from_target_type(declared_type, &self.state.builder)
                    .emit(value, &self.state.builder, &block);
                (block, Some(value))
            }
            None => (block, None),
        };

        let pointer = self.state.builder.emit_sol_alloca(declared_type, &block);
        match initial_value {
            Some(value) => self.state.builder.emit_sol_store(value, pointer, &block),
            None => {
                self.emit_zero_initialization(&declared_slang_type, declared_type, pointer, &block)
            }
        }
        self.environment
            .define_variable(name, pointer, declared_type);
        Ok(Some(block))
    }

    /// Default-initialises a local declared without an initializer, matching
    /// Solidity's implicit zero value: a fixed-size array / struct
    /// (`T[n] memory a;`, `S memory s;`) becomes a freshly allocated zero-filled
    /// memory aggregate, a dynamic `string` / `bytes` an empty (length-0)
    /// buffer, and an integer a zero constant. Other non-integer types (function
    /// pointers, user-defined value types, …) keep the raw `sol.alloca` pointer,
    /// which the backend zero-fills.
    fn emit_zero_initialization(
        &self,
        declared_slang_type: &SlangType,
        declared_type: Type<'context>,
        pointer: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) {
        let builder = &self.state.builder;
        match declared_slang_type {
            SlangType::FixedSizeArray(_) | SlangType::Struct(_) => {
                let allocated = builder.emit_sol_malloc_zeroed(declared_type, block);
                builder.emit_sol_store(allocated, pointer, block);
            }
            SlangType::String(_) | SlangType::Bytes(_) => {
                let zero = builder.emit_sol_constant(0, builder.types.ui256, block);
                let allocated = builder.emit_sol_malloc_sized(declared_type, zero, block);
                builder.emit_sol_store(allocated, pointer, block);
            }
            _ if IntegerType::try_from(declared_type).is_ok() => {
                let zero = builder.emit_sol_constant(0, declared_type, block);
                builder.emit_sol_store(zero, pointer, block);
            }
            // Other non-integer types are left as the raw `sol.alloca` pointer.
            _ => {}
        }
    }
}
