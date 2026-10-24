//!
//! `solx` Sourcify sweep report failure census.
//!

use std::collections::BTreeMap;
use std::sync::LazyLock;

use regex::Regex;
use solx_benchmark_converter::SummaryTruncated as Truncated;

use crate::test::sourcify::compilation::normalize;
use crate::test::sourcify::record::leg::Leg;
use crate::test::sourcify::record::leg::kind::Kind;

use super::listing::Listing;

/// The number of failure signatures listed per census.
const TOP_SIGNATURES: usize = 40;

/// The number of example contracts listed per signature.
const EXAMPLES: usize = 3;

static CRASH_LINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(Assertion `.*?' failed|LLVM ERROR: [^\n]+|fatal error: [^\n]+|Unexpected [^\n]+)")
        .expect("Always valid")
});
static MLIR_ERROR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"error: ([^\n]+)").expect("Always valid"));
static PANIC_LOCATION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"panicked at ([^\n:]+):\d+:\d+").expect("Always valid"));

///
/// The failures of one leg ranked by kind and signature, with counts and example contracts.
///
pub struct Census {
    /// The section heading.
    pub title: String,
    /// The number of failures.
    pub total: String,
    /// The kinds with their counts, most frequent first.
    pub kinds: String,
    /// The table rows: count, kind, signature, examples.
    pub rows: Vec<Vec<String>>,
    /// The overflow pointer for the signatures past the cap.
    pub more: Option<String>,
}

impl Census {
    ///
    /// Buckets `failures`, or nothing when there are none.
    ///
    pub fn new(title: &str, failures: &[(&str, &Leg)]) -> Option<Self> {
        if failures.is_empty() {
            return None;
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

        let mut ranked: Vec<((Kind, String), Vec<&str>)> = buckets.into_iter().collect();
        // Stable: equal counts keep the key order of the buckets.
        ranked.sort_by_key(|(_key, identifiers)| std::cmp::Reverse(identifiers.len()));
        let truncated = Truncated::with_cap(ranked.as_slice(), TOP_SIGNATURES);
        let rows = truncated
            .shown
            .iter()
            .map(|((kind, signature), identifiers)| {
                vec![
                    identifiers.len().to_string(),
                    kind.to_string(),
                    signature.replace('|', "\\|"),
                    Listing::new(identifiers.as_slice(), EXAMPLES).items,
                ]
            })
            .collect();
        let more = (truncated.extra > 0).then(|| {
            let contracts: usize = ranked
                .iter()
                .skip(TOP_SIGNATURES)
                .map(|(_key, identifiers)| identifiers.len())
                .sum();
            format!(
                "+{} more signatures over {contracts} contracts",
                truncated.extra
            )
        });

        Some(Self {
            title: title.to_owned(),
            total: failures.len().to_string(),
            kinds: kinds
                .iter()
                .map(|(kind, count)| format!("`{kind}` {count}"))
                .collect::<Vec<_>>()
                .join(", "),
            rows,
            more,
        })
    }
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
