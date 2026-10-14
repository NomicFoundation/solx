//!
//! `solx` Sourcify sweep report.
//!

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::LazyLock;

use regex::Regex;

use super::record::Kind;
use super::record::Leg;
use super::record::Outcome;
use super::record::Record;

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
static LIBRARY_HASH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$[0-9a-f]{64}\$").expect("Always valid"));
static SOL_PATH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"[^\s"'`]+\.sol\b"#).expect("Always valid"));
static HEX_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"0x[0-9a-fA-F]+").expect("Always valid"));
static DECIMAL_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b\d+\b").expect("Always valid"));
static UNRESOLVED_IMPORT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^failed to resolve import (\S+) in (\S+)").expect("Always valid")
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
        for line in content.lines().filter(|line| !line.trim().is_empty()) {
            let value: serde_json::Value = serde_json::from_str(line)
                .map_err(|error| anyhow::anyhow!("Sweep results {path:?} parsing: {error}"))?;
            match value.get("meta").and_then(serde_json::Value::as_object) {
                Some(line_meta) => meta.extend(line_meta.clone()),
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
    // Shards finish in any order; sorting keeps the tie order of the census stable.
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
    for name in ["slang", "baseline"] {
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
                "| {} | {count} | {:.2}% | {} |",
                outcome.as_str(),
                count as f64 / total as f64 * 100.0,
                outcome.meaning()
            ));
        }
    }
    lines.push(String::new());

    let candidate_only: Vec<(&str, &Leg)> = records
        .iter()
        .filter(|record| record.outcome == Outcome::SlangFail)
        .map(|record| (record.identifier.as_str(), &record.slang))
        .collect();
    census(
        candidate_only.as_slice(),
        "Candidate-only failures",
        &mut lines,
    );
    // For shared failures the baseline's diagnostic is the one that explains the contract.
    let shared: Vec<(&str, &Leg)> = records
        .iter()
        .filter(|record| record.outcome == Outcome::BothFail)
        .filter_map(|record| Some((record.identifier.as_str(), record.baseline.as_ref()?)))
        .collect();
    census(
        shared.as_slice(),
        "Failures shared with the baseline (baseline diagnostics)",
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
    let mut kinds: Vec<(Kind, usize)> = Vec::new();
    for (identifier, leg) in failures.iter() {
        let (kind, signature) = refine(leg);
        buckets
            .entry((kind, signature))
            .or_default()
            .push(identifier);
        match kinds.iter_mut().find(|(seen, _count)| *seen == kind) {
            Some((_kind, count)) => *count += 1,
            None => kinds.push((kind, 1)),
        }
    }
    // Stable: equal counts keep first-seen order.
    kinds.sort_by(|(_, left), (_, right)| right.cmp(left));

    lines.push(format!("### {title} ({})", failures.len()));
    lines.push(String::new());
    lines.push(format!(
        "kinds: {}",
        kinds
            .iter()
            .map(|(kind, count)| format!("`{}` {count}", kind.as_str()))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    lines.push(String::new());
    lines.push("| count | kind | signature | examples |".to_owned());
    lines.push("|---|---|---|---|".to_owned());

    let mut ranked: Vec<((Kind, String), Vec<&str>)> = buckets.into_iter().collect();
    ranked.sort_by(
        |(left_key, left_identifiers), (right_key, right_identifiers)| {
            right_identifiers
                .len()
                .cmp(&left_identifiers.len())
                .then_with(|| left_key.0.as_str().cmp(right_key.0.as_str()))
                .then_with(|| left_key.1.cmp(&right_key.1))
        },
    );
    for ((kind, signature), identifiers) in ranked.iter().take(TOP_SIGNATURES) {
        let mut examples = identifiers.clone();
        examples.sort();
        lines.push(format!(
            "| {} | {} | {} | {} |",
            identifiers.len(),
            kind.as_str(),
            signature.replace('|', "\\|"),
            examples
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
            if let Some(import) = leg
                .messages
                .iter()
                .find(|message| message.starts_with("failed to resolve import"))
            {
                import_bucket(import.as_str())
            } else {
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

///
/// The bucket of an unresolved import, by what kind of path failed to resolve.
///
fn import_bucket(message: &str) -> String {
    let Some(captures) = UNRESOLVED_IMPORT.captures(message) else {
        return format!("import resolution: {}", normalize(message));
    };
    let import_path = captures.get(1).expect("Always exists").as_str();
    let importer = captures.get(2).expect("Always exists").as_str();
    if importer.contains("://") || import_path.contains("://") {
        "import resolution: URL-style source path".to_owned()
    } else if import_path.starts_with('.') {
        "import resolution: relative import not found".to_owned()
    } else {
        "import resolution: non-relative import (remapping or exact source name)".to_owned()
    }
}

///
/// Collapses a diagnostic into a bucket key: its first two lines with paths, library hashes and
/// numbers replaced.
///
fn normalize(text: &str) -> String {
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
