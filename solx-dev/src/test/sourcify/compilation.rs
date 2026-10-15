//!
//! `solx` Sourcify sweep compiler invocation.
//!

use std::collections::BTreeMap;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use std::sync::LazyLock;
use std::time::Duration;
use std::time::Instant;

use regex::Regex;

use super::record::Kind;
use super::record::Leg;

/// The number of trailing standard error characters kept in a record.
const STDERR_TAIL: usize = 2000;

/// The number of error-severity messages kept in a record.
const MAX_MESSAGES: usize = 5;

/// The interval at which a running compiler is polled for exit.
const POLL_INTERVAL: Duration = Duration::from_millis(20);

static SOL_PATH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"[^\s"'`]+\.sol\b"#).expect("Always valid"));
static HEX_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"0x[0-9a-fA-F]+").expect("Always valid"));
static DECIMAL_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b\d+\b").expect("Always valid"));
static PANIC_WITH_MESSAGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"panicked at [^\n]+:\n([^\n]*)").expect("Always valid"));
static PANIC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"panicked at [^\n]+").expect("Always valid"));

///
/// Compiles a standard JSON input and classifies the result.
///
/// Errors only on harness breakage: a compiler that cannot be spawned or waited for.
///
pub fn compile(
    compiler: &Path,
    input: Vec<u8>,
    target: &str,
    timeout: Duration,
    memory_limit_mib: u64,
) -> anyhow::Result<Leg> {
    let mut command = Command::new(compiler);
    command.arg("--standard-json");
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    if memory_limit_mib > 0 {
        limit_address_space(&mut command, memory_limit_mib)?;
    }
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
        let _ = stdout.read_to_end(&mut buffer);
        buffer
    });
    let stderr_reader = std::thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = stderr.read_to_end(&mut buffer);
        buffer
    });

    let started = Instant::now();
    let status = loop {
        if let Some(status) = process
            .try_wait()
            .map_err(|error| anyhow::anyhow!("{command:?} waiting error: {error}"))?
        {
            break status;
        }
        if started.elapsed() > timeout {
            let _ = process.kill();
            let _ = process.wait();
            // The readers are left behind: a surviving worker process may still hold the pipes.
            return Ok(Leg {
                kind: Kind::Timeout,
                ..Leg::default()
            });
        }
        std::thread::sleep(POLL_INTERVAL);
    };
    let _ = writer.join();
    let stdout = stdout_reader.join().expect("Never panics");
    let stderr = String::from_utf8_lossy(stderr_reader.join().expect("Never panics").as_slice())
        .into_owned();

    Ok(classify(status, stdout.as_slice(), stderr.as_str(), target))
}

///
/// Classifies a finished compiler run: ok | error | no-bytecode | panic | abort.
///
fn classify(status: std::process::ExitStatus, stdout: &[u8], stderr: &str, target: &str) -> Leg {
    let exit = exit_code(status);
    let stderr_tail = tail(stderr, STDERR_TAIL);
    let mut leg = Leg {
        exit: Some(exit),
        ..Leg::default()
    };

    if exit < 0 {
        let last_line = stderr_tail.trim().lines().last().unwrap_or_default();
        leg.kind = Kind::Abort;
        leg.signature = Some(format!("signal {}: {}", -exit, signature(last_line)));
        leg.stderr = Some(stderr_tail);
        return leg;
    }
    if stderr.contains("panicked at") {
        leg.kind = Kind::Panic;
        leg.signature = Some(signature(panic_message(stderr).as_str()));
        leg.stderr = Some(stderr_tail);
        return leg;
    }

    let Ok(output) = serde_json::from_slice::<serde_json::Value>(stdout) else {
        let message = if stderr_tail.is_empty() {
            format!("exit {exit} without standard-JSON output")
        } else {
            stderr_tail.clone()
        };
        leg.kind = Kind::Error;
        leg.signature = Some(signature(message.as_str()));
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
            ["message", "formattedMessage"]
                .into_iter()
                .filter_map(|key| error.get(key).and_then(serde_json::Value::as_str))
                .find(|message| !message.is_empty())
                .unwrap_or_default()
                .to_owned()
        })
        .collect();
    if let Some(first) = messages.first() {
        leg.kind = Kind::Error;
        leg.signature = Some(signature(first.as_str()));
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
        return leg;
    }

    leg.kind = Kind::Ok;
    leg
}

///
/// Collapses a diagnostic into a bucket key: first line, paths and numbers normalized.
///
pub fn signature(message: &str) -> String {
    let line = message.trim().lines().next().unwrap_or("<empty>");
    let line = SOL_PATH.replace_all(line, "<file>");
    let line = HEX_NUMBER.replace_all(line.as_ref(), "0x…");
    let line = DECIMAL_NUMBER.replace_all(line.as_ref(), "N");
    line.chars().take(200).collect()
}

///
/// The panic message, which is the bucket key; the location stays in the recorded stderr tail.
///
fn panic_message(stderr: &str) -> String {
    if let Some(captures) = PANIC_WITH_MESSAGE.captures(stderr) {
        let message = captures.get(1).expect("Always exists").as_str();
        return if message.is_empty() {
            "<no panic message>".to_owned()
        } else {
            message.to_owned()
        };
    }
    if let Some(found) = PANIC.find(stderr) {
        return found.as_str().to_owned();
    }
    stderr.trim().lines().last().unwrap_or_default().to_owned()
}

///
/// Whether a standard JSON output contract has non-empty bytecode.
///
fn has_bytecode(contract: &serde_json::Value) -> bool {
    contract
        .pointer("/evm/bytecode/object")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|object| !object.is_empty())
}

///
/// The last `count` characters of `text`.
///
fn tail(text: &str, count: usize) -> String {
    let skip = text.chars().count().saturating_sub(count);
    text.chars().skip(skip).collect()
}

///
/// The exit code, or the negated signal number if the process was killed.
///
fn exit_code(status: std::process::ExitStatus) -> i32 {
    #[cfg(unix)]
    if let Some(signal) = std::os::unix::process::ExitStatusExt::signal(&status) {
        return -signal;
    }
    status.code().unwrap_or(-1)
}

///
/// Caps the compiler's address space so an out-of-memory compilation aborts deterministically
/// instead of taking the machine down.
///
#[cfg(unix)]
fn limit_address_space(command: &mut Command, memory_limit_mib: u64) -> anyhow::Result<()> {
    use std::os::unix::process::CommandExt;

    let limit = (memory_limit_mib * 1024 * 1024) as libc::rlim_t;
    // SAFETY: `setrlimit` is async-signal-safe and touches only the child's own limits.
    unsafe {
        command.pre_exec(move || {
            let rlimit = libc::rlimit {
                rlim_cur: limit,
                rlim_max: limit,
            };
            if libc::setrlimit(libc::RLIMIT_AS, &rlimit) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    Ok(())
}

///
/// Caps the compiler's address space so an out-of-memory compilation aborts deterministically
/// instead of taking the machine down.
///
#[cfg(not(unix))]
fn limit_address_space(_command: &mut Command, _memory_limit_mib: u64) -> anyhow::Result<()> {
    anyhow::bail!("The compiler memory limit is only supported on Unix")
}
