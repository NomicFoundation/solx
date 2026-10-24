//!
//! `solx` Sourcify sweep compiler invocation.
//!

use std::collections::BTreeMap;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::sync::LazyLock;
use std::time::Duration;
use std::time::Instant;

use regex::Regex;
use sysinfo::Pid;
use sysinfo::ProcessRefreshKind;
use sysinfo::ProcessesToUpdate;
use sysinfo::System;

use super::record::leg::Leg;
use super::record::leg::kind::Kind;

/// The number of trailing standard error characters kept in a record.
const STDERR_TAIL: usize = 2000;

/// The number of error-severity messages kept in a record.
const MAX_MESSAGES: usize = 5;

/// The interval at which a running compiler is polled for exit.
const POLL_INTERVAL: Duration = Duration::from_millis(20);

static LIBRARY_HASH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$[0-9a-f]{64}\$").expect("Always valid"));
static SOL_PATH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"[^\s"'`]+\.sol\b"#).expect("Always valid"));
static HEX_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"0x[0-9a-fA-F]+").expect("Always valid"));
static DECIMAL_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b\d+\b").expect("Always valid"));
static PANIC_WITH_MESSAGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"panicked at [^\n]+:\n([^\n]*)").expect("Always valid"));

///
/// Compiles a standard JSON input and classifies the result.
///
/// Errors only on harness breakage: a compiler that cannot be spawned, waited for or read.
///
pub fn compile(
    compiler: &Path,
    input: Vec<u8>,
    target: &str,
    timeout: Duration,
    memory_limit_mib: Option<u64>,
) -> anyhow::Result<Leg> {
    let mut command = Command::new(compiler);
    command.arg("--standard-json");
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut command, 0);
    let mut process = command
        .spawn()
        .map_err(|error| anyhow::anyhow!("{command:?} process spawning error: {error}"))?;

    let mut stdin = process.stdin.take().expect("Always piped");
    let mut stdout = process.stdout.take().expect("Always piped");
    let mut stderr = process.stderr.take().expect("Always piped");
    // A compiler that exits before reading all input closes the pipe; its output says why.
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(input.as_slice());
    });
    let stdout_reader = std::thread::spawn(move || {
        let mut buffer = Vec::new();
        stdout.read_to_end(&mut buffer).map(|_| buffer)
    });
    let stderr_reader = std::thread::spawn(move || {
        let mut buffer = Vec::new();
        stderr.read_to_end(&mut buffer).map(|_| buffer)
    });

    let mut system = System::new();
    let started = Instant::now();
    let status = loop {
        if let Some(status) = process
            .try_wait()
            .map_err(|error| anyhow::anyhow!("{command:?} waiting error: {error}"))?
        {
            break status;
        }
        if started.elapsed() > timeout {
            kill(&mut process);
            return Ok(Leg {
                kind: Kind::Timeout,
                ..Leg::default()
            });
        }
        if let Some(memory_limit_mib) = memory_limit_mib
            && memory_usage(&mut system, &process) > memory_limit_mib * 1024 * 1024
        {
            kill(&mut process);
            return Ok(Leg {
                kind: Kind::Abort,
                signature: Some(format!("memory limit of {memory_limit_mib} MiB exceeded")),
                ..Leg::default()
            });
        }
        std::thread::sleep(POLL_INTERVAL);
    };

    let _ = writer.join();
    let stdout = stdout_reader
        .join()
        .expect("Never panics")
        .map_err(|error| anyhow::anyhow!("{command:?} stdout reading error: {error}"))?;
    let stderr = stderr_reader
        .join()
        .expect("Never panics")
        .map_err(|error| anyhow::anyhow!("{command:?} stderr reading error: {error}"))?;
    let stderr = String::from_utf8_lossy(stderr.as_slice()).into_owned();

    Ok(classify(status, stdout.as_slice(), stderr.as_str(), target))
}

///
/// Collapses a diagnostic into a bucket key: its first two lines with paths, library hashes and
/// numbers replaced.
///
pub fn normalize(text: &str) -> String {
    let text = text.replace("\\n", "\n");
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(2)
        .collect();
    let line = if lines.is_empty() {
        "<empty>".to_owned()
    } else {
        lines.join(" ")
    };
    let line = LIBRARY_HASH.replace_all(line.as_str(), "$$<hash>$$");
    let line = SOL_PATH.replace_all(line.as_ref(), "<file>");
    let line = HEX_NUMBER.replace_all(line.as_ref(), "0x…");
    let line = DECIMAL_NUMBER.replace_all(line.as_ref(), "N");
    line.chars().take(160).collect()
}

