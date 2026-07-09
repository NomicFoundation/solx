//!
//! The pool of persistent worker subprocesses.
//!

use std::path::PathBuf;
use std::sync::Mutex;

use crate::error::Error;
use crate::process::job::Job;
use crate::process::output::Output as EVMOutput;
use crate::process::session::Session;
use crate::process::worker::Worker;

/// The lock-poisoning invariant shared by the idle-pool accessors.
const POISON: &str = "lock is never poisoned because worker threads do not panic";

///
/// The pool of persistent worker subprocesses.
///
/// Workers are spawned on demand and returned to the idle list after each job,
/// so the number of live workers never exceeds the number of dispatching threads.
///
pub struct Pool {
    /// The worker executable path.
    executable: PathBuf,
    /// The project-wide data sent to every spawned worker.
    session: Session,
    /// Whether workers may outlive a single job.
    /// LLVM list-typed command line options accumulate occurrences with every parsed unit,
    /// so sessions carrying extra LLVM arguments run every job in a fresh worker.
    reuse_workers: bool,
    /// The idle workers available for checkout.
    idle: Mutex<Vec<Worker>>,
}

impl Pool {
    ///
    /// Creates a pool that dispatches jobs of `session` to worker subprocesses.
    ///
    pub fn new(session: Session) -> anyhow::Result<Self> {
        let executable = crate::process::EXECUTABLE
            .get()
            .cloned()
            .unwrap_or_else(|| {
                std::env::current_exe().expect("Current executable path getting error")
            });
        Ok(Self {
            executable,
            reuse_workers: session.llvm_options.is_empty(),
            session,
            idle: Mutex::new(Vec::new()),
        })
    }

    ///
    /// Compiles a single translation unit, retrying once on a fresh worker if a reused one dies.
    ///
    pub fn execute(
        &self,
        contract_name: &solx_utils::ContractName,
        job: &Job,
    ) -> crate::Result<EVMOutput> {
        let idle = self.idle.lock().expect(POISON).pop();
        if let Some(worker) = idle
            && let Ok(result) = self.dispatch(worker, job)
        {
            return result;
        }
        self.dispatch(
            Worker::spawn(self.executable.as_path(), &self.session)?,
            job,
        )
        .unwrap_or_else(|error| {
            Err(solx_standard_json::OutputError::new_error_contract(
                Some(contract_name.path.as_str()),
                format!("{:?} subprocess error: {error}", self.executable),
            )
            .into())
        })
    }

    ///
    /// Runs `job` on `worker`, returning it to the idle list when it stays reusable.
    ///
    fn dispatch(&self, mut worker: Worker, job: &Job) -> anyhow::Result<crate::Result<EVMOutput>> {
        let result = worker.execute(job)?;
        if self.reuse_workers
            && job.optimizer_settings.spill_area_size.is_none()
            && !matches!(result, Err(Error::StackTooDeep(_)))
        {
            self.idle.lock().expect(POISON).push(worker);
        }
        Ok(result)
    }
}
