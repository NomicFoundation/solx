//!
//! The literal builder.
//!

use crate::test::function_call::parser::lexical::Literal as LexicalLiteral;
use crate::test::function_call::parser::lexical::Location;
use crate::test::function_call::parser::syntax::tree::literal::Literal;
use crate::test::function_call::parser::syntax::tree::literal::alignment::Alignment;
use crate::test::function_call::parser::syntax::tree::literal::boolean::Literal as BooleanLiteral;
use crate::test::function_call::parser::syntax::tree::literal::hex::Literal as HexLiteral;
use crate::test::function_call::parser::syntax::tree::literal::integer::Literal as IntegerLiteral;
use crate::test::function_call::parser::syntax::tree::literal::string::Literal as StringLiteral;

///
/// The literal builder.
///
#[derive(Default)]
pub struct Builder {
    /// The location of the syntax construction.
    location: Option<Location>,
    /// The identifier string contents.
    literal: Option<LexicalLiteral>,
    /// The alignment.
    alignment: Option<Alignment>,
}

impl Builder {
    ///
    /// Sets the corresponding builder value.
    ///
    pub fn set_location(&mut self, value: Location) {
        self.location = Some(value);
    }

    ///
    /// Sets the corresponding builder value.
    ///
    pub fn set_literal(&mut self, value: LexicalLiteral) {
        self.literal = Some(value);
    }

    ///
    /// Sets the corresponding builder value.
    ///
    pub fn set_alignment(&mut self, value: Alignment) {
        self.alignment = Some(value);
    }

    ///
    /// Finalizes the builder and returns the built value.
    ///
    /// # Errors
    /// If some of the required items has not been set.
    ///
    pub fn finish(mut self) -> anyhow::Result<Literal> {
        let location = self
            .location
            .take()
            .ok_or_else(|| anyhow::anyhow!("Missing mandatory field: location"))?;

        let alignment = self.alignment.take().unwrap_or(Alignment::Default);

        match self.literal {
            Some(LexicalLiteral::Integer(integer)) => Ok(Literal::Integer(IntegerLiteral::new(
                location, integer, alignment,
            ))),
            Some(LexicalLiteral::String(string)) => Ok(Literal::String(StringLiteral::new(
                location, string, alignment,
            ))),
            Some(LexicalLiteral::Boolean(boolean)) => Ok(Literal::Boolean(BooleanLiteral::new(
                location, boolean, alignment,
            ))),
            Some(LexicalLiteral::Hex(hex)) => {
                Ok(Literal::Hex(HexLiteral::new(location, hex, alignment)))
            }
            None => Err(anyhow::anyhow!("Missing mandatory field: literal")),
        }
    }
}
