//!
//! `solx` Sourcify sweep report.
//!

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::LazyLock;

use regex::Regex;

use super::compilation::normalize;
use super::record::Record;
use super::record::leg::Leg;
use super::record::leg::kind::Kind;
use super::record::outcome::Outcome;

/// The number of failure signatures listed per census.
const TOP_SIGNATURES: usize = 40;

/// The number of example contracts listed per signature.
const EXAMPLES: usize = 3;

/// The number of timed-out contracts listed.
const TIMEOUT_EXAMPLES: usize = 20;

static CRASH_LINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(Assertion `.*?' failed|LLVM ERROR: [^\n]+|fatal error: [^\n]+|Unexpected [^\n]+)")
        .expect("Always valid")
});
static MLIR_ERROR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"error: ([^\n]+)").expect("Always valid"));
static PANIC_LOCATION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"panicked at ([^\n:]+):\d+:\d+").expect("Always valid"));

///
/// Renders the markdown report from sweep results: the outcome table and the failure census,
/// which ranks failures by signature with example contracts and so doubles as the frontend's
/// feature-gap backlog.
///
/// Shard outputs may be passed separately or concatenated.
///
pub fn report(
    results_paths: Vec<PathBuf>,
    output_path: Option<PathBuf>,
    title: String,
) -> anyhow::Result<()> {
    let mut meta = serde_json::Map::new();
    let mut records = Vec::new();
    for path in results_paths.iter() {
        let content = std::fs::read_to_string(path.as_path())
            .map_err(|error| anyhow::anyhow!("Sweep results {path:?} reading: {error}"))?;
        for line in content.lines() {
            let value: serde_json::Value = serde_json::from_str(line)
                .map_err(|error| anyhow::anyhow!("Sweep results {path:?} parsing: {error}"))?;
            match value.get("meta").and_then(serde_json::Value::as_object) {
                Some(line_meta) if meta.is_empty() => meta = line_meta.clone(),
                Some(line_meta) if line_meta != &meta => anyhow::bail!(
                    "Sweep results {path:?} were produced by other compilers: {line_meta:?}, not {meta:?}"
                ),
                Some(_) => {}
                None => {
                    records.push(serde_json::from_value::<Record>(value).map_err(|error| {
                        anyhow::anyhow!("Sweep results {path:?} parsing: {error}")
                    })?)
                }
            }
        }
    }
    if records.is_empty() {
        anyhow::bail!("No sweep results found in {results_paths:?}");
    }
    // Threads finish in any order; sorting fixes which contracts a bucket lists as examples.
    records.sort_by(|left, right| left.identifier.cmp(&right.identifier));

    let report = render(&meta, records.as_slice(), title.as_str());
    match output_path {
        Some(output_path) => std::fs::write(output_path.as_path(), report)
            .map_err(|error| anyhow::anyhow!("Writing {output_path:?}: {error}"))?,
        None => println!("{report}"),
    }
    Ok(())
}

///
/// Renders the report markdown.
///
fn render(
    meta: &serde_json::Map<String, serde_json::Value>,
    records: &[Record],
    title: &str,
) -> String {
    let total = records.len();
    let mut lines = vec![format!("## {title}"), String::new()];
    for name in ["candidate", "reference"] {
        if let Some(version) = meta.get(name).and_then(serde_json::Value::as_str) {
            lines.push(format!("- {name}: `{version}`"));
        }
    }
    lines.push(format!("- contracts: {total}"));
    lines.push(String::new());

    lines.push("| outcome | count | share | meaning |".to_owned());
    lines.push("|---|---|---|---|".to_owned());
    for outcome in Outcome::ALL {
        let count = records
            .iter()
            .filter(|record| record.outcome == outcome)
            .count();
        if count > 0 {
            lines.push(format!(
                "| {outcome} | {count} | {:.2}% | {} |",
                count as f64 / total as f64 * 100.0,
                outcome.meaning()
            ));
        }
    }
    lines.push(String::new());

    let candidate_only: Vec<(&str, &Leg)> = records
        .iter()
        .filter(|record| record.outcome == Outcome::CandidateFail)
        .map(|record| (record.identifier.as_str(), &record.candidate))
        .collect();
    census(
        candidate_only.as_slice(),
        "Candidate-only failures",
        &mut lines,
    );
    // For shared failures the reference's diagnostic is the one that explains the contract.
    let shared: Vec<(&str, &Leg)> = records
        .iter()
        .filter(|record| record.outcome == Outcome::BothFail)
        .map(|record| {
            (
                record.identifier.as_str(),
                record
                    .reference
                    .as_ref()
                    .expect("Always exists for both-fail"),
            )
        })
        .collect();
    census(
        shared.as_slice(),
        "Failures shared with the reference (reference diagnostics)",
        &mut lines,
    );
    let timeouts: Vec<&str> = records
        .iter()
        .filter(|record| record.outcome == Outcome::Timeout)
        .map(|record| record.identifier.as_str())
        .collect();
    if !timeouts.is_empty() {
        lines.push(format!("### Timeouts ({})", timeouts.len()));
        lines.push(String::new());
        lines.push(
            timeouts
                .iter()
                .take(TIMEOUT_EXAMPLES)
                .map(|identifier| format!("`{identifier}`"))
                .collect::<Vec<_>>()
                .join(", "),
        );
        lines.push(String::new());
    }

    lines.join("\n")
}

///
/// Ranks the failures of one leg by kind and signature, with counts and example contracts.
///
fn census(failures: &[(&str, &Leg)], title: &str, lines: &mut Vec<String>) {
    if failures.is_empty() {
        return;
    }
    let mut buckets: BTreeMap<(Kind, String), Vec<&str>> = BTreeMap::new();
    let mut kinds: BTreeMap<Kind, usize> = BTreeMap::new();
    for (identifier, leg) in failures.iter() {
        let (kind, signature) = refine(leg);
        buckets
            .entry((kind, signature))
            .or_default()
            .push(identifier);
        *kinds.entry(kind).or_default() += 1;
    }
    let mut kinds: Vec<(Kind, usize)> = kinds.into_iter().collect();
    kinds.sort_by_key(|(_kind, count)| std::cmp::Reverse(*count));

    lines.push(format!("### {title} ({})", failures.len()));
    lines.push(String::new());
    lines.push(format!(
        "kinds: {}",
        kinds
            .iter()
            .map(|(kind, count)| format!("`{kind}` {count}"))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    lines.push(String::new());
    lines.push("| count | kind | signature | examples |".to_owned());
    lines.push("|---|---|---|---|".to_owned());

    let mut ranked: Vec<((Kind, String), Vec<&str>)> = buckets.into_iter().collect();
    // Stable: equal counts keep the key order of the buckets.
    ranked.sort_by_key(|(_key, identifiers)| std::cmp::Reverse(identifiers.len()));
    for ((kind, signature), identifiers) in ranked.iter().take(TOP_SIGNATURES) {
        lines.push(format!(
            "| {} | {kind} | {} | {} |",
            identifiers.len(),
            signature.replace('|', "\\|"),
            identifiers
                .iter()
                .take(EXAMPLES)
                .map(|identifier| format!("`{identifier}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if ranked.len() > TOP_SIGNATURES {
        let rest: usize = ranked
            .iter()
            .skip(TOP_SIGNATURES)
            .map(|(_key, identifiers)| identifiers.len())
            .sum();
        lines.push(format!(
            "| {rest} | | *{} further signatures* | |",
            ranked.len() - TOP_SIGNATURES
        ));
    }
    lines.push(String::new());
}

