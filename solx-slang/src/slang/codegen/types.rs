//!
//! Solidity to MLIR type mapping.
//!

use slang_solidity::backend::ir::ast::ElementaryType;
use slang_solidity::backend::ir::ast::Expression;
use slang_solidity::backend::ir::ast::TypeName;

/// Maps Solidity types to MLIR LLVM dialect types.
pub(crate) struct TypeMapper;

impl TypeMapper {
    /// The ABI-encoded size of a single word slot in bytes.
    const ABI_ENCODED_WORD_SIZE: usize = 32;

    /// Returns the MLIR type string for a Solidity type.
    ///
    /// In the first pass all value types map to `i256` (the EVM word size).
    #[expect(
        dead_code,
        reason = "will be used when per-type MLIR lowering is implemented"
    )]
    pub(crate) fn mlir_type(_type_name: &TypeName) -> &'static str {
        "i256"
    }

    /// Returns the ABI-encoded size in bytes for a Solidity type.
    ///
    /// In the first pass all types are treated as a single 32-byte slot.
    #[expect(dead_code, reason = "will be used when ABI encoding is implemented")]
    pub(crate) fn abi_encoded_size(_type_name: &TypeName) -> usize {
        Self::ABI_ENCODED_WORD_SIZE
    }

    /// Returns whether a type is a signed integer (`int8`..`int256`).
    pub(crate) fn is_signed(type_name: &TypeName) -> bool {
        matches!(
            type_name,
            TypeName::ElementaryType(ElementaryType::IntKeyword(_))
        )
    }

    /// Returns the canonical ABI type string for a Solidity type name.
    ///
    /// Used when computing function selectors.
    ///
    /// # Errors
    ///
    /// Returns an error for non-literal array size expressions.
    pub(crate) fn canonical_type(type_name: &TypeName) -> anyhow::Result<String> {
        match type_name {
            TypeName::ElementaryType(elementary) => Ok(Self::canonical_elementary(elementary)),
            // TODO: resolve IdentifierPath to struct fields for ABI tuple encoding.
            TypeName::IdentifierPath(path) => Ok(path.name()),
            TypeName::ArrayTypeName(array) => {
                let base = Self::canonical_type(&array.operand())?;
                match array.index() {
                    Some(Expression::DecimalNumberExpression(decimal)) => {
                        let size = &decimal.literal().text;
                        Ok(format!("{base}[{size}]"))
                    }
                    Some(Expression::HexNumberExpression(hex)) => {
                        let text = &hex.literal().text;
                        let stripped = text
                            .strip_prefix("0x")
                            .or(text.strip_prefix("0X"))
                            .unwrap_or(text);
                        let decimal = u64::from_str_radix(stripped, 16)
                            .map_err(|_| anyhow::anyhow!("invalid hex array size: {text}"))?;
                        Ok(format!("{base}[{decimal}]"))
                    }
                    Some(_) => anyhow::bail!("unsupported array size expression"),
                    None => Ok(format!("{base}[]")),
                }
            }
            // TODO: MappingType and FunctionType are not valid ABI parameter types.
            TypeName::MappingType(_) => Ok("mapping".to_owned()),
            TypeName::FunctionType(_) => Ok("function".to_owned()),
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
