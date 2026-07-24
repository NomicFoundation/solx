//!
//! The subprocess-side worker: reads a session, then compiles jobs until `stdin` closes.
//!

use std::sync::atomic::Ordering;
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

            inkwell::support::error_handling::install_stack_error_handler(evm_stack_error_handler);

            while let Some(job) = stdin.recv::<Job>()? {
                solx_codegen_evm::IS_SIZE_FALLBACK.store(
                    job.optimizer_settings.is_fallback_to_size_active(),
                    Ordering::Relaxed,
                );
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
/// Handles LLVM stack-too-deep errors.
///
/// # Safety
///
/// This function is unsafe because it is called from the LLVM stackifier.
/// The function must terminate the process after handling the error.
///
unsafe extern "C" fn evm_stack_error_handler(spill_area_size: u64) {
    let result: crate::Result<EVMOutput> = Err(Error::stack_too_deep(
        spill_area_size,
        solx_codegen_evm::IS_SIZE_FALLBACK.load(Ordering::Relaxed),
    ));
    std::io::stdout()
        .send(&result)
        .unwrap_or_else(|error| panic!("Stack-too-deep response writing error: {error}"));
    unsafe { inkwell::support::shutdown_llvm() };
    std::process::exit(solx_utils::EXIT_CODE_SUCCESS);
}
