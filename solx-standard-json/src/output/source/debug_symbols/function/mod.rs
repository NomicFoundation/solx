//!
//! A function, modifier or public state variable getter in the debug symbol table.
//!

pub mod kind;
pub mod visibility;

use self::kind::FunctionKind;
use self::visibility::Visibility;

///
/// A function, modifier or public state variable getter in the debug symbol table.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Function {
    /// The name, the keyword for a constructor, fallback or receive function.
    pub name: String,
    /// Whether this is a function, constructor, fallback, receive, modifier, getter or free function.
    pub kind: FunctionKind,
    /// The visibility Slang assigns: the declared one, otherwise `internal` for a modifier or free
    /// function, `public` for a constructor and `external` for a getter.
    pub visibility: Visibility,
    /// The state mutability Slang assigns, `view` for a getter.
    pub state_mutability: solx_mlir::StateMutability,
    /// The 4-byte selector in lowercase hex, for a public or external function of kind `function`,
    /// and for a getter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// The definition range, the state variable's for a getter.
    pub range: [usize; 2],
    /// Whether the function has a body, `true` for a getter, whose body the compiler generates.
    pub implemented: bool,
}
