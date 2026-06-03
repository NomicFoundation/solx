//!
//! User-defined operator bindings.
//!
//! Solidity lets a `using {f as op} for T global;` directive bind a function to
//! an operator on a user-defined value type. An operation such as `a + b` on a
//! `T`-typed operand must then call `f(a, b)` rather than emit native
//! arithmetic — `f` carries its own checked/unchecked context, so an
//! `unchecked` body wraps on overflow while the surrounding caller stays
//! checked (and vice versa).
//!
//! Operator bindings are always file-level and `global`, so they are gathered
//! once per compilation unit and shared across every contract. The gathered map
//! is keyed by `(udvt_definition_id, operator)`; the operand's UDVT definition
//! id is read from the operator function's first parameter, which Solidity
//! requires to be the bound type.
//!

use std::collections::HashMap;
use std::collections::HashSet;

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Type as SlangType;
use slang_solidity_v2::ast::UsingClause;
use slang_solidity_v2::ast::UsingOperator;
use slang_solidity_v2::compilation::CompilationUnit;

use solx_mlir::UserDefinedOperator;

use crate::ast::contract::function::expression::operator::Operator;

/// The user-defined operator bindings of a compilation unit.
pub struct OperatorBindings {
    /// Maps `(udvt_definition_id, operator)` to the bound function's node id.
    pub map: HashMap<(NodeId, UserDefinedOperator), NodeId>,
    /// The bound operator functions, to be registered and emitted so the
    /// dispatched calls resolve. A function bound to several operators (e.g.
    /// `using {foo as +, foo as -}`) appears once.
    pub functions: Vec<FunctionDefinition>,
}

impl OperatorBindings {
    /// Gathers every file-level operator binding in the unit.
    pub fn gather(unit: &CompilationUnit) -> Self {
        let mut map = HashMap::new();
        let mut functions = Vec::new();
        let mut seen_functions = HashSet::new();

        let directives: Vec<_> = unit
            .file_ids()
            .iter()
            .filter_map(|file_identifier| unit.file(file_identifier))
            .flat_map(|file| {
                file.ast()
                    .members()
                    .iter()
                    .filter_map(|member| {
                        if let slang_solidity_v2::ast::SourceUnitMember::UsingDirective(directive) =
                            member
                        {
                            Some(directive)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        for directive in directives {
            // Operators are bound only via a deconstruction clause `{f as op}`;
            // `using L for T` attaches library functions, never operators.
            let UsingClause::UsingDeconstruction(deconstruction) = directive.clause() else {
                continue;
            };
            for symbol in deconstruction.symbols().iter() {
                let Some(operator_token) = symbol.alias() else {
                    continue;
                };
                let Some(Definition::Function(function)) = symbol.name().resolve_to_definition()
                else {
                    continue;
                };
                let parameters = function.parameters();
                let arity = parameters.iter().count();
                let Some(first_parameter) = parameters.iter().next() else {
                    continue;
                };
                let Some(SlangType::UserDefinedValue(udvt_type)) = first_parameter.get_type() else {
                    continue;
                };
                let Definition::UserDefinedValueType(udvt_definition) = udvt_type.definition()
                else {
                    continue;
                };
                let operator = map_using_operator(&operator_token, arity);
                map.insert((udvt_definition.node_id(), operator), function.node_id());
                if seen_functions.insert(function.node_id()) {
                    functions.push(function);
                }
            }
        }

        Self { map, functions }
    }
}

/// Maps a `using` operator token to its [`UserDefinedOperator`]. `arity`
/// disambiguates the `-` token: a one-parameter function binds unary negation,
/// a two-parameter function binds binary subtraction.
fn map_using_operator(operator: &UsingOperator, arity: usize) -> UserDefinedOperator {
    match operator {
        UsingOperator::Plus(_) => UserDefinedOperator::Add,
        UsingOperator::Minus(_) if arity == 1 => UserDefinedOperator::Neg,
        UsingOperator::Minus(_) => UserDefinedOperator::Sub,
        UsingOperator::Asterisk(_) => UserDefinedOperator::Mul,
        UsingOperator::Slash(_) => UserDefinedOperator::Div,
        UsingOperator::Percent(_) => UserDefinedOperator::Rem,
        UsingOperator::Ampersand(_) => UserDefinedOperator::BitAnd,
        UsingOperator::Bar(_) => UserDefinedOperator::BitOr,
        UsingOperator::Caret(_) => UserDefinedOperator::BitXor,
        UsingOperator::EqualEqual(_) => UserDefinedOperator::Eq,
        UsingOperator::BangEqual(_) => UserDefinedOperator::Ne,
        UsingOperator::LessThan(_) => UserDefinedOperator::Lt,
        UsingOperator::LessThanEqual(_) => UserDefinedOperator::Le,
        UsingOperator::GreaterThan(_) => UserDefinedOperator::Gt,
        UsingOperator::GreaterThanEqual(_) => UserDefinedOperator::Ge,
        UsingOperator::Tilde(_) => UserDefinedOperator::BitNot,
    }
}

/// Maps a binary [`Operator`] to the [`UserDefinedOperator`] it could be bound
/// to, or `None` for operators that cannot be user-defined (`**`, shifts).
pub fn binary_operator(operator: Operator) -> Option<UserDefinedOperator> {
    Some(match operator {
        Operator::Add => UserDefinedOperator::Add,
        Operator::Subtract => UserDefinedOperator::Sub,
        Operator::Multiply => UserDefinedOperator::Mul,
        Operator::Divide => UserDefinedOperator::Div,
        Operator::Remainder => UserDefinedOperator::Rem,
        Operator::BitwiseAnd => UserDefinedOperator::BitAnd,
        Operator::BitwiseOr => UserDefinedOperator::BitOr,
        Operator::BitwiseXor => UserDefinedOperator::BitXor,
        _ => return None,
    })
}

/// Maps a prefix [`Operator`] to the unary [`UserDefinedOperator`] it could be
/// bound to, or `None` for prefix operators that cannot be user-defined
/// (`!`, `++`, `--`, `delete`).
pub fn unary_operator(operator: Operator) -> Option<UserDefinedOperator> {
    Some(match operator {
        Operator::Subtract => UserDefinedOperator::Neg,
        Operator::BitwiseNot => UserDefinedOperator::BitNot,
        _ => return None,
    })
}
