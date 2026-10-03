//!
//! A function, modifier or public state variable getter in the debug symbol table.
//!

///
/// A function, modifier or public state variable getter in the debug symbol table.
///
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Function {
    /// The name, the keyword for a constructor, fallback or receive function.
    pub name: String,
    /// The function kind.
    pub kind: FunctionKind,
    /// The visibility.
    pub visibility: Visibility,
    /// The state mutability.
    pub mutability: Mutability,
    /// The 4-byte selector in lowercase hex, for an externally visible function or a getter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// The definition range, the state variable's for a getter.
    pub range: [usize; 2],
    /// Whether the function has a body.
    pub implemented: bool,
}

///
/// The kind of a function in the debug symbol table.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

///
/// The visibility of a function.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// `public`.
    Public,
    /// `external`.
    External,
    /// `internal`.
    Internal,
    /// `private`.
    Private,
}

///
/// The state mutability of a function.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mutability {
    /// `pure`.
    Pure,
    /// `view`.
    View,
    /// Neither `pure`, `view` nor `payable`.
    NonPayable,
    /// `payable`.
    Payable,
}
