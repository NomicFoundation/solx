//!
//! The function call builder.
//!

use crate::test::function_call::parser::lexical::Location;
use crate::test::function_call::parser::syntax::tree::event::Event;
use crate::test::function_call::parser::syntax::tree::event::literal::EventLiteral;
use crate::test::function_call::parser::syntax::tree::event::variant::Variant;
use crate::test::function_call::parser::syntax::tree::identifier::Identifier;
use crate::test::function_call::parser::syntax::tree::r#type::Type;

///
/// The function call builder.
///
#[derive(Default)]
pub struct Builder {
    /// The location of the syntax construction.
    location: Option<Location>,
    /// The event name.
    identifier: Option<Identifier>,
    /// The types.
    types: Vec<Type>,
    /// The expected values.
    expected: Vec<EventLiteral>,
    /// The address.
    address: Option<String>,
    /// The flag if for empty expected should be empty vector.
    is_expected: bool,
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
    pub fn set_is_expected(&mut self) {
        self.is_expected = true;
    }

    ///
    /// Sets the corresponding builder value.
    ///
    pub fn set_identifier(&mut self, value: Identifier) {
        self.identifier = Some(value);
    }

    ///
    /// Sets the corresponding builder value.
    ///
    pub fn set_address(&mut self, address: String) {
        self.address = Some(address);
    }

    ///
    /// Pushes the corresponding builder value.
    ///
    pub fn push_expected(&mut self, value: EventLiteral) {
        self.expected.push(value)
    }

    ///
    /// Pushes the corresponding builder value.
    ///
    pub fn push_type(&mut self, value: Type) {
        self.types.push(value)
    }

    ///
    /// Finalizes the builder and returns the built value.
    ///
    /// # Panics
    /// If some of the required items has not been set.
    ///
    pub fn finish(mut self) -> Event {
        let location = self
            .location
            .take()
            .unwrap_or_else(|| panic!("{}{}", "Mandatory value missing: ", "location"));

        let expected = if self.expected.is_empty() {
            if !self.is_expected {
                None
            } else {
                Some(Vec::new())
            }
        } else {
            Some(self.expected)
        };

        let variant = if let Some(identifier) = self.identifier.take() {
            Variant::signature(identifier, self.types)
        } else {
            Variant::anonymous()
        };

        Event::new(location, variant, self.address, expected)
    }
}
