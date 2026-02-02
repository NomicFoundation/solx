//!
//! `solx` tester summary element.
//!

pub mod outcome;

use colored::Colorize;

use crate::test::description::TestDescription;

use self::outcome::Outcome;
use self::outcome::passed_variant::PassedVariant;

///
/// `solx` tester summary element.
///
#[derive(Debug)]
pub struct Element {
    /// Information about test instance.
    pub test_description: TestDescription,
    /// The test outcome.
    pub outcome: Outcome,
}

impl Element {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(name: TestDescription, outcome: Outcome) -> Self {
        Self {
            test_description: name,
            outcome,
        }
    }

    ///
    /// Prints the element.
    ///
    pub fn print(&self, verbosity: bool) -> Option<String> {
        match self.outcome {
            Outcome::Passed { .. } if !verbosity => return None,
            Outcome::Ignored => return None,
            _ => {}
        }

        let outcome = match self.outcome {
            Outcome::Passed { .. } => "PASSED".green(),
            Outcome::Failed { .. } => "FAILED".bright_red(),
            Outcome::Invalid { .. } => "INVALID".red(),
            Outcome::Ignored => "IGNORED".bright_black(),
        };

        let details = match self.outcome {
            Outcome::Passed {
                ref variant,
                ref group,
            } => {
                let mut details = Vec::new();
                match variant {
                    PassedVariant::Deploy {
                        deploy_size,
                        runtime_size,
                        gas,
                        ..
                    } => {
                        details.push(
                            format!("deploy size {deploy_size}")
                                .bright_white()
                                .to_string(),
                        );
                        details.push(
                            format!("runtime size {runtime_size}")
                                .bright_white()
                                .to_string(),
                        );
                        details.push(format!("gas {gas}").bright_white().to_string());
                    }
                    PassedVariant::Runtime { gas } => {
                        details.push(format!("gas {gas}").bright_white().to_string());
                    }
                    _ => {}
                };
                if let Some(group) = group {
                    details.push(format!("group '{group}'").bright_white().to_string())
                };
                if details.is_empty() {
                    "".to_string()
                } else {
                    format!("({})", details.join(", "))
                }
            }
            Outcome::Failed {
                ref expected,
                ref found,
                ref calldata,
            } => {
                format!(
                    "(expected {}, found {}, calldata {})",
                    ron::ser::to_string_pretty(expected, ron::ser::PrettyConfig::default())
                        .expect("Always valid"),
                    ron::ser::to_string_pretty(found, ron::ser::PrettyConfig::default())
                        .expect("Always valid"),
                    calldata,
                )
            }
            Outcome::Invalid { ref error } => error.to_string(),
            _ => String::new(),
        };

        Some(format!(
            "{:16} {:>7} {} {}",
            self.test_description
                .mode
                .as_ref()
                .map(|mode| mode.to_string())
                .unwrap_or_default()
                .bright_white(),
            outcome,
            self.test_description.selector,
            details
        ))
    }
}
