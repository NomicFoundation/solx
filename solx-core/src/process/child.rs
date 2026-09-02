//!
//! The subprocess-side worker: reads a session, then compiles jobs until `stdin` closes.
//!

use std::thread::Builder;

use crate::error::Error;
use crate::process::channel::FrameRead;
use crate::process::channel::FrameWrite;
use crate::process::job::Job;
use crate::process::output::Output as EVMOutput;
use crate::process::session::Session;
use crate::project::contract::Contract;

///
/// Runs the worker loop on a dedicated stack-sized thread until `stdin` closes.
///
pub fn run() -> anyhow::Result<()> {
    Builder::new()
        .stack_size(crate::WORKER_THREAD_STACK_SIZE)
        .spawn(|| -> anyhow::Result<()> {
            let mut stdin = std::io::stdin().lock();
            let session: Session = stdin
                .recv()?
                .ok_or_else(|| anyhow::anyhow!("The worker received no session"))?;

            inkwell::support::error_handling::install_fatal_error_handler(llvm_fatal_error_handler);

            while let Some(job) = stdin.recv::<Job>()? {
                let result = Contract::compile_to_evm(
                    session.language,
                    session.solc_version.clone(),
                    job.contract_name.clone(),
                    job.contract_ir,
                    job.code_segment,
                    session.evm_version,
                    job.debug_info,
                    &session.output_selection,
                    job.immutables,
                    job.metadata_bytes,
                    job.optimizer_settings,
                    session.llvm_options.clone(),
                    session.output_config.clone(),
                )
                .map(EVMOutput::new)
                .map_err(|error| match error {
                    Error::Generic(error) => solx_standard_json::OutputError::new_error_contract(
                        Some(job.contract_name.path.as_str()),
                        error,
                    )
                    .into(),
                    error => error,
                });
                std::io::stdout().send(&result)?;
            }

            unsafe { inkwell::support::shutdown_llvm() };
            Ok(())
        })
        .expect("Threading error")
        .join()
        .expect("Threading error")
}

///
/// Handles LLVM fatal errors, e.g. failed stackification of a recursive function.
///
/// Replies with the error over the channel so that the parent reports it per contract
/// instead of only observing a dead worker; LLVM state is unrecoverable at this point,
/// so the process exits without the usual LLVM shutdown. Called from LLVM's
/// `report_fatal_error`, which aborts if the handler returns, so the process must exit here.
///
extern "C" fn llvm_fatal_error_handler(message: *const std::ffi::c_char) {
    let message = unsafe { std::ffi::CStr::from_ptr(message) }.to_string_lossy();
    let result: crate::Result<EVMOutput> = Err(Error::Generic(format!("LLVM error: {message}")));
    std::io::stdout()
        .send(&result)
        .unwrap_or_else(|error| panic!("LLVM fatal error response writing error: {error}"));
    std::process::exit(solx_utils::EXIT_CODE_SUCCESS);
}
