//!
//! Solidity to MLIR type mapping.
//!

use slang_solidity::backend::ir::ir2_flat_contracts::ElementaryType;
use slang_solidity::backend::ir::ir2_flat_contracts::TypeName;

/// Maps Solidity types to MLIR LLVM dialect types.
pub struct TypeMapper;

/// The ABI-encoded size of a single word slot in bytes.
#[allow(dead_code)]
const ABI_ENCODED_WORD_SIZE: usize = 32;

impl TypeMapper {
    /// Returns the MLIR type string for a Solidity type.
    ///
    /// In the first pass all value types map to `i256` (the EVM word size).
    #[allow(dead_code)]
    pub(crate) fn mlir_type(_type_name: &TypeName) -> &'static str {
        "i256"
    }

    /// Returns the ABI-encoded size in bytes for a Solidity type.
    ///
    /// In the first pass all types are treated as a single 32-byte slot.
    #[allow(dead_code)]
    pub(crate) fn abi_encoded_size(_type_name: &TypeName) -> usize {
        ABI_ENCODED_WORD_SIZE
    }

    /// Returns whether a type is a signed integer (`int8`..`int256`).
    pub(crate) fn is_signed(type_name: &TypeName) -> bool {
        matches!(type_name, TypeName::ElementaryType(ElementaryType::IntKeyword(_)))
    }

    /// Returns the canonical ABI type string for a Solidity type name.
    ///
    /// Used when computing function selectors.
    pub(crate) fn canonical_type(type_name: &TypeName) -> String {
        match type_name {
            TypeName::ElementaryType(elementary) => Self::canonical_elementary(elementary),
            TypeName::IdentifierPath(path) => path
                .iter()
                .map(|segment| segment.text.as_str())
                .collect::<Vec<_>>()
                .join("."),
            TypeName::ArrayTypeName(array) => {
                let base = Self::canonical_type(&array.operand);
                if let Some(ref size_expr) = array.index {
                    format!("{base}[{size_expr:?}]")
                } else {
                    format!("{base}[]")
                }
            }
            TypeName::MappingType(_) => "mapping".to_owned(),
            TypeName::FunctionType(_) => "function".to_owned(),
        }
    }

    /// Returns the canonical ABI string for an elementary type.
    ///
    /// Normalizes `uint` -> `uint256` and `int` -> `int256` per ABI spec.
    fn canonical_elementary(elementary: &ElementaryType) -> String {
        match elementary {
            ElementaryType::AddressType(_) => "address".to_owned(),
            ElementaryType::BoolKeyword => "bool".to_owned(),
            ElementaryType::ByteKeyword => "bytes1".to_owned(),
            ElementaryType::StringKeyword => "string".to_owned(),
            ElementaryType::UintKeyword(terminal) => {
                let text = &terminal.text;
                if text == "uint" {
                    "uint256".to_owned()
                } else {
                    text.clone()
                }
            }
            ElementaryType::IntKeyword(terminal) => {
                let text = &terminal.text;
                if text == "int" {
                    "int256".to_owned()
                } else {
                    text.clone()
                }
            }
            ElementaryType::BytesKeyword(terminal)
            | ElementaryType::FixedKeyword(terminal)
            | ElementaryType::UfixedKeyword(terminal) => terminal.text.clone(),
        }
    }
}
