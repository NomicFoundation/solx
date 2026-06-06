//!
//! User-defined operator bindings (`using {f as op} for T global;`).
//!
//! Scaffold: only the type + the `gather` entry the frontend pipeline calls are
//! present, returning an empty binding set. The real gathering pass lands in a
//! later additive commit alongside the operator-lowering cluster.
//!

use std::collections::HashMap;

use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::UsingOperator;
use slang_solidity_v2::compilation::CompilationUnit;

use solx_mlir::UserDefinedOperator;

use crate::ast::contract::function::expression::operator::Operator;

/// User-defined operator bindings gathered from a compilation unit.
pub struct OperatorBindings {
    /// Maps `(udvt_definition_id, operator)` to the bound function's node id.
    pub map: HashMap<(NodeId, UserDefinedOperator), NodeId>,
    /// The bound operator functions, to be registered and emitted so the
    /// dispatched calls resolve.
    pub functions: Vec<FunctionDefinition>,
}

impl OperatorBindings {
    /// Gathers `using {f as op} for T global;` bindings from the unit.
    ///
    /// Scaffold: returns an empty binding set. (HELPERS renames this to
    /// `from_unit` at the operator-cluster fill; kept as `gather` here so the
    /// commit-1 call site in `slang/mod.rs` stays intact.)
    pub fn gather(unit: &CompilationUnit) -> Self {
        let _ = unit;
        Self {
            map: HashMap::new(),
            functions: Vec::new(),
        }
    }

    /// Maps the typed [`UsingOperator`] token → [`UserDefinedOperator`]; `arity
    /// == 1` disambiguates `Minus` → `Neg` vs `Sub`. Exhaustive over the 15
    /// `UsingOperator` variants (16 arms — `Minus` arity-split).
    pub fn map_using_operator(operator: &UsingOperator, arity: usize) -> UserDefinedOperator {
        let _ = (operator, arity);
        unimplemented!("using-operator mapping")
    }

    /// The user-defined binary operator for an [`Operator`], when one exists
    /// (`Add`/`Subtract`/`Multiply`/`Divide`/`Remainder`/`BitwiseAnd`/`Or`/`Xor`
    /// → `Some`; else `None`). `Option` is a genuine domain answer.
    pub fn binary_operator(operator: Operator) -> Option<UserDefinedOperator> {
        let _ = operator;
        unimplemented!("binary user-defined operator")
    }

    /// The user-defined unary operator for an [`Operator`] (`Subtract` → `Neg`,
    /// `BitwiseNot` → `BitNot`; else `None`).
    pub fn unary_operator(operator: Operator) -> Option<UserDefinedOperator> {
        let _ = operator;
        unimplemented!("unary user-defined operator")
    }
}