///
/// Classifies a finished compiler run: ok | error | no-bytecode | panic | abort.
///
fn classify(status: std::process::ExitStatus, stdout: &[u8], stderr: &str, target: &str) -> Leg {
    let stderr_tail = tail(stderr, STDERR_TAIL);
    let mut leg = Leg {
        exit: status.code(),
        signal: signal(status),
        ..Leg::default()
    };

    if let Some(signal) = leg.signal {
        let last_line = stderr_tail.trim().lines().last().unwrap_or_default();
        leg.kind = Kind::Abort;
        leg.signature = Some(format!("signal {signal}: {}", normalize(last_line)));
        leg.stderr = Some(stderr_tail);
        return leg;
    }
    if stderr.contains("panicked at") {
        leg.kind = Kind::Panic;
        leg.signature = Some(normalize(panic_message(stderr)));
        leg.stderr = Some(stderr_tail);
        return leg;
    }

    let Ok(output) = serde_json::from_slice::<serde_json::Value>(stdout) else {
        let message = if stderr_tail.is_empty() {
            format!("exit {:?} without standard-JSON output", leg.exit)
        } else {
            stderr_tail.clone()
        };
        leg.kind = Kind::Error;
        leg.signature = Some(normalize(message.as_str()));
        leg.stderr = Some(stderr_tail);
        return leg;
    };

    let messages: Vec<String> = output
        .get("errors")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter(|error| error.get("severity").and_then(serde_json::Value::as_str) == Some("error"))
        .map(|error| {
            error
                .get("message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_owned()
        })
        .collect();
    if let Some(first) = messages.first() {
        leg.kind = Kind::Error;
        leg.signature = Some(normalize(first.as_str()));
        leg.error_count = Some(messages.len());
        leg.messages = messages.into_iter().take(MAX_MESSAGES).collect();
        if !stderr_tail.trim().is_empty() {
            leg.stderr = Some(stderr_tail);
        }
        return leg;
    }

    let contracts = output
        .get("contracts")
        .and_then(serde_json::Value::as_object);
    let is_target_compiled = contracts
        .and_then(|contracts| contracts.get(target))
        .and_then(serde_json::Value::as_object)
        .is_some_and(|file| file.values().any(has_bytecode));
    if !is_target_compiled {
        let emitted: BTreeMap<String, Vec<String>> = contracts
            .into_iter()
            .flatten()
            .map(|(path, file)| {
                let names = file
                    .as_object()
                    .into_iter()
                    .flatten()
                    .filter(|(_name, contract)| has_bytecode(contract))
                    .map(|(name, _contract)| name.to_owned())
                    .collect();
                (path.to_owned(), names)
            })
            .collect();
        leg.kind = Kind::NoBytecode;
        leg.signature = Some("no errors reported but no bytecode for the target source".to_owned());
        leg.stderr = Some(stderr_tail);
        leg.emitted = Some(emitted);
    }
    leg
}

///
/// The panic message, which is the bucket key; the location stays in the recorded stderr tail.
///
fn panic_message(stderr: &str) -> &str {
    PANIC_WITH_MESSAGE
        .captures(stderr)
        .and_then(|captures| captures.get(1))
        .map(|message| message.as_str())
        .filter(|message| !message.is_empty())
        .unwrap_or("<no panic message>")
}

///
/// Whether a standard JSON output contract has bytecode.
///
fn has_bytecode(contract: &serde_json::Value) -> bool {
    contract
        .pointer("/evm/bytecode/object")
        .and_then(serde_json::Value::as_str)
        .is_some()
}

///
/// The last `count` characters of `text`.
///
fn tail(text: &str, count: usize) -> String {
    let skip = text.chars().count().saturating_sub(count);
    text.chars().skip(skip).collect()
}

///
/// The signal that killed the process.
///
#[cfg(unix)]
fn signal(status: std::process::ExitStatus) -> Option<i32> {
    std::os::unix::process::ExitStatusExt::signal(&status)
}

///
/// The signal that killed the process.
///
#[cfg(not(unix))]
fn signal(_status: std::process::ExitStatus) -> Option<i32> {
    None
}

///
/// Kills the compiler with the worker processes it spawned, which share its process group and
/// would otherwise keep compiling.
///
#[cfg(unix)]
fn kill(process: &mut Child) {
    // SAFETY: `killpg` only sends a signal to the group the compiler leads.
    unsafe {
        libc::killpg(process.id() as libc::pid_t, libc::SIGKILL);
    }
    let _ = process.wait();
}

///
/// Kills the compiler with the worker processes it spawned, children first, as they would
/// otherwise keep compiling.
///
#[cfg(not(unix))]
fn kill(process: &mut Child) {
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().without_tasks(),
    );
    for pid in descendants(&system, Pid::from_u32(process.id()))
        .into_iter()
        .rev()
    {
        if let Some(descendant) = system.process(pid) {
            descendant.kill();
        }
    }
    let _ = process.wait();
}

///
/// The resident memory of the compiler and the worker processes it spawned, in bytes.
///
/// Threads are left out, as Linux would otherwise list each one as a process with the memory of
/// the whole process.
///
fn memory_usage(system: &mut System, process: &Child) -> u64 {
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_memory().without_tasks(),
    );
    descendants(system, Pid::from_u32(process.id()))
        .into_iter()
        .filter_map(|pid| system.process(pid))
        .map(|process| process.memory())
        .sum()
}

///
/// The process and all its descendants, parents before children.
///
fn descendants(system: &System, root: Pid) -> Vec<Pid> {
    let mut tree = vec![root];
    let mut index = 0;
    while let Some(&parent) = tree.get(index) {
        tree.extend(
            system
                .processes()
                .values()
                .filter(|process| process.parent() == Some(parent))
                .map(|process| process.pid()),
        );
        index += 1;
    }
    tree
}
