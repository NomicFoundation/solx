//!
//! A function, modifier or public state variable getter in the debug symbol table.
//!

pub mod kind;
pub mod visibility;

use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind as SlangFunctionKind;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::Type;

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

impl From<&FunctionDefinition> for Function {
    /// Reads callable metadata from Slang, including whether the declaration is at file level.
    fn from(function: &FunctionDefinition) -> Self {
        let attributes = function.attributes();
        let range = function.get_text_range();
        let kind = match function.kind() {
            SlangFunctionKind::Regular if function.enclosing_definition().is_none() => {
                FunctionKind::Free
            }
            kind => kind.into(),
        };
        Self {
            name: Self::source_name(function),
            kind,
            visibility: attributes.visibility().into(),
            state_mutability: attributes.mutability().into(),
            selector: function
                .compute_selector()
                .map(|selector| format!("{selector:08x}")),
            range: [range.start, range.len()],
            implemented: function.body().is_some(),
        }
    }
}

impl From<&StateVariableDefinition> for Function {
    /// The getter uses the public variable's definition range.
    fn from(state_variable: &StateVariableDefinition) -> Self {
        let Some(Type::Function(getter)) = state_variable.getter_type() else {
            unreachable!("slang types a public state variable's getter as a function");
        };
        let range = state_variable.get_text_range();
        Self {
            name: state_variable.name().name().to_owned(),
            kind: FunctionKind::Getter,
            visibility: getter.visibility().into(),
            state_mutability: getter.mutability().into(),
            selector: state_variable
                .compute_selector()
                .map(|selector| format!("{selector:08x}")),
            range: [range.start, range.len()],
            implemented: true,
        }
    }
}

impl Function {
    /// Constructors, fallback and receive functions use their keyword as the source name.
    pub fn source_name(function: &FunctionDefinition) -> String {
        match function.kind() {
            SlangFunctionKind::Constructor => "constructor".to_owned(),
            SlangFunctionKind::Fallback => "fallback".to_owned(),
            SlangFunctionKind::Receive => "receive".to_owned(),
            SlangFunctionKind::Regular | SlangFunctionKind::Modifier => function
                .name()
                .expect("slang names every regular function and modifier")
                .name()
                .to_owned(),
        }
    }
}
