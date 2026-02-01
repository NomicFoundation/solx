//!
//! The type builder.
//!

use crate::test::function_call::parser::lexical::Keyword;
use crate::test::function_call::parser::lexical::Location;
use crate::test::function_call::parser::syntax::tree::r#type::Type;
use crate::test::function_call::parser::syntax::tree::r#type::variant::Variant as TypeVariant;

///
/// The type builder.
///
#[derive(Default)]
pub struct Builder {
    /// The location of the syntax construction.
    location: Option<Location>,
    /// The type keyword, which means that the type is intrinsic.
    keyword: Option<Keyword>,
    /// The array type, which means that the type is an array.
    array_type: Option<Type>,
    /// The array size expression, which means that the type is an array.
    array_size: Option<String>,
    /// The tuple elements, which means that the type is a tuple.
    tuple_element_types: Vec<Type>,
}

/// The invalid type keyword panic, which is prevented by the type parser.
static BUILDER_TYPE_INVALID_KEYWORD: &str =
    "The type builder has got an unexpected non-type keyword: ";

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
    pub fn set_array_type(&mut self, value: Type) {
        self.array_type = Some(value);
    }

    ///
    /// Sets the corresponding builder value.
    ///
    pub fn set_array_size(&mut self, value: String) {
        self.array_size = Some(value);
    }

    ///
    /// Pushes the corresponding builder value.
    ///
    pub fn push_tuple_element_type(&mut self, value: Type) {
        self.tuple_element_types.push(value)
    }

    ///
    /// Finalizes the builder and returns the built value.
    ///
    /// # Panics
    /// If some of the required items has not been set.
    ///
    pub fn finish(mut self) -> Type {
        let location = self
            .location
            .take()
            .unwrap_or_else(|| panic!("{}{}", "Mandatory value missing: ", "location"));

        let variant = if let Some(keyword) = self.keyword.take() {
            match keyword {
                Keyword::Bool => TypeVariant::boolean(),
                Keyword::String => TypeVariant::string(),
                Keyword::Address => TypeVariant::address(),
                Keyword::Function => TypeVariant::function(),
                Keyword::IntegerUnsigned { bit_length } => {
                    TypeVariant::integer_unsigned(bit_length)
                }
                Keyword::IntegerSigned { bit_length } => TypeVariant::integer_signed(bit_length),
                Keyword::Bytes { byte_length } => TypeVariant::bytes(byte_length),
                keyword => panic!("{}{}", self::BUILDER_TYPE_INVALID_KEYWORD, keyword),
            }
        } else if let Some(array_type) = self.array_type.take() {
            TypeVariant::array(array_type, self.array_size.take())
        } else if !self.tuple_element_types.is_empty() {
            TypeVariant::tuple(self.tuple_element_types)
        } else {
            TypeVariant::tuple(Vec::new())
        };

        Type::new(location, variant)
    }
}
