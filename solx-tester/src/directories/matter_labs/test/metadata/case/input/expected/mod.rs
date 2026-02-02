//!
//! The Matter Labs compiler test metadata expected data variant.
//!

pub mod variant;

use serde::Deserialize;

use crate::compilers::mode::Mode;

use self::variant::Variant;
use self::variant::extended::Extended;

///
/// The Matter Labs compiler test metadata expected data variant.
///
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Expected {
    /// The single expected data variant.
    Single(Variant),
    /// The several expected data variants, normally filtered by the compiler version and settings.
    Multiple(Vec<Variant>),
}

impl Expected {
    ///
    /// Creates successful deployer call expected data.
    ///
    pub fn successful_deployer_expected(instance: String) -> Self {
        Self::Single(Variant::Simple(vec![format!("{instance}.address")]))
    }

    ///
    /// Creates EVM interpreter benchmark expected data.
    ///
    pub fn successful_evm_interpreter_benchmark(exception: bool) -> Self {
        Self::Single(Variant::Extended(Extended {
            return_data: vec![],
            events: vec![],
            exception,
            compiler_version: None,
        }))
    }

    ///
    /// Returns exception flag for specified mode.
    ///
    pub fn exception(&self, mode: &Mode) -> anyhow::Result<bool> {
        let variants = match self {
            Self::Single(variant) => vec![variant],
            Self::Multiple(variants) => variants.iter().collect(),
        };
        let variant = variants
            .into_iter()
            .find(|variant| {
                let version = match variant {
                    Variant::Simple(_) => None,
                    Variant::Extended(inner) => inner.compiler_version.as_ref(),
                };
                match version {
                    Some(version) => mode.check_version(version),
                    None => true,
                }
            })
            .ok_or_else(|| anyhow::anyhow!("Version is not covered"))?;
        Ok(match variant {
            Variant::Simple(_) => false,
            Variant::Extended(inner) => inner.exception,
        })
    }
}
