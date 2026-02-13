//!
//! Solidity compiler executable.
//!

use std::io::Write;

use clap::Parser;

#[cfg(not(any(feature = "solc", feature = "slang")))]
compile_error!(
    "No Solidity frontend is enabled. Please enable exactly one: --features solc or --features slang."
);
#[cfg(all(feature = "solc", feature = "slang"))]
compile_error!(
    "Multiple Solidity frontends are enabled. Please enable exactly one: --features solc or --features slang."
);

///
/// The application entry point.
///
fn main() -> anyhow::Result<()> {
    let arguments = match solx_core::Arguments::try_parse() {
        Ok(arguments) => arguments,
        Err(error) => {
            let error: String = error.to_string();
            eprintln!(
                "{}",
                error.strip_prefix("Error: ").unwrap_or(error.as_str())
            );
            std::process::exit(solx_utils::EXIT_CODE_FAILURE);
        }
    };
    let is_standard_json = arguments.standard_json.is_some();
    let messages = arguments.validate();
    if messages
        .lock()
        .expect("Sync")
        .iter()
        .all(|error| error.severity != "error")
    {
        if !is_standard_json {
            std::io::stderr()
                .write_all(
                    messages
                        .lock()
                        .expect("Sync")
                        .drain(..)
                        .map(|error| error.to_string())
                        .collect::<Vec<String>>()
                        .join("\n")
                        .as_bytes(),
                )
                .expect("Stderr writing error");
        }
        #[cfg(feature = "solc")]
        let frontend = solx::Solc::default();
        #[cfg(all(feature = "slang", not(feature = "solc")))]
        let frontend = solx_slang::SlangFrontend::default();

        if let Err(error) = solx_core::main(arguments, frontend, messages.clone()) {
            messages
                .lock()
                .expect("Sync")
                .push(solx_standard_json::OutputError::new_error(error));
        }
    }

    if is_standard_json {
        let output = solx_standard_json::Output::new_with_messages(messages);
        output.write_and_exit(&solx_standard_json::InputSelection::default());
    }

    let exit_code = if messages
        .lock()
        .expect("Sync")
        .iter()
        .any(|error| error.severity == "error")
    {
        solx_utils::EXIT_CODE_FAILURE
    } else {
        solx_utils::EXIT_CODE_SUCCESS
    };
    std::io::stderr()
        .write_all(
            messages
                .lock()
                .expect("Sync")
                .iter()
                .map(|error| error.to_string())
                .collect::<Vec<String>>()
                .join("\n")
                .as_bytes(),
        )
        .expect("Stderr writing error");
    std::process::exit(exit_code);
}
