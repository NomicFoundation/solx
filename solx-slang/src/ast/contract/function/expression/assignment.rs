//!
//! Assignment expression lowering (`=` and the compound `+=`, `&=`, … forms).
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Location;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::operation::Operation;
use slang_solidity_v2::ast;
use slang_solidity_v2::ast::AssignmentExpression;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::arithmetic::ArithmeticOperation;
use crate::ast::contract::function::expression::bitwise::BitwiseOperation;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::storage_slot::StorageSlot;

/// The operator of a compound assignment, dispatched to its domain.
enum CompoundOperation {
    /// An arithmetic compound assignment (`+=`, `-=`, `*=`, `/=`, `%=`).
    Arithmetic(ArithmeticOperation),
    /// A bitwise compound assignment (`&=`, `|=`, `^=`, `<<=`, `>>=`).
    Bitwise(BitwiseOperation),
}

impl CompoundOperation {
    /// Maps a slang assignment operator to its compound operation, or `None`
    /// for the plain `=` operator.
    fn from_operator(operator: &ast::AssignmentExpressionOperator) -> Option<Self> {
        match operator {
            ast::AssignmentExpressionOperator::Equal(_) => None,
            ast::AssignmentExpressionOperator::PlusEqual(_) => {
                Some(Self::Arithmetic(ArithmeticOperation::Add))
            }
            ast::AssignmentExpressionOperator::MinusEqual(_) => {
                Some(Self::Arithmetic(ArithmeticOperation::Subtract))
            }
            ast::AssignmentExpressionOperator::AsteriskEqual(_) => {
                Some(Self::Arithmetic(ArithmeticOperation::Multiply))
            }
            ast::AssignmentExpressionOperator::SlashEqual(_) => {
                Some(Self::Arithmetic(ArithmeticOperation::Divide))
            }
            ast::AssignmentExpressionOperator::PercentEqual(_) => {
                Some(Self::Arithmetic(ArithmeticOperation::Remainder))
            }
            ast::AssignmentExpressionOperator::AmpersandEqual(_) => {
                Some(Self::Bitwise(BitwiseOperation::And))
            }
            ast::AssignmentExpressionOperator::BarEqual(_) => {
                Some(Self::Bitwise(BitwiseOperation::Or))
            }
            ast::AssignmentExpressionOperator::CaretEqual(_) => {
                Some(Self::Bitwise(BitwiseOperation::Xor))
            }
            ast::AssignmentExpressionOperator::LessThanLessThanEqual(_) => {
                Some(Self::Bitwise(BitwiseOperation::ShiftLeft))
            }
            ast::AssignmentExpressionOperator::GreaterThanGreaterThanEqual(_)
            | ast::AssignmentExpressionOperator::GreaterThanGreaterThanGreaterThanEqual(_) => {
                Some(Self::Bitwise(BitwiseOperation::ShiftRight))
            }
        }
    }

    /// Builds the Sol dialect operation for this compound operator.
    fn emit_operation<'context>(
        self,
        checked: bool,
        context: &'context melior::Context,
        location: Location<'context>,
        lhs: Value<'context, '_>,
        rhs: Value<'context, '_>,
    ) -> Operation<'context> {
        match self {
            Self::Arithmetic(operation) => {
                operation.emit_operation(checked, context, location, lhs, rhs)
            }
            Self::Bitwise(operation) => operation.emit_operation(context, location, lhs, rhs),
        }
    }
}

