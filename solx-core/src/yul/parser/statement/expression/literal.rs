//!
//! The Yul source code literal.
//!

use crate::declare_wrapper;
use crate::yul::parser::wrapper::Wrap;
use inkwell::values::BasicValue;
use num::Num;
use num::One;
use num::Zero;
use solx_yul::yul::lexer::token::lexeme::literal::Literal as LexicalLiteral;
use solx_yul::yul::lexer::token::lexeme::literal::boolean::Boolean as BooleanLiteral;
use solx_yul::yul::lexer::token::lexeme::literal::integer::Integer as IntegerLiteral;

declare_wrapper!(
    solx_yul::yul::parser::statement::expression::literal::Literal,
    Literal
);

impl Literal {
    ///
    /// Converts the literal into its LLVM.
    ///
    pub fn into_llvm<'ctx, C>(self, context: &C) -> anyhow::Result<solx_codegen_evm::Value<'ctx>>
    where
        C: solx_codegen_evm::IContext<'ctx>,
    {
        match self.0.inner {
            LexicalLiteral::Boolean(inner) => {
                let value = self
                    .0
                    .yul_type
                    .unwrap_or_default()
                    .wrap()
                    .into_llvm(context)
                    .const_int(
                        match inner {
                            BooleanLiteral::False => 0,
                            BooleanLiteral::True => 1,
                        },
                        false,
                    )
                    .as_basic_value_enum();

                let constant = match inner {
                    BooleanLiteral::False => num::BigUint::zero(),
                    BooleanLiteral::True => num::BigUint::one(),
                };

                Ok(solx_codegen_evm::Value::new_with_constant(value, constant))
            }
            LexicalLiteral::Integer(inner) => {
                let r#type = self
                    .0
                    .yul_type
                    .unwrap_or_default()
                    .wrap()
                    .into_llvm(context);
                let value = match inner {
                    IntegerLiteral::Decimal { ref inner } => r#type.const_int_from_string(
                        inner.as_str(),
                        inkwell::types::StringRadix::Decimal,
                    ),
                    IntegerLiteral::Hexadecimal { ref inner } => r#type.const_int_from_string(
                        &inner["0x".len()..],
                        inkwell::types::StringRadix::Hexadecimal,
                    ),
                }
                .expect("The value is valid")
                .as_basic_value_enum();

                let constant = match inner {
                    IntegerLiteral::Decimal { ref inner } => {
                        num::BigUint::from_str_radix(inner.as_str(), solx_utils::BASE_DECIMAL)
                    }
                    IntegerLiteral::Hexadecimal { ref inner } => num::BigUint::from_str_radix(
                        &inner["0x".len()..],
                        solx_utils::BASE_HEXADECIMAL,
                    ),
                }
                .expect("Always valid");

                Ok(solx_codegen_evm::Value::new_with_constant(value, constant))
            }
            LexicalLiteral::String(inner) => {
                let string = inner.inner;
                let r#type = self
                    .0
                    .yul_type
                    .unwrap_or_default()
                    .wrap()
                    .into_llvm(context);

                let mut hex_string = if inner.is_hexadecimal {
                    string.clone()
                } else {
                    let mut hex_string = String::with_capacity(solx_utils::BYTE_LENGTH_FIELD * 2);
                    let mut index = 0;
                    loop {
                        if index >= string.len() {
                            break;
                        }

                        if string[index..].starts_with('\\') {
                            index += 1;

                            if string[index..].starts_with('x') {
                                hex_string.push_str(&string[index + 1..index + 3]);
                                index += 3;
                            } else if string[index..].starts_with('u') {
                                let codepoint_str = &string[index + 1..index + 5];
                                let codepoint = u32::from_str_radix(
                                    codepoint_str,
                                    solx_utils::BASE_HEXADECIMAL,
                                )
                                .map_err(|error| {
                                    anyhow::anyhow!("Invalid codepoint `{codepoint_str}`: {error}")
                                })?;
                                let unicode_char = char::from_u32(codepoint).ok_or_else(|| {
                                    anyhow::anyhow!("Invalid codepoint {codepoint}")
                                })?;
                                let mut unicode_bytes = vec![0u8; 3];
                                unicode_char.encode_utf8(&mut unicode_bytes);

                                for byte in unicode_bytes.into_iter() {
                                    hex_string.push_str(format!("{byte:02x}").as_str());
                                }
                                index += 5;
                            } else if string[index..].starts_with('t') {
                                hex_string.push_str("09");
                                index += 1;
                            } else if string[index..].starts_with('n') {
                                hex_string.push_str("0a");
                                index += 1;
                            } else if string[index..].starts_with('r') {
                                hex_string.push_str("0d");
                                index += 1;
                            } else if string[index..].starts_with("\r\n") {
                                index += 2;
                            } else if string[index..].starts_with('\n') {
                                index += 1;
                            } else {
                                hex_string
                                    .push_str(format!("{:02x}", string.as_bytes()[index]).as_str());
                                index += 1;
                            }
                        } else {
                            hex_string
                                .push_str(format!("{:02x}", string.as_bytes()[index]).as_str());
                            index += 1;
                        }
                    }
                    hex_string
                };

                if hex_string.len() > solx_utils::BYTE_LENGTH_FIELD * 2 {
                    return Ok(solx_codegen_evm::Value::new_with_original(
                        r#type.const_zero().as_basic_value_enum(),
                        string,
                    ));
                }

                if hex_string.len() < solx_utils::BYTE_LENGTH_FIELD * 2 {
                    hex_string.push_str(
                        "0".repeat((solx_utils::BYTE_LENGTH_FIELD * 2) - hex_string.len())
                            .as_str(),
                    );
                }

                let value = r#type
                    .const_int_from_string(
                        hex_string.as_str(),
                        inkwell::types::StringRadix::Hexadecimal,
                    )
                    .expect("The value is valid")
                    .as_basic_value_enum();
                Ok(solx_codegen_evm::Value::new_with_original(value, string))
            }
        }
    }
}
