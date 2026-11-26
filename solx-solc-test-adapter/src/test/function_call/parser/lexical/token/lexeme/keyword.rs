//!
//! The lexical token keyword lexeme.
//!

use std::fmt;
use std::ops::RangeInclusive;
use std::str;

///
/// The keyword defined in the language.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword {
    /// The `left` keyword.
    Left,
    /// The `right` keyword.
    Right,
    /// The `FAILURE` keyword.
    Failure,

    /// The `library` keyword.
    Library,
    /// The `emit` keyword.
    Emit,
    /// The `from` keyword.
    From,
    /// The `anonymous` keyword.
    Anonymous,
    /// The `ether` keyword.
    Ether,
    /// The `wei` keyword.
    Wei,

    /// The `gas` keyword.
    Gas,
    /// The `ir` keyword.
    Ir,
    /// The `irOptimized` keyword.
    IrOptimized,
    /// The `legacy` keyword.
    Legacy,
    /// The `legacyOptimized` keyword.
    LegacyOptimized,
    /// The `ssaCFG` keyword.
    SsaCfg,
    /// The `ssaCFGOptimized` keyword.
    SsaCfgOptimized,
    /// The `code` keyword.
    Code,

    /// The `bool` type keyword.
    Bool,
    /// The `string` type keyword.
    String,
    /// The `address` type keyword.
    Address,
    /// The `function` type keyword.
    Function,
    /// The `uint{N}` type keyword.
    IntegerUnsigned {
        /// The unsigned type bit-length.
        bit_length: usize,
    },
    /// The `int{N}` type keyword.
    IntegerSigned {
        /// The signed type bit-length.
        bit_length: usize,
    },
    /// The `bytes{}` type keyword.
    Bytes {
        /// The bytes type byte-length.
        byte_length: Option<usize>,
    },

    /// The `true` literal keyword.
    True,
    /// The `false` literal keyword.
    False,
}

impl Keyword {
    /// The range including the minimal and maximal integer bit-lengths.
    pub const INTEGER_BIT_LENGTH_RANGE: RangeInclusive<usize> =
        solx_utils::BIT_LENGTH_BYTE..=solx_utils::BIT_LENGTH_FIELD;

    /// The range including the minimal and maximal bytes byte-lengths.
    pub const BYTES_BYTE_LENGTH_RANGE: RangeInclusive<usize> =
        solx_utils::BYTE_LENGTH_BYTE..=solx_utils::BYTE_LENGTH_FIELD;

    ///
    /// Creates a `uint{N}` keyword.
    ///
    pub fn new_integer_unsigned(bit_length: usize) -> Self {
        Self::IntegerUnsigned { bit_length }
    }

    ///
    /// Creates an `int{N}` keyword.
    ///
    pub fn new_integer_signed(bit_length: usize) -> Self {
        Self::IntegerSigned { bit_length }
    }

    ///
    /// Creates an `bytes{N}` keyword.
    ///
    pub fn new_bytes(byte_length: Option<usize>) -> Self {
        Self::Bytes { byte_length }
    }
}

///
/// The keyword parsing error.
///
/// If the parser returns such an error, it means that the word is not a keyword,
/// but an ordinar identifier or something else.
///
#[derive(Debug)]
pub enum Error {
    /// There is no number after the `uint` or `int` character.
    IntegerBitLengthEmpty,
    /// There is an invalid after the `uint` or `int` character.
    IntegerBitLengthNotNumeric(String),
    /// The bit-length is not multiple of `8`, which is forbidden.
    IntegerBitLengthNotMultipleOfEight(usize, usize),
    /// The bit-length is beyond the allowed range.
    IntegerBitLengthOutOfRange(usize, RangeInclusive<usize>),
    /// There is an invalid after the `bytes` character.
    BytesByteLengthNotNumeric(String),
    /// The byte-length is beyond the allowed range.
    BytesByteLengthOutOfRange(usize, RangeInclusive<usize>),
    /// The keyword is unknown, which means that the word is a valid identifier or something else.
    Unknown(String),
}

impl TryFrom<&str> for Keyword {
    type Error = Error;