/// Assignment target resolved from the Slang binder.
enum AssignmentTarget<'context, 'block> {
    /// Address-typed pointer with its declared element type.
    ///
    /// Covers local variables, function parameters, and the result of an
    /// `a[i]` / `m[k]` index-access expression on the left-hand side.
    Pointer(Value<'context, 'block>, Type<'context>),
    /// State variable — storage slot and declared element type.
    Storage(StorageSlot, Type<'context>),
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits an assignment expression (`=`, `+=`, `-=`, `*=`, …).
    pub fn emit_assignment(
        &self,
        assign: &AssignmentExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let left = assign.left_operand();
        let (target, block) = self.resolve_assignment_target(&left, block)?;

        let right = assign.right_operand();
        let (value, block) =
            match CompoundOperation::from_operator(&assign.operator()) {
                None => self.emit_value(&right, block)?,
                Some(operation) => {
                    let (old, target_type) = match &target {
                        AssignmentTarget::Pointer(pointer, element_type) => {
                            let old = self.state.builder.emit_sol_load(
                                *pointer,
                                *element_type,
                                &block,
                            )?;
                            (old, *element_type)
                        }
                        AssignmentTarget::Storage(slot, element_type) => {
                            let old = self.emit_storage_load(slot, *element_type, &block)?;
                            (old, *element_type)
                        }
                    };
                    let (rhs, block) = self.emit_value(&right, block)?;
                    let old = TypeConversion::from_target_type(target_type, &self.state.builder)
                        .emit(old, &self.state.builder, &block);
                    let rhs = TypeConversion::from_target_type(target_type, &self.state.builder)
                        .emit(rhs, &self.state.builder, &block);
                    let result = block
                        .append_operation(operation.emit_operation(
                            self.checked,
                            self.state.builder.context,
                            self.state.builder.unknown_location,
                            old,
                            rhs,
                        ))
                        .result(0)
                        .expect("a binary operation always produces one result")
                        .into();
                    (result, block)
                }
            };

        let result = match &target {
            AssignmentTarget::Pointer(pointer, element_type) => {
                let stored_value = TypeConversion::from_target_type(
                    *element_type,
                    &self.state.builder,
                )
                .emit(value, &self.state.builder, &block);
                self.state
                    .builder
                    .emit_sol_store(stored_value, *pointer, &block);
                stored_value
            }
            AssignmentTarget::Storage(slot, element_type) => {
                let stored_value = TypeConversion::from_target_type(
                    *element_type,
                    &self.state.builder,
                )
                .emit(value, &self.state.builder, &block);
                self.emit_storage_store(slot, stored_value, *element_type, &block);
                stored_value
            }
        };
        Ok((result, block))
    }

    /// Resolves the left-hand side of an assignment to its storage location.
    fn resolve_assignment_target(
        &self,
        left: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        AssignmentTarget<'context, 'block>,
        BlockRef<'context, 'block>,
    )> {
        match left {
            Expression::Identifier(identifier) => {
                let name = identifier.name();
                let target = match identifier.resolve_to_definition() {
                    Some(Definition::StateVariable(state_variable)) => {
                        let declared_type = state_variable
                            .get_type()
                            .expect("the binder types every state variable");
                        if declared_type.is_reference_type() {
                            unimplemented!(
                                "assignment to a reference-typed state variable is not yet supported"
                            );
                        }
                        let slot = self
                            .storage_layout
                            .get(&state_variable.node_id())
                            .expect("every state variable has a storage slot")
                            .clone();
                        let element_type = TypeConversion::resolve_slang_type(
                            &declared_type,
                            None,
                            &self.state.builder,
                        );
                        AssignmentTarget::Storage(slot, element_type)
                    }
                    Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                        let (pointer, element_type) = self.environment.variable_with_type(&name);
                        AssignmentTarget::Pointer(pointer, element_type)
                    }
                    None => unreachable!("slang resolves every identifier reference"),
                    Some(_) => {
                        unimplemented!("assignment to '{name}' is not yet supported")
                    }
                };
                Ok((target, block))
            }
            Expression::IndexAccessExpression(index_access) => {
                let (address, element_type, block) =
                    self.emit_index_access_address(index_access, block)?;
                if address.r#type() == element_type {
                    unimplemented!(
                        "assignment to a reference-typed `a[i]` element in storage/calldata is not yet supported"
                    );
                }
                Ok((AssignmentTarget::Pointer(address, element_type), block))
            }
            Expression::MemberAccessExpression(access) => {
                let (address, element_type, block) = self
                    .emit_struct_field_address(access, block)?
                    .expect("slang validates a member-access lvalue resolves to a struct field");
                if address.r#type() == element_type {
                    unimplemented!(
                        "assignment to a reference-typed struct field in storage/calldata is not yet supported"
                    );
                }
                Ok((AssignmentTarget::Pointer(address, element_type), block))
            }
            _ => unimplemented!(
                "assignment target {:?} is not yet supported",
                std::mem::discriminant(left)
            ),
        }
    }
}
