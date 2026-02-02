//!
//! The gas option builder.
//!

use crate::test::function_call::parser::lexical::Keyword;
use crate::test::function_call::parser::lexical::Location;
use crate::test::function_call::parser::syntax::tree::gas::Gas;
use crate::test::function_call::parser::syntax::tree::gas::variant::Variant as GasVariant;

///
/// The gas option builder.
///
#[derive(Default)]
pub struct Builder {
    /// The location of the syntax construction.
    location: Option<Location>,
    /// The gas option variant keyword.
    keyword: Option<Keyword>,
    /// The gas value.
    value: Option<String>,
}

/// The invalid type keyword panic, which is prevented by the gas option parser.
static BUILDER_GAS_INVALID_KEYWORD: &str =
    "The type builder has got an unexpected non-gas keyword: ";

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
    pub fn set_value(&mut self, value: String) {
        self.value = Some(value);
    }

    ///
    /// Finalizes the builder and returns the built value.
    ///
    /// # Panics
    /// If some of the required items has not been set.
    ///
    pub fn finish(mut self) -> Gas {
        let location = self
            .location
            .take()
            .unwrap_or_else(|| panic!("{}{}", "Mandatory value missing: ", "location"));

        let variant = match self.keyword.take() {
            Some(Keyword::IrOptimized) => GasVariant::ir_optimized(),
            Some(Keyword::Legacy) => GasVariant::legacy(),
            Some(Keyword::LegacyOptimized) => GasVariant::legacy_optimized(),
            Some(Keyword::Ir) => GasVariant::ir(),
            Some(keyword) => panic!("{}{}", self::BUILDER_GAS_INVALID_KEYWORD, keyword),
            None => panic!("{}{}", "Mandatory value missing: ", "keyword"),
        };

        let value = self
            .value
            .take()
            .unwrap_or_else(|| panic!("{}{}", "Mandatory value missing: ", "value"));

        Gas::new(location, variant, value)
    }
}
