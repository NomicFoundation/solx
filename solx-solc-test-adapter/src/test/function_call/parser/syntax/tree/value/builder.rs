//!
//! The value option builder.
//!

use crate::test::function_call::parser::lexical::Keyword;
use crate::test::function_call::parser::lexical::Location;
use crate::test::function_call::parser::syntax::tree::value::Value;
use crate::test::function_call::parser::syntax::tree::value::unit::Unit;

///
/// The value option builder.
///
#[derive(Default)]
pub struct Builder {
    /// The location of the syntax construction.
    location: Option<Location>,
    /// The unit keyword.
    keyword: Option<Keyword>,
    /// The amount.
    amount: Option<String>,
}

/// The invalid type keyword panic, which is prevented by the gas option parser.
static BUILDER_VALUE_INVALID_KEYWORD: &str =
    "The type builder has got an unexpected non-unit keyword: ";

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
    pub fn set_keyword(&mut self, value: Keyword) {
        self.keyword = Some(value);
    }

    ///
    /// Sets the corresponding builder value.
    ///
    pub fn set_amount(&mut self, value: String) {
        self.amount = Some(value);
    }

    ///
    /// Finalizes the builder and returns the built value.
    ///
    /// # Errors
    /// If some of the required items has not been set.
    ///
    pub fn finish(mut self) -> anyhow::Result<Value> {
        let location = self
            .location
            .take()
            .ok_or_else(|| anyhow::anyhow!("Missing mandatory field: location"))?;

        let unit = match self.keyword.take() {
            Some(Keyword::Ether) => Unit::ether(),
            Some(Keyword::Wei) => Unit::wei(),
            Some(keyword) => anyhow::bail!("{}{}", self::BUILDER_VALUE_INVALID_KEYWORD, keyword),
            None => anyhow::bail!("Missing mandatory field: keyword"),
        };

        let amount = self
            .amount
            .take()
            .ok_or_else(|| anyhow::anyhow!("Missing mandatory field: amount"))?;

        Ok(Value::new(location, unit, amount))
    }
}
