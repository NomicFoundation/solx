//!
//! `solx` Sourcify sweep: compiles every contract of a Sourcify corpus through the candidate
//! compiler, re-runs its failures other than timeouts with the reference compiler, and records the
//! outcomes.
//!

pub mod compilation;
pub mod config;
pub mod contract;
pub mod corpus;
pub mod record;
pub mod report;

use std::collections::BTreeMap;
use std::io::Write;
use std::num::NonZeroUsize;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use self::config::Config;
use self::contract::Contract;
use self::corpus::Slice;
use self::record::Record;
use self::record::leg::Leg;
use self::record::leg::kind::Kind;
use self::record::outcome::Outcome;

///
/// The compiler invocation limits of a sweep.
///
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// The number of concurrent compiler processes.
    pub threads: NonZeroUsize,
    /// The per-compilation timeout.
    pub timeout: Duration,
    /// The resident memory cap of each compiler with its workers in MiB.
    pub memory_limit_mib: Option<u64>,
}

///
/// Sweeps a slice of the corpus and writes one JSON line per contract to `output_path`, after a
/// first line with both compilers' versions.
///
/// Compilation failures are results, not errors: only harness breakage fails the sweep.
///
pub fn run(
    config: Config,
    corpus_directory: PathBuf,
    corpus_path: Option<PathBuf>,
    output_path: PathBuf,
    solidity_version: String,
    slice: Slice,
    limits: Limits,
) -> anyhow::Result<()> {
    let candidate = config.candidate()?;
    let reference = config.reference()?;
    let candidate_path = crate::utils::absolute_path(candidate.path.as_str())?;
    let reference_path = crate::utils::absolute_path(reference.path.as_str())?;
    let candidate_version = candidate
        .solidity_version
        .as_deref()
        .unwrap_or(solidity_version.as_str());
    let reference_version = reference
        .solidity_version
        .as_deref()
        .unwrap_or(solidity_version.as_str());

    let contracts = match corpus_path {
        Some(corpus_path) => corpus::local(corpus_path.as_path(), slice)?,
        None => corpus::pinned(&config.corpus, corpus_directory.as_path(), slice)?,
    };

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| anyhow::anyhow!("Creating {parent:?}: {error}"))?;
    }
    let mut output = std::fs::File::create(output_path.as_path())
        .map_err(|error| anyhow::anyhow!("Creating {output_path:?}: {error}"))?;
    let meta = serde_json::json!({
        "meta": {
            "slang": version(candidate_path.as_path())?,
            "baseline": version(reference_path.as_path())?,
        }
    });
    writeln!(output, "{meta}")
        .map_err(|error| anyhow::anyhow!("Writing {output_path:?}: {error}"))?;

    eprintln!(
        "{} {} contracts with {} threads",
        solx_utils::cargo_status_ok("Sweeping"),
        contracts.len(),
        limits.threads
    );
    let started = Instant::now();
    let mut counts = BTreeMap::new();
    let next = AtomicUsize::new(0);
    std::thread::scope(|scope| -> anyhow::Result<()> {
        let (sender, receiver) = std::sync::mpsc::channel();
        for _ in 0..limits.threads.get() {
            let sender = sender.clone();
            let next = &next;
            let contracts = contracts.as_slice();
            let candidate = (candidate_path.as_path(), candidate_version);
            let reference = (reference_path.as_path(), reference_version);
            scope.spawn(move || {
                while let Some(path) = contracts.get(next.fetch_add(1, Ordering::Relaxed)) {
                    let record = sweep_one(path.as_path(), candidate, reference, limits);
                    if sender.send(record).is_err() {
                        break;
                    }
                }
            });
        }
        drop(sender);

        for record in receiver {
            let record = record?;
            writeln!(
                output,
                "{}",
                serde_json::to_string(&record).expect("Always valid")
            )
            .map_err(|error| anyhow::anyhow!("Writing {output_path:?}: {error}"))?;
            *counts.entry(record.outcome.to_string()).or_insert(0usize) += 1;
            let total: usize = counts.values().sum();
            if record.outcome != Outcome::Ok || total.is_multiple_of(100) {
                let detail = if record.outcome == Outcome::Ok {
                    String::new()
                } else {
                    format!(
                        " [{}] {}",
                        record.slang.kind,
                        record.slang.signature.as_deref().unwrap_or_default()
                    )
                };
                eprintln!(
                    "[{total}/{}] {}: {}{detail}",
                    contracts.len(),
                    record.identifier,
                    record.outcome
                );
            }
        }
        Ok(())
    })?;

    eprintln!(
        "{} in {}s: {}",
        solx_utils::cargo_status_ok("Finished"),
        started.elapsed().as_secs(),
        counts
            .iter()
            .map(|(outcome, count)| format!("{outcome}={count}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(())
}

///
/// Compiles one corpus contract with the candidate, and with the reference if the candidate
/// fails without timing out.
///
fn sweep_one(
    path: &Path,
    (candidate_path, candidate_version): (&Path, &str),
    (reference_path, reference_version): (&Path, &str),
    limits: Limits,
) -> anyhow::Result<Record> {
    let contract = Contract::try_from(path.to_owned())?;

    let slang = compilation::compile(
        candidate_path,
        contract.standard_json_input(candidate_version),
        contract.target.as_str(),
        limits.timeout,
        limits.memory_limit_mib,
    )?;

    let (baseline, outcome): (Option<Leg>, Outcome) = match slang.kind {
        Kind::Ok => (None, Outcome::Ok),
        Kind::Timeout => (None, Outcome::Timeout),
        _ => {
            let baseline = compilation::compile(
                reference_path,
                contract.standard_json_input(reference_version),
                contract.target.as_str(),
                limits.timeout,
                limits.memory_limit_mib,
            )?;
            let outcome = if baseline.kind == Kind::Ok {
                Outcome::SlangFail
            } else {
                Outcome::BothFail
            };
            (Some(baseline), outcome)
        }
    };

    Ok(Record {
        identifier: path
            .file_stem()
            .expect("Always exists")
            .to_string_lossy()
            .into_owned(),
        target: contract.target,
        slang,
        baseline,
        outcome,
    })
}

///
/// The first line of the compiler's `--version`.
///
fn version(compiler: &Path) -> anyhow::Result<String> {
    let output = std::process::Command::new(compiler)
        .arg("--version")
        .output()
        .map_err(|error| anyhow::anyhow!("{compiler:?} --version: {error}"))?;
    if !output.status.success() {
        anyhow::bail!("{compiler:?} --version failed with {}", output.status);
    }
    Ok(String::from_utf8_lossy(output.stdout.as_slice())
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_owned())
}
