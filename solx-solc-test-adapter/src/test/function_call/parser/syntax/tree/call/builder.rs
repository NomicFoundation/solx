//!
//! The function call builder.
//!

use crate::test::function_call::parser::lexical::Location;
use crate::test::function_call::parser::syntax::tree::call::Call;
use crate::test::function_call::parser::syntax::tree::call::variant::Variant;
use crate::test::function_call::parser::syntax::tree::event::Event;
use crate::test::function_call::parser::syntax::tree::identifier::Identifier;
use crate::test::function_call::parser::syntax::tree::literal::Literal;
use crate::test::function_call::parser::syntax::tree::r#type::Type;
use crate::test::function_call::parser::syntax::tree::value::Value;

///
/// The function call builder.
///
#[derive(Default)]
pub struct Builder {
    /// The location of the syntax construction.
    location: Option<Location>,
    /// The function name.
    call: Option<Identifier>,
    /// The library name.
    library: Option<Identifier>,
    /// The library source file name.
    library_source: Option<String>,
    /// The params types.
    types: Vec<Type>,
    /// The flag if for empty types should be empty vector.
    is_types: bool,
    /// The value option.
    value: Option<Value>,
    /// The input values.
    input: Vec<Literal>,
    /// The flag if for empty input should be empty vector.
    is_input: bool,
    /// The expected values.
    expected: Vec<Literal>,
    /// The flag if for empty expected should be empty vector.
    is_expected: bool,
    /// The failure expected flag.
    pub(crate) failure: bool,
    /// The expected events.
    events: Vec<Event>,
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
    pub fn set_call(&mut self, value: Identifier) {
        self.call = Some(value);
    }

    ///
    /// Sets the corresponding builder value.
    ///
    pub fn set_library(&mut self, value: Identifier) {
        self.library = Some(value);
    }

    ///
    /// Sets the corresponding builder value.
    ///
    pub fn set_library_source(&mut self, value: String) {
        self.library_source = Some(value);
    }

    ///
    /// Pushes the corresponding builder value.
    ///
    pub fn push_types(&mut self, value: Type) {
        self.types.push(value)
    }

    ///
    /// Sets the corresponding builder value.
    ///
    pub fn set_is_types(&mut self) {
        self.is_types = true;
    }

    ///
    /// Sets the corresponding builder value.
    ///
    pub fn set_value(&mut self, value: Value) {
        self.value = Some(value);
    }

    ///
    /// Pushes the corresponding builder value.
    ///
    pub fn push_input(&mut self, value: Literal) {
        self.input.push(value)
    }

    ///
    /// Sets the corresponding builder value.
    ///
    pub fn set_is_input(&mut self) {
        self.is_input = true;
    }

    ///
    /// Pushes the corresponding builder value.
    ///
    pub fn push_expected(&mut self, value: Literal) {
        self.expected.push(value)
    }

    ///
    /// Sets the corresponding builder value.
    ///
    pub fn set_is_expected(&mut self) {
        self.is_expected = true;
    }

    ///
    /// Pushes the corresponding builder value.
    ///
    pub fn push_event(&mut self, value: Event) {
        self.events.push(value)
    }

    /// Finalizes the builder and returns the built value.
    ///
    /// # Errors
    /// If some of the required items has not been set.
    ///
    pub fn finish(mut self) -> anyhow::Result<Call> {
        let location = self
            .location
            .take()
            .ok_or_else(|| anyhow::anyhow!("Missing mandatory field: location"))?;

        let variant = if let Some(identifier) = self.library.take() {
            Variant::library(identifier, self.library_source)
        } else {
            let identifier = self.call.take();
            let types = if self.types.is_empty() {
                if self.is_types {
                    Some(Vec::new())
                } else {
                    None
                }
            } else {
                Some(self.types)
            };

            let input = if self.input.is_empty() {
                if self.is_input {
                    Some(Vec::new())
                } else {
                    None
                }
            } else {
                Some(self.input)
            };

            let expected = if self.expected.is_empty() {
                if self.is_expected {
                    Some(Vec::new())
                } else {
                    None
                }
            } else {
                Some(self.expected)
            };

            Variant::call(
                identifier,
                types,
                self.value,
                input,
                expected,
                self.failure,
                self.events,
            )
        };

        Ok(Call::new(location, variant))
    }
}
