//!
//! `solx` Sourcify sweep: compiles every contract of a Sourcify corpus through the candidate
//! compiler, re-runs its failures other than timeouts with the reference compiler, and records the
//! outcomes.
//!

pub mod compilation;
pub mod config;
pub mod contract;
pub mod corpus;
pub mod limits;
pub mod record;
pub mod report;

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;

use crate::test::config::compiler::Compiler;

use self::config::Config;
use self::contract::Contract;
use self::corpus::slice::Slice;
use self::limits::Limits;
use self::record::Record;
use self::record::leg::Leg;
use self::record::leg::kind::Kind;
use self::record::outcome::Outcome;

///
/// Sweeps a slice of the corpus and writes one JSON line per contract to `output_path`, after a
/// first line with both compilers' names and versions.
///
pub fn run(
    config: Config,
    corpus_directory: PathBuf,
    local_corpus_directory: Option<PathBuf>,
    output_path: PathBuf,
    solidity_version: String,
    slice: Slice,
    limits: Limits,
) -> anyhow::Result<()> {
    let candidate = Compiler::candidate(&config.compilers, "Sourcify sweep")?;
    let reference = Compiler::reference(&config.compilers, "Sourcify sweep")?;
    let candidate_path = crate::utils::absolute_path(candidate.path.as_str())?;
    let reference_path = crate::utils::absolute_path(reference.path.as_str())?;

    let contracts = match local_corpus_directory {
        Some(directory) => corpus::local(directory.as_path(), &slice)?,
        None => corpus::pinned(&config.corpus, corpus_directory.as_path(), &slice)?,
    };

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| anyhow::anyhow!("Creating {parent:?}: {error}"))?;
    }
    let mut output = std::fs::File::create(output_path.as_path())
        .map_err(|error| anyhow::anyhow!("Creating {output_path:?}: {error}"))?;
    let meta = serde_json::json!({
        "meta": {
            "candidate": version(candidate, candidate_path.as_path())?,
            "reference": version(reference, reference_path.as_path())?,
        }
    });
    writeln!(output, "{meta}")
        .map_err(|error| anyhow::anyhow!("Writing {output_path:?}: {error}"))?;

    let mut thread_pool_builder = rayon::ThreadPoolBuilder::new();
    if let Some(threads) = limits.threads {
        thread_pool_builder = thread_pool_builder.num_threads(threads);
    }
    let pool = thread_pool_builder
        .build()
        .map_err(|error| anyhow::anyhow!("Thread pool building: {error}"))?;
    eprintln!(
        "{} {} contracts with {} threads",
        solx_utils::cargo_status_ok("Sweeping"),
        contracts.len(),
        pool.current_num_threads()
    );
    let started = Instant::now();
    let mut counts = BTreeMap::new();
    let (sender, receiver) = std::sync::mpsc::channel();
    pool.in_place_scope(|scope| -> anyhow::Result<()> {
        let contracts = contracts.as_slice();
        let candidate_path = candidate_path.as_path();
        let reference_path = reference_path.as_path();
        let solidity_version = solidity_version.as_str();
        // A send fails only once the receiver has stopped on an error, which ends the sweep.
        scope.spawn(move |_scope| {
            contracts
                .par_iter()
                .try_for_each_with(sender, |sender, path| {
                    sender
                        .send(sweep_one(
                            path.as_path(),
                            candidate_path,
                            reference_path,
                            solidity_version,
                            limits,
                        ))
                        .map_err(|_disconnected| ())
                })
                .ok();
        });

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
            if record.outcome != Outcome::Ok {
                eprintln!(
                    "{} [{total}/{}] {}: {} [{}] {}",
                    solx_utils::cargo_status_error("Failed"),
                    contracts.len(),
                    record.identifier,
                    record.outcome,
                    record.candidate.kind,
                    record.candidate.signature.as_deref().unwrap_or_default()
                );
            } else if total.is_multiple_of(100) {
                eprintln!(
                    "{} [{total}/{}]",
                    solx_utils::cargo_status_ok("Sweeping"),
                    contracts.len()
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
/// The reference is skipped after a timeout, which would otherwise cost the slowest contracts
/// twice over for an outcome the record already names.
///
fn sweep_one(
    path: &Path,
    candidate_path: &Path,
    reference_path: &Path,
    solidity_version: &str,
    limits: Limits,
) -> anyhow::Result<Record> {
    let contract = Contract::try_from(path.to_owned())?;
    let input = contract.standard_json_input(solidity_version);

    let candidate = compilation::compile(
        candidate_path,
        input.clone(),
        contract.target.as_str(),
        limits.timeout,
        limits.memory_limit_mib,
    )?;

    let (reference, outcome): (Option<Leg>, Outcome) = match candidate.kind {
        Kind::Ok => (None, Outcome::Ok),
        Kind::Timeout => (None, Outcome::Timeout),
        _ => {
            let reference = compilation::compile(
                reference_path,
                input,
                contract.target.as_str(),
                limits.timeout,
                limits.memory_limit_mib,
            )?;
            let outcome = if reference.kind == Kind::Ok {
                Outcome::CandidateFail
            } else {
                Outcome::BothFail
            };
            (Some(reference), outcome)
        }
    };

    Ok(Record {
        identifier: path
            .file_stem()
            .expect("Always exists")
            .to_string_lossy()
            .into_owned(),
        target: contract.target,
        candidate,
        reference,
        outcome,
    })
}

///
/// The compiler's display name with the first line of its `--version`.
///
fn version(compiler: &Compiler, path: &Path) -> anyhow::Result<String> {
    let mut command = Command::new(path);
    command.arg("--version");
    let output = crate::utils::command_with_output(
        &mut command,
        format!(
            "{} compiler {}",
            solx_utils::cargo_status_ok("Identifying"),
            compiler.name
        )
        .as_str(),
    )?;
    Ok(format!(
        "{} ({})",
        compiler.name,
        output.lines().next().unwrap_or_default().trim()
    ))
}
