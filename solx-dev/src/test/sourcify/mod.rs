//!
//! `solx` Sourcify sweep: compiles every contract of a Sourcify corpus through the candidate
//! compiler, re-runs its failures with the reference compiler, and records the outcomes.
//!

pub mod compilation;
pub mod config;
pub mod corpus;
pub mod record;
pub mod report;

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use regex::Regex;

use self::config::Config;
use self::corpus::Slice;
use self::record::Kind;
use self::record::Leg;
use self::record::Outcome;
use self::record::Record;

/// Settings passed through verbatim: dropping one would turn a real frontend gap into a harness
/// artifact, and the same input must stay valid for the reference compiler.
const PASSTHROUGH_SETTINGS: [&str; 4] = ["evmVersion", "libraries", "remappings", "viaIR"];

/// Version-expression characters only, so `pragma solidity` in a comment never swallows code up
/// to the next `;`.
static VERSION_PRAGMA: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"pragma\s+solidity\s+[0-9xX*.^~<>=|\s-]+;").expect("Always valid")
});

///
/// The compiler invocation limits of a sweep.
///
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// The number of concurrent compiler processes.
    pub threads: usize,
    /// The per-compilation timeout.
    pub timeout: Duration,
    /// The address-space cap of each compiler process in MiB, or 0 for none.
    pub memory_limit_mib: u64,
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
    let candidate_pragma = format!(
        "pragma solidity >={};",
        candidate
            .solidity_version
            .as_deref()
            .unwrap_or(solidity_version.as_str())
    );
    let reference_pragma = format!(
        "pragma solidity >={};",
        reference
            .solidity_version
            .as_deref()
            .unwrap_or(solidity_version.as_str())
    );

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
        for _ in 0..limits.threads {
            let sender = sender.clone();
            let next = &next;
            let contracts = contracts.as_slice();
            let candidate = (candidate_path.as_path(), candidate_pragma.as_str());
            let reference = (reference_path.as_path(), reference_pragma.as_str());
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
            *counts.entry(record.outcome.as_str()).or_insert(0usize) += 1;
            let total: usize = counts.values().sum();
            if record.outcome != Outcome::Ok || total.is_multiple_of(100) {
                let detail = if record.outcome == Outcome::Ok {
                    String::new()
                } else {
                    format!(
                        " [{}] {}",
                        record.slang.kind.as_str(),
                        record.slang.signature.as_deref().unwrap_or_default()
                    )
                };
                eprintln!(
                    "[{total}/{}] {}: {}{detail}",
                    contracts.len(),
                    record.identifier,
                    record.outcome.as_str()
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
/// fails.
///
fn sweep_one(
    path: &Path,
    (candidate_path, candidate_pragma): (&Path, &str),
    (reference_path, reference_pragma): (&Path, &str),
    limits: Limits,
) -> anyhow::Result<Record> {
    let content = std::fs::read_to_string(path)
        .map_err(|error| anyhow::anyhow!("Corpus record {path:?} reading: {error}"))?;
    let corpus_record: serde_json::Value = serde_json::from_str(content.as_str())
        .map_err(|error| anyhow::anyhow!("Corpus record {path:?} parsing: {error}"))?;
    let target = corpus_record
        .get("target")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("Corpus record {path:?} has no target"))?;

    let slang = compilation::compile(
        candidate_path,
        standard_json_input(&corpus_record, candidate_pragma, path)?,
        target,
        limits.timeout,
        limits.memory_limit_mib,
    )?;
    let (baseline, outcome): (Option<Leg>, Outcome) = match slang.kind {
        Kind::Ok => (None, Outcome::Ok),
        Kind::Timeout => (None, Outcome::Timeout),
        _ => {
            let baseline = compilation::compile(
                reference_path,
                standard_json_input(&corpus_record, reference_pragma, path)?,
                target,
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
        chain_id: corpus_record.get("chain_id").cloned().unwrap_or_default(),
        target: target.to_owned(),
        slang,
        baseline,
        outcome,
    })
}

///
/// The standard JSON input for a corpus record: all sources inline under their original paths,
/// the optimizer at the compiler default, bytecode output only, and every version pragma replaced
/// by `pragma`.
///
fn standard_json_input(
    corpus_record: &serde_json::Value,
    pragma: &str,
    path: &Path,
) -> anyhow::Result<Vec<u8>> {
    let sources: serde_json::Map<String, serde_json::Value> = corpus_record
        .get("sources")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("Corpus record {path:?} has no sources"))?
        .iter()
        .map(|(source_path, content)| {
            let content = content.as_str().unwrap_or_default();
            let content = VERSION_PRAGMA.replace_all(content, regex::NoExpand(pragma));
            (
                source_path.to_owned(),
                serde_json::json!({ "content": content }),
            )
        })
        .collect();
    let mut settings: serde_json::Map<String, serde_json::Value> = corpus_record
        .get("settings")
        .and_then(serde_json::Value::as_object)
        .into_iter()
        .flatten()
        .filter(|(key, value)| {
            PASSTHROUGH_SETTINGS.contains(&key.as_str())
                && !value.is_null()
                && value.as_array().is_none_or(|array| !array.is_empty())
                && value.as_object().is_none_or(|object| !object.is_empty())
        })
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect();
    settings.insert(
        "outputSelection".to_owned(),
        serde_json::json!({ "*": { "*": ["evm.bytecode.object", "evm.deployedBytecode.object"] } }),
    );
    let input = serde_json::json!({
        "language": "Solidity",
        "sources": sources,
        "settings": settings,
    });
    Ok(serde_json::to_vec(&input).expect("Always valid"))
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
