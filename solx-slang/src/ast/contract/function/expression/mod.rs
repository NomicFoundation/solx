//!
//! Expression lowering to MLIR SSA values.
//!

pub mod arithmetic;
pub mod assignment;
pub mod call;
pub mod logical;
pub mod operator;
pub mod storage;

use std::collections::HashMap;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::r#type::IntegerType;
use slang_solidity::backend::ir::ast::Definition;
use slang_solidity::backend::ir::ast::Expression;
use slang_solidity::backend::ir::ast::MemberAccessExpression;
use slang_solidity::backend::ir::ast::Type as SlangType;
use slang_solidity::backend::types::DataLocation as SlangDataLocation;
use slang_solidity::cst::NodeId;

use solx_mlir::CmpPredicate;
use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::ods::sol::ThisOperation;

use self::call::type_conversion::TypeConversion;

/// Lowers Solidity expressions to MLIR SSA values.
pub struct ExpressionEmitter<'state, 'context, 'block> {
    /// The shared MLIR context.
    pub state: &'state Context<'context>,
    /// Variable environment.
    pub environment: &'state Environment<'context, 'block>,
    /// State variable node ID to storage slot mapping.
    pub storage_layout: &'state HashMap<NodeId, u64>,
    /// Whether arithmetic operations use checked variants (`sol.cadd` etc.).
    ///
    /// `true` by default (Solidity 0.8+). Set to `false` inside `unchecked {}`
    /// blocks and for-loop step expressions.
    pub checked: bool,
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Creates a new expression emitter.
    pub fn new(
        state: &'state Context<'context>,
        environment: &'state Environment<'context, 'block>,
        storage_layout: &'state HashMap<NodeId, u64>,
        checked: bool,
    ) -> Self {
        Self {
            state,
            environment,
            storage_layout,
            checked,
        }
    }

    /// Emits MLIR for an expression that must produce a value.
    ///
    /// Delegates to [`Self::emit`] and returns an error for void expressions
    /// (e.g. calls to functions with no return value).
    pub fn emit_value(
        &self,
        expression: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (value, block) = self.emit(expression, block)?;
        let value = value.ok_or_else(|| anyhow::anyhow!("expression produced no value"))?;
        Ok((value, block))
    }

    /// Emits MLIR for an expression, appending operations to `block`.
    ///
    /// Returns `None` for void expressions (calls with no return value).
    /// Use [`Self::emit_value`] when a value is required.
    ///
    /// # Errors
    ///
    /// Returns an error if the expression contains unsupported constructs.
    pub fn emit(
        &self,
        expression: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        match expression {
            Expression::DecimalNumberExpression(decimal_number) => {
                let value = decimal_number.integer_value().ok_or_else(|| {
                    anyhow::anyhow!(
                        "decimal literal cannot be lowered: it must evaluate to an integer \
                         after applying any units"
                    )
                })?;
                let result_type = self
                    .resolve_slang_type(decimal_number.get_type())
                    .expect("binder types every decimal literal node");
                let constant = self
                    .state
                    .builder
                    .emit_constant(&value, result_type, &block);
                Ok((Some(constant), block))
            }
            Expression::HexNumberExpression(hex_number) => {
                let value = hex_number
                    .integer_value()
                    .expect("hex literals always evaluate to integers");
                let result_type = self
                    .resolve_slang_type(hex_number.get_type())
                    .expect("binder types every hex literal node");
                let constant = self
                    .state
                    .builder
                    .emit_constant(&value, result_type, &block);
                Ok((Some(constant), block))
            }
            Expression::TrueKeyword => {
                let constant = self.state.builder.emit_bool(true, &block);
                Ok((Some(constant), block))
            }
            Expression::FalseKeyword => {
                let constant = self.state.builder.emit_bool(false, &block);
                Ok((Some(constant), block))
            }
            Expression::ThisKeyword => {
                let contract_type = self
                    .state
                    .current_contract_type
                    .ok_or_else(|| anyhow::anyhow!("sol.this emitted outside a contract"))?;
                let operation = ThisOperation::builder(
                    self.state.builder.context,
                    self.state.builder.unknown_location,
                )
                .addr(contract_type)
                .build();
                let value = block
                    .append_operation(operation.into())
                    .result(0)
                    .expect("sol.this always produces one result")
                    .into();
                Ok((Some(value), block))
            }
            Expression::StringExpression(string_expression) => {
                let bytes = string_expression.value();
                let text = std::str::from_utf8(&bytes).expect("string literal is valid UTF-8");
                let value = self.state.builder.emit_sol_string_lit(text, &block);
                Ok((Some(value), block))
            }
            Expression::Identifier(identifier) => {
                let name = identifier.name();
                match identifier.resolve_to_definition() {
                    Some(Definition::StateVariable(state_variable)) => {
                        let slot = self
                            .storage_layout
                            .get(&state_variable.node_id())
                            .ok_or_else(|| {
                                anyhow::anyhow!("unregistered state variable: {name}")
                            })?;
                        let slang_type = state_variable.get_type().ok_or_else(|| {
                            anyhow::anyhow!("unresolved type for state variable: {name}")
                        })?;
                        let element_type = TypeConversion::resolve_slang_type(
                            &slang_type,
                            None,
                            &self.state.builder,
                        );
                        // A reference-type state variable (struct, array,
                        // mapping, bytes, string) in Storage IS its slot
                        // region — `sol.addr_of` returns the type directly and
                        // there is no value to `sol.load`. Mirror solc.
                        let value = if matches!(
                            slang_type,
                            SlangType::Struct(_)
                                | SlangType::Array(_)
                                | SlangType::FixedSizeArray(_)
                                | SlangType::Mapping(_)
                                | SlangType::Bytes(_)
                                | SlangType::String(_)
                        ) {
                            self.state.builder.emit_sol_addr_of(
                                &format!("slot_{slot}"),
                                element_type,
                                &block,
                            )
                        } else {
                            self.emit_storage_load(*slot, element_type, &block)?
                        };
                        Ok((Some(value), block))
                    }
                    Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                        let (pointer, element_type) =
                            self.environment.variable_with_type(&name).ok_or_else(|| {
                                anyhow::anyhow!("unregistered local variable: {name}")
                            })?;
                        let value =
                            self.state
                                .builder
                                .emit_sol_load(pointer, element_type, &block)?;
                        Ok((Some(value), block))
                    }
                    None => anyhow::bail!("unresolved identifier: {name}"),
                    Some(_) => anyhow::bail!("unsupported identifier reference: {name}"),
                }
            }
            Expression::AssignmentExpression(assign) => self
                .emit_assignment(assign, block)
                .map(|(value, block)| (Some(value), block)),
            Expression::AdditiveExpression(expression) => {
                let result_type = self.resolve_slang_type(expression.get_type());
                let left = expression.left_operand();
                let right = expression.right_operand();
                let operator = expression.operator();
                self.emit_binary_op(&left, &right, &operator.text, result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::MultiplicativeExpression(expression) => {
                let result_type = self.resolve_slang_type(expression.get_type());
                let left = expression.left_operand();
                let right = expression.right_operand();
                let operator = expression.operator();
                self.emit_binary_op(&left, &right, &operator.text, result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::ExponentiationExpression(expression) => {
                let target_type = self.resolve_slang_type(expression.get_type());
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_binary_op(&left, &right, "**", target_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::EqualityExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                let operator = expression.operator();
                self.emit_comparison(&left, &right, &operator.text, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::InequalityExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                let operator = expression.operator();
                self.emit_comparison(&left, &right, &operator.text, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::AndExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_and(&left, &right, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::OrExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_or(&left, &right, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::PostfixExpression(expression) => {
                let operand = expression.operand();
                let operator = expression.operator();
                self.emit_postfix(&operand, &operator.text, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::PrefixExpression(expression) => {
                let operator = expression.operator();
                let result_type = self.resolve_slang_type(expression.get_type());
                let operand = expression.operand();
                self.emit_prefix(&operator.text, &operand, result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::BitwiseAndExpression(expression) => {
                let result_type = self.resolve_slang_type(expression.get_type());
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_binary_op(&left, &right, "&", result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::BitwiseOrExpression(expression) => {
                let result_type = self.resolve_slang_type(expression.get_type());
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_binary_op(&left, &right, "|", result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::BitwiseXorExpression(expression) => {
                let result_type = self.resolve_slang_type(expression.get_type());
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_binary_op(&left, &right, "^", result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::ShiftExpression(expression) => {
                let result_type = self.resolve_slang_type(expression.get_type());
                let left = expression.left_operand();
                let right = expression.right_operand();
                let operator = expression.operator();
                self.emit_binary_op(&left, &right, &operator.text, result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::FunctionCallExpression(call) => {
                self::call::CallEmitter::new(self).emit_function_call(call, block)
            }
            Expression::MemberAccessExpression(access) => {
                if let Some((address, element_type, block)) =
                    self.emit_struct_field_address(access, block.clone())?
                {
                    let value = self.load_or_address(address, element_type, &block)?;
                    Ok((Some(value), block))
                } else {
                    self::call::CallEmitter::new(self)
                        .emit_member_access(access, block)
                        .map(|(value, block)| (Some(value), block))
                }
            }
            Expression::TupleExpression(tuple) => {
                let items = tuple.items();
                // TODO: support multi-value tuples (e.g. tuple deconstruction)
                anyhow::ensure!(items.len() == 1, "multi-value tuples not yet supported");
                let item = items.iter().next().expect("length checked to be 1 above");
                let inner = item
                    .expression()
                    .ok_or_else(|| anyhow::anyhow!("empty tuple element"))?;
                self.emit(&inner, block)
            }
            Expression::ConditionalExpression(conditional) => {
                let result_type = self
                    .resolve_slang_type(conditional.get_type())
                    .unwrap_or(self.state.builder.types.ui256);
                let condition = conditional.operand();
                let (condition_value, block) = self.emit_value(&condition, block)?;
                let condition_boolean = self.emit_is_nonzero(condition_value, &block);

                let (then_block, else_block, result) =
                    self.state
                        .builder
                        .emit_scf_if(condition_boolean, result_type, &block)?;

                let true_expression = conditional.true_expression();
                let (then_value, then_end) = self.emit_value(&true_expression, then_block)?;
                let then_cast = TypeConversion::from_target_type(result_type, &self.state.builder)
                    .emit(then_value, &self.state.builder, &then_end);
                self.state.builder.emit_scf_yield(&[then_cast], &then_end);

                let false_expression = conditional.false_expression();
                let (else_value, else_end) = self.emit_value(&false_expression, else_block)?;
                let else_cast = TypeConversion::from_target_type(result_type, &self.state.builder)
                    .emit(else_value, &self.state.builder, &else_end);
                self.state.builder.emit_scf_yield(&[else_cast], &else_end);

                Ok((Some(result), block))
            }
            Expression::IndexAccessExpression(index_access) => {
                let (address, element_type, block) =
                    self.emit_index_access_address(index_access, block)?;
                let value = self.load_or_address(address, element_type, &block)?;
                Ok((Some(value), block))
            }
            _ => anyhow::bail!(
                "unsupported expression: {:?}",
                std::mem::discriminant(expression)
            ),
        }
    }

    /// Emits a `sol.cmp ne 0` producing `i1` from a value.
    ///
    /// Short-circuits when the value is already `i1` (e.g. from `sol.cmp`),
    /// avoiding the redundant `sol.cmp ne, %i1, %zero_i1 : i1` pattern.
    pub fn emit_is_nonzero(
        &self,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        if solx_mlir::TypeFactory::integer_bit_width(value.r#type()) == 1 {
            return value;
        }
        let zero = self
            .state
            .builder
            .emit_sol_constant(0, value.r#type(), block);
        self.state
            .builder
            .emit_sol_cmp(value, zero, CmpPredicate::Ne, block)
    }

    /// Loads `address` as `element_type` when `element_type` is reached via a
    /// pointer, or returns `address` directly when it already IS the value.
    ///
    /// The latter holds for reference-type elements (struct, array, mapping,
    /// bytes, string) in Storage or CallData — the non-ptr-ref-in-storage rule
    /// produces an address whose type equals the element type, and `sol.load`
    /// on it would be invalid.
    fn load_or_address(
        &self,
        address: Value<'context, 'block>,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        if address.r#type() == element_type {
            Ok(address)
        } else {
            self.state
                .builder
                .emit_sol_load(address, element_type, block)
        }
    }

    /// Emits the address for an `a[i]` / `m[k]` expression and returns it
    /// together with the element MLIR type.
    ///
    /// Shared between the value-producing read path and the lvalue write path
    /// in `emit_assignment`.
    pub fn emit_index_access_address(
        &self,
        index_access: &slang_solidity::backend::ir::ast::IndexAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        Value<'context, 'block>,
        Type<'context>,
        BlockRef<'context, 'block>,
    )> {
        if index_access.end().is_some() {
            unimplemented!("range index (a[i:j]) is not yet supported");
        }
        let index_expression = index_access
            .start()
            .expect("slang validates a[i] has an index expression");
        let base = index_access.operand();
        let base_slang_type = match &base {
            Expression::Identifier(identifier) => {
                let name = identifier.name();
                match identifier.resolve_to_definition() {
                    Some(Definition::StateVariable(state_variable)) => state_variable.get_type(),
                    Some(Definition::Variable(variable)) => variable.get_type(),
                    Some(Definition::Parameter(parameter)) => parameter.get_type(),
                    None => anyhow::bail!("unresolved identifier: {name}"),
                    Some(_) => anyhow::bail!("unsupported identifier reference: {name}"),
                }
            }
            Expression::MemberAccessExpression(member) => member.get_type(),
            Expression::IndexAccessExpression(inner) => inner.get_type(),
            Expression::FunctionCallExpression(call) => call.get_type(),
            Expression::TupleExpression(tuple) => tuple.get_type(),
            Expression::ConditionalExpression(conditional) => conditional.get_type(),
            _ => unimplemented!("index access base of unsupported expression kind"),
        }
        .ok_or_else(|| anyhow::anyhow!("base of index access has no resolved type"))?;
        let result_slang_type = index_access
            .get_type()
            .expect("slang types every index-access expression");
        let (base_value, block) = self.emit_value(&base, block)?;
        let (index_value, block) = self.emit_value(&index_expression, block)?;
        let builder = &self.state.builder;
        let element_type = TypeConversion::resolve_slang_type(&result_slang_type, None, builder);
        let base_slang_location = match &base_slang_type {
            SlangType::Mapping(_) => SlangDataLocation::Storage,
            SlangType::Array(t) => t.location(),
            SlangType::FixedSizeArray(t) => t.location(),
            SlangType::Bytes(t) => t.location(),
            SlangType::String(t) => t.location(),
            other => anyhow::bail!(
                "unsupported index access base type: {:?}",
                std::mem::discriminant(other)
            ),
        };
        let base_location = match base_slang_location {
            SlangDataLocation::Inherited => unimplemented!(
                "index access through Inherited (struct-field) location is not yet supported"
            ),
            other => solx_utils::DataLocation::from_slang(other, None),
        };
        // Mirror `Sol_GepOp::build`'s non-ptr-ref-in-storage rule
        // (solx-llvm SolOps.cpp:299-302): when the element is itself a
        // reference type and lives in Storage or CallData, the result
        // address IS the element type rather than a pointer to it.
        let element_is_non_ptr_ref = matches!(
            result_slang_type,
            SlangType::Array(_)
                | SlangType::FixedSizeArray(_)
                | SlangType::Bytes(_)
                | SlangType::String(_)
                | SlangType::Mapping(_)
                | SlangType::Struct(_)
        );
        let address_type = if element_is_non_ptr_ref
            && matches!(
                base_location,
                solx_utils::DataLocation::Storage | solx_utils::DataLocation::CallData
            ) {
            element_type
        } else {
            builder.types.pointer(element_type, base_location)
        };
        let address = match &base_slang_type {
            SlangType::Mapping(_) => {
                builder.emit_sol_map(base_value, index_value, address_type, &block)
            }
            _ => builder.emit_sol_gep(base_value, index_value, address_type, &block),
        };
        Ok((address, element_type, block))
    }

    /// Emits the address for a struct field access (`s.field`), returning it
    /// together with the field's element MLIR type and the post-evaluation
    /// block. Returns `Ok(None)` when the access is not a struct field (e.g.
    /// `msg.sender` or other built-in / non-struct base).
    ///
    /// Shared between the value-producing read path and the lvalue write path
    /// in `emit_assignment`.
    pub fn emit_struct_field_address(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<
        Option<(
            Value<'context, 'block>,
            Type<'context>,
            BlockRef<'context, 'block>,
        )>,
    > {
        let base = access.operand();
        let base_slang_type = match &base {
            Expression::Identifier(identifier) => match identifier.resolve_to_definition() {
                Some(Definition::StateVariable(state_variable)) => state_variable.get_type(),
                Some(Definition::Variable(variable)) => variable.get_type(),
                Some(Definition::Parameter(parameter)) => parameter.get_type(),
                _ => None,
            },
            Expression::MemberAccessExpression(member) => member.get_type(),
            Expression::IndexAccessExpression(inner) => inner.get_type(),
            Expression::FunctionCallExpression(call) => call.get_type(),
            Expression::TupleExpression(tuple) => tuple.get_type(),
            Expression::ConditionalExpression(conditional) => conditional.get_type(),
            _ => None,
        };
        let Some(SlangType::Struct(struct_type)) = base_slang_type else {
            return Ok(None);
        };
        let struct_location = solx_utils::DataLocation::from_slang(struct_type.location(), None);
        let struct_definition = match struct_type.definition() {
            Definition::Struct(definition) => definition,
            _ => unreachable!("Slang StructType always references a Struct definition"),
        };
        let member_name = access.member().name();
        let mut field_info = None;
        for (idx, member) in struct_definition.members().iter().enumerate() {
            if member.name().name() == member_name {
                field_info = Some((idx, member.get_type()));
                break;
            }
        }
        let (field_index, field_slang_type) =
            field_info.ok_or_else(|| anyhow::anyhow!("unknown struct member: {member_name}"))?;
        let field_slang_type = field_slang_type
            .ok_or_else(|| anyhow::anyhow!("struct member has no resolved type: {member_name}"))?;
        let (base_value, block) = self.emit_value(&base, block)?;
        let builder = &self.state.builder;
        let element_type =
            TypeConversion::resolve_slang_type(&field_slang_type, Some(struct_location), builder);
        // Mirror `Sol_GepOp::build`'s non-ptr-ref-in-storage rule
        // (solx-llvm SolOps.cpp:299-302): when the element is itself a
        // reference type and lives in Storage or CallData, the result
        // address IS the element type rather than a pointer to it.
        let element_is_non_ptr_ref = matches!(
            field_slang_type,
            SlangType::Array(_)
                | SlangType::FixedSizeArray(_)
                | SlangType::Bytes(_)
                | SlangType::String(_)
                | SlangType::Mapping(_)
                | SlangType::Struct(_)
        );
        let address_type = if element_is_non_ptr_ref
            && matches!(
                struct_location,
                solx_utils::DataLocation::Storage | solx_utils::DataLocation::CallData
            ) {
            element_type
        } else {
            builder.types.pointer(element_type, struct_location)
        };
        let ui64_type = Type::from(IntegerType::unsigned(builder.context, 64));
        let index_value = builder.emit_sol_constant(field_index as i64, ui64_type, &block);
        let address = builder.emit_sol_gep(base_value, index_value, address_type, &block);
        Ok(Some((address, element_type, block)))
    }

    /// Resolves the Solidity type from Slang to an MLIR type.
    ///
    /// Returns `None` when the incoming slang type is `None`. This can happen when calling
    /// `node.get_type()` if the node doesn't have typing information, for example when
    /// there are unresolved references or semantic errors.
    /// Panics on types that `TypeConversion::resolve_slang_type` does not yet handle.
    // TODO: slang's binder does not fold binary expressions of literal operands —
    // its typing rules return the type of one operand (e.g. type of the left
    // operand for shifts), so `1 << 100` gets typed as ui8 (the type of `1`)
    // and constant subexpressions overflow at that width. solc folds via
    // `RationalNumberType::binaryOperatorResult`, sizing the result to fit the
    // folded value. Either teach slang to fold, or fold here before lowering.
    pub fn resolve_slang_type(&self, slang_type: Option<SlangType>) -> Option<Type<'context>> {
        Some(TypeConversion::resolve_slang_type(
            &slang_type?,
            None,
            &self.state.builder,
        ))
    }
}
