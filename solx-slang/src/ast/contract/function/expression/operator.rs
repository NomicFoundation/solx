//!
//! Solidity operator parsed from source text.
//!

use melior::ir::BlockRef;
use melior::ir::Location;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::operation::Operation;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Function;
use solx_mlir::UserDefinedOperator;
use solx_mlir::ods::sol::AddOperation;
use solx_mlir::ods::sol::AndOperation;
use solx_mlir::ods::sol::CAddOperation;
use solx_mlir::ods::sol::CDivOperation;
use solx_mlir::ods::sol::CExpOperation;
use solx_mlir::ods::sol::CMulOperation;
use solx_mlir::ods::sol::CSubOperation;
use solx_mlir::ods::sol::DivOperation;
use solx_mlir::ods::sol::ExpOperation;
use solx_mlir::ods::sol::ModOperation;
use solx_mlir::ods::sol::MulOperation;
use solx_mlir::ods::sol::OrOperation;
use solx_mlir::ods::sol::ShlOperation;
use solx_mlir::ods::sol::ShrOperation;
use solx_mlir::ods::sol::SubOperation;
use solx_mlir::ods::sol::XorOperation;

use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

/// Solidity operator parsed from source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// `+`
    Add,
    /// `-`
    Subtract,
    /// `*`
    Multiply,
    /// `/`
    Divide,
    /// `%`
    Remainder,
    /// `**`
    Exponentiation,

    /// `+=`
    AddAssign,
    /// `-=`
    SubtractAssign,
    /// `*=`
    MultiplyAssign,
    /// `/=`
    DivideAssign,
    /// `%=`
    RemainderAssign,

    /// `&`
    BitwiseAnd,
    /// `|`
    BitwiseOr,
    /// `^`
    BitwiseXor,
    /// `<<`
    ShiftLeft,
    /// `>>`
    ShiftRight,
    /// `~`
    BitwiseNot,

    /// `&=`
    BitwiseAndAssign,
    /// `|=`
    BitwiseOrAssign,
    /// `^=`
    BitwiseXorAssign,
    /// `<<=`
    ShiftLeftAssign,
    /// `>>=`
    ShiftRightAssign,

    /// `==`
    Equal,
    /// `!=`
    NotEqual,
    /// `>`
    Greater,
    /// `>=`
    GreaterEqual,
    /// `<`
    Less,
    /// `<=`
    LessEqual,
    /// `!`
    Not,

    /// `++`
    Increment,
    /// `--`
    Decrement,
}

impl Operator {
    /// Builds a Sol dialect binary operation via ODS-generated builders.
    ///
    /// When `checked` is true, uses checked variants (`sol.cadd`, `sol.csub`,
    /// `sol.cmul`, `sol.cdiv`, `sol.cexp`) for arithmetic operators. Modulo, bitwise,
    /// and shift operators are always unchecked. Result type is inferred
    /// from `lhs` (`SameOperandsAndResultType`).
    ///
    /// # Panics
    ///
    /// Panics if called on a comparison or assignment operator.
    pub fn emit_sol_binary_operation<'context>(
        self,
        checked: bool,
        context: &'context melior::Context,
        location: Location<'context>,
        lhs: Value<'context, '_>,
        rhs: Value<'context, '_>,
    ) -> Operation<'context> {
        match self {
            Self::Add | Self::Increment if checked => CAddOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Add | Self::Increment => AddOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Subtract | Self::Decrement if checked => {
                CSubOperation::builder(context, location)
                    .lhs(lhs)
                    .rhs(rhs)
                    .build()
                    .into()
            }
            Self::Subtract | Self::Decrement => SubOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Multiply if checked => CMulOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Multiply => MulOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Divide if checked => CDivOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Divide => DivOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Remainder => ModOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Exponentiation if checked => CExpOperation::builder(context, location)
                .result(lhs.r#type())
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Exponentiation => ExpOperation::builder(context, location)
                .result(lhs.r#type())
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::BitwiseAnd => AndOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::BitwiseOr => OrOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::BitwiseXor => XorOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::ShiftLeft => ShlOperation::builder(context, location)
                .result(lhs.r#type())
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::ShiftRight => ShrOperation::builder(context, location)
                .result(lhs.r#type())
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            _ => unreachable!(
                "emit_sol_binary_operation called on non-arithmetic operator: {self:?}"
            ),
        }
    }
}

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// The function bound to `user_operator` for `operand`'s user-defined value type, or `None` when
    /// the operand is not a bound user-defined value type.
    pub fn user_defined_operator(
        &self,
        operand: &Expression,
        user_operator: UserDefinedOperator,
    ) -> Option<NodeId> {
        let SlangType::UserDefinedValue(udvt_type) = operand.get_type()? else {
            return None;
        };
        let Definition::UserDefinedValueType(udvt_definition) = udvt_type.definition() else {
            return None;
        };
        self.state
            .operator_bindings
            .get(&(udvt_definition.node_id(), user_operator))
            .copied()
    }

    /// Emits a `sol.call` to the bound user-defined-operator function `function_id`, coercing each
    /// argument to its declared parameter type, and returns the single result value.
    pub fn emit_operator_call(
        &self,
        function_id: NodeId,
        argument_values: Vec<Value<'context, 'block>>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let (mlir_name, parameter_types, return_types) = self
            .state
            .resolve_function(function_id)
            .expect("bound operator function resolves to a registered signature");
        let argument_values: Vec<_> = argument_values
            .into_iter()
            .zip(parameter_types)
            .map(|(value, &parameter_type)| {
                TypeConversion::from_target_type(parameter_type, self.state).emit(
                    value,
                    self.state,
                    block,
                )
            })
            .collect();
        let results = Function::call(mlir_name, &argument_values, return_types, self.state, block)
            .expect("bound operator function call resolves to a registered signature");
        results.into_iter().next().expect("slang validated")
    }
}