    ///
    /// The converter checks if the number after the `uint` or `int` symbol represents a valid
    /// amount of bits for an integer value (Also valid amount of bytes after `bytes`). If the
    /// amount is not a valid, the word is treated as an ordinar identifier.
    ///
    fn try_from(input: &str) -> Result<Self, Self::Error> {
        match input {
            "left" => return Ok(Self::Left),
            "right" => return Ok(Self::Right),
            "FAILURE" => return Ok(Self::Failure),

            "library" => return Ok(Self::Library),
            "emit" => return Ok(Self::Emit),
            "from" => return Ok(Self::From),
            "anonymous" => return Ok(Self::Anonymous),
            "ether" => return Ok(Self::Ether),
            "wei" => return Ok(Self::Wei),

            "gas" => return Ok(Self::Gas),
            "ir" => return Ok(Self::Ir),
            "irOptimized" => return Ok(Self::IrOptimized),
            "legacy" => return Ok(Self::Legacy),
            "legacyOptimized" => return Ok(Self::LegacyOptimized),
            "ssaCFG" => return Ok(Self::SsaCfg),
            "ssaCFGOptimized" => return Ok(Self::SsaCfgOptimized),
            "code" => return Ok(Self::Code),

            "bool" => return Ok(Self::Bool),
            "string" => return Ok(Self::String),
            "address" => return Ok(Self::Address),
            "function" => return Ok(Self::Function),
            "bytes" => return Ok(Self::new_bytes(None)),

            "true" => return Ok(Self::True),
            "false" => return Ok(Self::False),

            _ => {}
        }

        if let Some("uint") = input.get(..4) {
            let bit_length = &input[4..];
            if bit_length.is_empty() {
                return Err(Error::IntegerBitLengthEmpty);
            }
            let bit_length = bit_length
                .parse::<usize>()
                .map_err(|_| Error::IntegerBitLengthNotNumeric(bit_length.to_owned()))?;
            if !Self::INTEGER_BIT_LENGTH_RANGE.contains(&bit_length) {
                return Err(Error::IntegerBitLengthOutOfRange(
                    bit_length,
                    Self::INTEGER_BIT_LENGTH_RANGE,
                ));
            }
            if bit_length % solx_utils::BIT_LENGTH_BYTE != 0 {
                return Err(Error::IntegerBitLengthNotMultipleOfEight(
                    bit_length,
                    solx_utils::BIT_LENGTH_BYTE,
                ));
            }
            return Ok(Self::new_integer_unsigned(bit_length));
        }

        if let Some("int") = input.get(..3) {
            let bit_length = &input[3..];
            if bit_length.is_empty() {
                return Err(Error::IntegerBitLengthEmpty);
            }
            let bit_length = bit_length
                .parse::<usize>()
                .map_err(|_| Error::IntegerBitLengthNotNumeric(bit_length.to_owned()))?;
            if !Self::INTEGER_BIT_LENGTH_RANGE.contains(&bit_length) {
                return Err(Error::IntegerBitLengthOutOfRange(
                    bit_length,
                    Self::INTEGER_BIT_LENGTH_RANGE,
                ));
            }
            if bit_length % solx_utils::BIT_LENGTH_BYTE != 0 {
                return Err(Error::IntegerBitLengthNotMultipleOfEight(
                    bit_length,
                    solx_utils::BIT_LENGTH_BYTE,
                ));
            }
            return Ok(Self::new_integer_signed(bit_length));
        }

        if let Some("bytes") = input.get(..5) {
            let byte_length = &input[5..];
            let byte_length = byte_length
                .parse::<usize>()
                .map_err(|_| Error::BytesByteLengthNotNumeric(byte_length.to_owned()))?;
            if !Self::BYTES_BYTE_LENGTH_RANGE.contains(&byte_length) {
                return Err(Error::BytesByteLengthOutOfRange(
                    byte_length,
                    Self::BYTES_BYTE_LENGTH_RANGE,
                ));
            }
            return Ok(Self::new_bytes(Some(byte_length)));
        }

        Err(Error::Unknown(input.to_owned()))
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Left => write!(f, "left"),
            Self::Right => write!(f, "right"),
            Self::Failure => write!(f, "FAILURE"),

            Self::Library => write!(f, "library"),
            Self::Emit => write!(f, "emit"),
            Self::From => write!(f, "from"),
            Self::Anonymous => write!(f, "anonymous"),
            Self::Ether => write!(f, "ether"),
            Self::Wei => write!(f, "wei"),

            Self::Gas => write!(f, "gas"),
            Self::Ir => write!(f, "ir"),
            Self::IrOptimized => write!(f, "irOptimized"),
            Self::Legacy => write!(f, "legacy"),
            Self::LegacyOptimized => write!(f, "legacyOptimized"),
            Self::SsaCfg => write!(f, "ssaCFG"),
            Self::SsaCfgOptimized => write!(f, "ssaCFGOptimized"),
            Self::Code => write!(f, "code"),

            Self::Bool => write!(f, "bool"),
            Self::String => write!(f, "string"),
            Self::Address => write!(f, "address"),
            Self::Function => write!(f, "function"),
            Self::IntegerUnsigned { bit_length } => write!(f, "uint{bit_length}"),
            Self::IntegerSigned { bit_length } => write!(f, "int{bit_length}"),
            Self::Bytes { byte_length: None } => write!(f, "bytes"),
            Self::Bytes {
                byte_length: Some(byte_length),
            } => write!(f, "bytes{byte_length}"),

            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
        }
    }
}
