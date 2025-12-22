//!
//! LLVM-specific part of the parser.
//!

pub mod attributes;

use std::collections::BTreeSet;

use solx_yul::yul::lexer::Lexer;
use solx_yul::yul::parser::dialect::Dialect;
use solx_yul::yul::parser::identifier::Identifier;

use self::attributes::get_llvm_attributes;

///
/// solx-specific part of the parser.
///
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EraDialect {}

impl Dialect for EraDialect {
    type FunctionAttribute = solx_codegen_evm::Attribute;

    fn extract_attributes(
        identifier: &Identifier,
        _: &mut Lexer,
    ) -> Result<BTreeSet<Self::FunctionAttribute>, solx_yul::yul::error::Error> {
        get_llvm_attributes(identifier)
    }
}
