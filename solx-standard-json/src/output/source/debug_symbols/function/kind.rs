//!
//! The kind of a function in the debug symbol table.
//!

use slang_solidity_v2::ast::FunctionKind as SlangFunctionKind;

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

impl From<SlangFunctionKind> for FunctionKind {
    fn from(kind: SlangFunctionKind) -> Self {
        match kind {
            SlangFunctionKind::Regular => Self::Function,
            SlangFunctionKind::Constructor => Self::Constructor,
            SlangFunctionKind::Fallback => Self::Fallback,
            SlangFunctionKind::Receive => Self::Receive,
            SlangFunctionKind::Modifier => Self::Modifier,
        }
    }
}
