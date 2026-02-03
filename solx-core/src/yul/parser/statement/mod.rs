//!
//! The block statement.
//!

use crate::declare_wrapper;
use crate::yul::parser::dialect::era::EraDialect;

pub mod assignment;
pub mod block;
pub mod code;
pub mod expression;
pub mod for_loop;
pub mod function_definition;
pub mod if_conditional;
pub mod object;
pub mod switch;
pub mod variable_declaration;

declare_wrapper!(
    solx_yul::yul::parser::statement::Statement<EraDialect>,
    Statement
);
