//!
//! The kind of a function in the debug symbol table.
//!

///
/// The kind of a function in the debug symbol table.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FunctionKind {
    /// A regular contract, interface or library function.
    Function,
    /// A constructor.
    Constructor,
    /// A fallback function.
    Fallback,
    /// A receive function.
    Receive,
    /// A modifier.
    Modifier,
    /// The getter of a public state variable.
    Getter,
    /// A function declared at file level.
    Free,
}
