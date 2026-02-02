//!
//! Collectable errors trait.
//!

use std::io::Write;

use crate::output::error::Error;

///
/// Collectable errors trait.
///
/// Should be implemented by entities that can collect errors, and perform actions
/// on them as a list upon request.
///
pub trait Collectable {
    ///
    /// Filters errors out of messages to prepare the process for exiting.
    ///
    fn error_strings(&self) -> Vec<String>;

    ///
    /// Extracts warnings from the list of messages.
    ///
    fn take_warnings(&mut self) -> Vec<Error>;

    ///
    /// Checks if there is at least one error.
    ///
    fn has_errors(&self) -> bool;

    ///
    /// Collects errors into one message and bails, if there is at least one error.
    ///
    fn check_errors(&self) -> anyhow::Result<()> {
        if !self.has_errors() {
            return Ok(());
        }

        anyhow::bail!("{}", self.error_strings().join("\n"));
    }

    ///
    /// Exits the application with errors collected so far if there is at least one error.
    ///
    fn exit_on_error(&self) {
        if !self.has_errors() {
            return;
        }

        std::io::stderr()
            .write_all(self.error_strings().join("\n").as_bytes())
            .expect("Stderr writing error");
        std::process::exit(solx_utils::EXIT_CODE_FAILURE);
    }

    ///
    /// Removes warnings from the list of messages and prints them to stderr.
    ///
    fn take_and_write_warnings(&mut self) {
        let warnings = self.take_warnings();
        if warnings.is_empty() {
            return;
        }
        writeln!(
            std::io::stderr(),
            "{}",
            warnings
                .into_iter()
                .map(|error| error.to_string())
                .collect::<Vec<String>>()
                .join("\n")
        )
        .expect("Stderr writing error");
    }
}
