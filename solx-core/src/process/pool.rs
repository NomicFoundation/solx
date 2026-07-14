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
/// A worker returns to the idle list after every job it survives — a success or a per-unit
/// compile error alike — and is retired only by a transport failure or a `StackTooDeep` reply,
/// after which the child exits. The number of live workers never exceeds the number of
/// dispatching threads.
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
    /// A worker that survives the job rejoins the pool, including after a per-unit compile error.
    /// A transport failure or a `StackTooDeep` reply retires it instead.
    ///
    pub fn execute(&self, job: &Job) -> crate::Result<EVMOutput> {
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
}
