//!
//! The pool of persistent worker subprocesses.
//!

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::Once;

use crate::error::Error;
use crate::process::job::Job;
use crate::process::output::Output as EVMOutput;
use crate::process::session::Session;
use crate::process::worker::Worker;
use crate::project::contract::Contract;

/// The lock-poisoning invariant shared by the idle-pool accessors.
const POISON: &str = "lock is never poisoned because worker threads do not panic";

/// One-time installation of the in-process fatal error handler.
static FATAL_ERROR_HANDLER: Once = Once::new();

///
/// The pool of persistent worker subprocesses.
///
/// A worker returns to the idle list after every job it survives — a success or a per-unit
/// compile error alike — and is retired only by a transport failure or a `StackTooDeep` reply,
/// after which the child exits. The number of live workers never exceeds the number of
/// dispatching threads.
///
/// By default jobs compile on the dispatching threads themselves and no subprocess is ever
/// spawned: codegen state is per-`LLVMContext` and per-module, so concurrent in-process jobs
/// do not interfere. Windows is the exception — concurrent in-process codegen still corrupts
/// LLVM's `PrettyStackTrace` entry stack there ("destruction is out of order" abort), so the
/// worker pool stays its default. `SOLX_SUBPROCESS` overrides both ways: `0` forces
/// in-process, any other value forces the pool — the isolation escape hatch: a crash or LLVM
/// fatal error takes down one worker, not the compiler.
///
pub struct Pool {
    /// The worker executable path.
    executable: PathBuf,
    /// The project-wide data sent to every spawned worker.
    session: Session,
    /// The idle workers available for checkout.
    idle: Mutex<Vec<Worker>>,
    /// Whether jobs compile on the dispatching threads instead of worker subprocesses.
    in_process: bool,
}

impl Pool {
    ///
    /// Creates a pool that dispatches jobs of `session`.
    ///
    pub fn new(session: Session) -> anyhow::Result<Self> {
        let in_process = match std::env::var_os("SOLX_SUBPROCESS") {
            Some(value) => value == "0",
            None => !cfg!(windows),
        };
        if in_process {
            FATAL_ERROR_HANDLER.call_once(|| unsafe {
                inkwell::support::error_handling::install_fatal_error_handler(fatal_error_handler);
            });
        }
        let executable = crate::process::EXECUTABLE
            .get()
            .cloned()
            .unwrap_or_else(|| {
                std::env::current_exe().expect("Current executable path getting error")
            });
        Ok(Self {
            executable,
            session,
            idle: Mutex::new(Vec::new()),
            in_process,
        })
    }

    ///
    /// Compiles one translation unit on a pooled or freshly spawned worker.
    ///
    /// A worker that survives the job rejoins the pool, including after a per-unit compile error.
    /// A transport failure or a `StackTooDeep` reply retires it instead.
    ///
    pub fn execute(&self, job: &Job) -> crate::Result<EVMOutput> {
        if self.in_process {
            return self.execute_in_process(job);
        }
        let mut worker = match self.idle.lock().expect(POISON).pop() {
            Some(worker) => worker,
            None => Worker::spawn(self.executable.as_path(), &self.session)?,
        };
        match worker.execute(job) {
            Ok(output) => {
                self.idle.lock().expect(POISON).push(worker);
                Ok(output)
            }
            Err(error) => {
                if matches!(error, Error::StandardJson(_)) {
                    self.idle.lock().expect(POISON).push(worker);
                }
                Err(error)
            }
        }
    }

    ///
    /// Compiles one translation unit on the calling thread.
    ///
    /// The job takes the same serde roundtrip a worker subprocess would receive, so both
    /// modes compile identical inputs.
    ///
    fn execute_in_process(&self, job: &Job) -> crate::Result<EVMOutput> {
        let mut buffer = Vec::with_capacity(crate::r#const::DEFAULT_SERDE_BUFFER_SIZE);
        ciborium::into_writer(job, &mut buffer)
            .map_err(|error| anyhow::anyhow!("In-process job serializing error: {error}"))?;
        let job: Job =
            ciborium::de::from_reader_with_recursion_limit(buffer.as_slice(), usize::MAX)
                .map_err(|error| anyhow::anyhow!("In-process job deserializing error: {error}"))?;

        let contract_path = job.contract_name.path.clone();
        Contract::compile_to_evm(
            self.session.language,
            self.session.solc_version.clone(),
            job.contract_name,
            job.contract_ir,
            job.code_segment,
            self.session.evm_version,
            job.debug_info,
            &self.session.output_selection,
            job.immutables,
            job.metadata_bytes,
            job.optimizer_settings,
            self.session.llvm_options.clone(),
            self.session.output_config.clone(),
        )
        .map(EVMOutput::new)
        .map_err(|error| match error {
            Error::Generic(error) => solx_standard_json::OutputError::new_error_contract(
                Some(contract_path.as_str()),
                error,
            )
            .into(),
            error => error,
        })
    }
}

///
/// Reproduces the default handler's diagnostics on LLVM fatal errors, but leaves via
/// `_exit`: the default handler's `exit(1)` runs atexit destructors that tear down LLVM
/// globals under concurrently compiling threads, and `abort` raises a signal that LLVM's
/// crash handler decorates with bug-report boilerplate.
///
extern "C" fn fatal_error_handler(message: *const std::ffi::c_char) {
    let message = unsafe { std::ffi::CStr::from_ptr(message) }.to_string_lossy();
    eprintln!("LLVM ERROR: {message}");
    unsafe { _exit(1) }
}

unsafe extern "C" {
    /// Process exit without atexit handlers; provided by every libc, including mingw's.
    fn _exit(code: i32) -> !;
}
