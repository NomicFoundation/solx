//!
//! Implementation of a visitor pattern for Yul syntax tree.
//!

use crate::yul::parser::statement::Statement;
use crate::yul::parser::statement::assignment::Assignment;
use crate::yul::parser::statement::block::Block;
use crate::yul::parser::statement::code::Code;
use crate::yul::parser::statement::expression::Expression;
use crate::yul::parser::statement::expression::function_call::FunctionCall;
use crate::yul::parser::statement::expression::function_call::name::Name;
use crate::yul::parser::statement::expression::literal::Literal;
use crate::yul::parser::statement::for_loop::ForLoop;
use crate::yul::parser::statement::function_definition::FunctionDefinition;
use crate::yul::parser::statement::if_conditional::IfConditional;
use crate::yul::parser::statement::object::Object;
use crate::yul::parser::statement::switch::Switch;
use crate::yul::parser::statement::variable_declaration::VariableDeclaration;

use super::parser::dialect::Dialect;

///
/// Utility conventional name of a function corresponding to the `code` block of
/// an object.
///
pub const IMPLICIT_CODE_FUNCTION_NAME: &str = "BODY";

#[allow(unused_variables)]
///
/// Visitor for Yul syntax tree.
///
pub trait Visitor<P>
where
    P: Dialect,
{
    ///
    /// By convention, methods not implemented for a specific visitor should
    /// panic with this message.
    ///
    const MSG_METHOD_NOT_IMPLEMENTED: &'static str = "Method not implemented for this visitor.";

    ///
    /// Visit `switch` statement in Yul syntax tree.
    ///
    fn visit_switch(&mut self, switch: &Switch<P>) {
        unreachable!("{}", Self::MSG_METHOD_NOT_IMPLEMENTED)
    }

    ///
    /// Visit Yul object in Yul syntax tree.
    ///
    fn visit_object(&mut self, object: &Object<P>) {
        unreachable!("{}", Self::MSG_METHOD_NOT_IMPLEMENTED)
    }

    ///
    /// Visit `for` statement in Yul syntax tree.
    ///
    fn visit_for_loop(&mut self, for_loop: &ForLoop<P>) {
        unreachable!("{}", Self::MSG_METHOD_NOT_IMPLEMENTED)
    }

    ///
    /// Visit a variable declaration in Yul syntax tree: `var x` or `var x = <initializer>`.
    ///
    fn visit_variable_declaration(&mut self, variable_declaration: &VariableDeclaration) {
        unreachable!("{}", Self::MSG_METHOD_NOT_IMPLEMENTED)
    }

    ///
    /// Visit a function definition in Yul syntax tree.
    ///
    fn visit_function_definition(&mut self, function_definition: &FunctionDefinition<P>) {
        unreachable!("{}", Self::MSG_METHOD_NOT_IMPLEMENTED)
    }

    ///
    /// Visit an identifier in Yul syntax tree: a user defined one, or one of the predefined set like `lt`.
    ///
    fn visit_name(&mut self, name: &Name) {
        unreachable!("{}", Self::MSG_METHOD_NOT_IMPLEMENTED)
    }

    ///
    /// Visit a function call in Yul syntax tree.
    ///
    fn visit_function_call(&mut self, call: &FunctionCall) {
        unreachable!("{}", Self::MSG_METHOD_NOT_IMPLEMENTED)
    }

    ///
    /// Visit an `if` statement in Yul syntax tree.
    ///
    fn visit_if_conditional(&mut self, if_conditional: &IfConditional<P>) {
        unreachable!("{}", Self::MSG_METHOD_NOT_IMPLEMENTED)
    }

    ///
    /// Visit a literal (e.g. integer) in Yul syntax tree.
    ///
    fn visit_literal(&mut self, lit: &Literal) {
        unreachable!("{}", Self::MSG_METHOD_NOT_IMPLEMENTED)
    }

    ///
    /// Visit an arbitrary Yul expression in Yul syntax tree.
    ///
    fn visit_expression(&mut self, expr: &Expression) {
        unreachable!("{}", Self::MSG_METHOD_NOT_IMPLEMENTED)
    }

    ///
    /// Visit an assignment in Yul syntax tree.
    ///
    fn visit_assignment(&mut self, assignment: &Assignment) {
        unreachable!("{}", Self::MSG_METHOD_NOT_IMPLEMENTED)
    }

    ///
    /// Visit an arbitrary statement in Yul syntax tree.
    ///
    fn visit_statement(&mut self, stmt: &Statement<P>) {
        unreachable!("{}", Self::MSG_METHOD_NOT_IMPLEMENTED)
    }

    ///
    /// Visit a block of statements in Yul syntax tree.
    ///
    fn visit_block(&mut self, block: &Block<P>) {
        unreachable!("{}", Self::MSG_METHOD_NOT_IMPLEMENTED)
    }

    ///
    /// Visit a `code` block of an object in Yul syntax tree.
    ///
    fn visit_code(&mut self, code: &Code<P>) {
        unreachable!("{}", Self::MSG_METHOD_NOT_IMPLEMENTED)
    }
}
