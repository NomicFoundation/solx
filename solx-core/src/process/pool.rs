//!
//! The pool of persistent worker subprocesses.
//!

use std::path::PathBuf;
use std::sync::Mutex;

use crate::process::job::Job;
use crate::process::output::Output as EVMOutput;
use crate::process::session::Session;
use crate::process::worker::Worker;

/// The lock-poisoning invariant shared by the idle-pool accessors.
const POISON: &str = "lock is never poisoned because worker threads do not panic";

///
/// The pool of persistent worker subprocesses.
///
/// Workers are spawned on demand and returned to the idle list after each successful job,
/// so the number of live workers never exceeds the number of dispatching threads.
///
pub struct Pool {
    /// The worker executable path.
    executable: PathBuf,
    /// The project-wide data sent to every spawned worker.
    session: Session,
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
            session,
            idle: Mutex::new(Vec::new()),
        })
    }

    ///
    /// Compiles one translation unit on a pooled or freshly spawned worker.
    ///
    /// On success the worker rejoins the pool; any error drops it through the early return.
    ///
    pub fn execute(&self, job: &Job) -> crate::Result<EVMOutput> {
        let idle = self.idle.lock().expect(POISON).pop();
        let mut worker = match idle {
            Some(worker) => worker,
            None => Worker::spawn(self.executable.as_path(), &self.session)?,
        };
        let output = worker.execute(job)?;
        self.idle.lock().expect(POISON).push(worker);
        Ok(output)
    }
}
