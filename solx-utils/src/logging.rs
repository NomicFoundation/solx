//!
//! Logging helpers.
//!

use colored::Colorize;

///
/// Formats a verb as a `cargo` OK status string.
///
pub fn cargo_status_ok(verb: &str) -> String {
    format!(
        "{}{}",
        " ".repeat(12 - verb.len()),
        verb.bright_green().bold()
    )
}

///
/// Formats a verb as a `cargo` error status string.
///
pub fn cargo_status_error(verb: &str) -> String {
    format!(
        "{}{}",
        " ".repeat(12 - verb.len()),
        verb.bright_red().bold()
    )
}