///
/// The bucket of one failed leg: sharper than the signature recorded at sweep time, from the
/// recorded diagnostics and stderr tail.
///
fn refine(leg: &Leg) -> (Kind, String) {
    let stderr = leg.stderr.as_deref().unwrap_or_default();
    let recorded = leg.signature.as_deref().unwrap_or_default();
    let signature = match leg.kind {
        Kind::Error => {
            let first = leg.messages.first().map_or(recorded, String::as_str);
            if first.starts_with("Sol pass pipeline failed") {
                let detail = MLIR_ERROR.captures(stderr).map_or(first, |captures| {
                    captures.get(1).expect("Always exists").as_str()
                });
                format!("MLIR verifier: {}", normalize(detail))
            } else if first.contains("worker exited without replying") {
                let detail = CRASH_LINE
                    .find(stderr)
                    .map_or(first, |found| found.as_str());
                format!("worker crash: {}", normalize(detail))
            } else {
                normalize(first)
            }
        }
        Kind::Panic => {
            let location = PANIC_LOCATION
                .captures(stderr)
                .map(|captures| {
                    let path: Vec<&str> = captures
                        .get(1)
                        .expect("Always exists")
                        .as_str()
                        .split('/')
                        .collect();
                    path[path.len().saturating_sub(2)..].join("/")
                })
                .unwrap_or_else(|| "?".to_owned());
            format!("{recorded} @ {location}")
        }
        Kind::Abort => CRASH_LINE
            .find(stderr)
            .map_or_else(|| recorded.to_owned(), |found| normalize(found.as_str())),
        _ => recorded.to_owned(),
    };
    (leg.kind, signature)
}
