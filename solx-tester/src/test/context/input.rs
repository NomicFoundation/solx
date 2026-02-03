//!
//! Context used to process test inputs, organized in test cases.
//!

use crate::test::context::case::CaseContext;

///
/// Context used to process test inputs, organized in test cases.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputContext<'a> {
    /// Context of the parent case, which contains this input.
    pub case_context: &'a CaseContext<'a>,
    /// Optional name of the case.
    pub case_name: &'a Option<String>,
    /// Index of the input in the case's array of inputs.
    pub selector: usize,
}
