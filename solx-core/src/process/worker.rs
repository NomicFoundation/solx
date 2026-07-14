//!
//! A persistent worker subprocess owned by the pool.
//!

use std::io::BufReader;
use std::path::Path;
use std::process::Child;
use std::process::ChildStdout;
use std::process::Command;
use std::process::Stdio;

use crate::error::Error;
use crate::process::channel::FrameRead;
use crate::process::channel::FrameWrite;
use crate::process::job::Job;
use crate::process::output::Output as EVMOutput;
use crate::process::session::Session;

///
/// A persistent worker subprocess with its framed I/O channel.
///
/// Its `stderr` is inherited, so subprocess diagnostics stream directly to the parent.
/// Dropping a worker closes its `stdin`, which the worker loop treats as a shutdown request.
///
pub struct Worker {
    /// The worker subprocess handle.
    child: Child,
    /// The buffered response stream.
    stdout: BufReader<ChildStdout>,
}

impl Worker {
    ///
    /// Spawns a worker subprocess and sends it the session frame.
    ///
    pub fn spawn(executable: &Path, session: &Session) -> anyhow::Result<Self> {
        let mut command = Command::new(executable);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::inherit());
        command.arg("--recursive-process");

        let mut child = command.spawn().map_err(|error| {
            anyhow::anyhow!("{executable:?} subprocess spawning error: {error:?}")
        })?;
        child
            .stdin
            .as_mut()
            .expect("The worker stdin is always piped")
            .send(session)?;
        let stdout = BufReader::new(
            child
                .stdout
                .take()
                .expect("The worker stdout is always piped"),
        );
        Ok(Self { child, stdout })
    }

    ///
    /// Sends `job` to the worker and returns the compilation result it replies with.
    ///
    pub fn execute(&mut self, job: &Job) -> crate::Result<EVMOutput> {
        self.child
            .stdin
            .as_mut()
            .expect("The worker stdin is always piped")
            .send(job)?;
        match self.stdout.recv::<crate::Result<EVMOutput>>()? {
            Some(result) => result,
            None => Err(Error::Generic(format!(
                "The worker exited without replying: {}",
                self.child.wait()?
            ))),
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        drop(self.child.stdin.take());
        let _ = self.child.wait();
    }
}
