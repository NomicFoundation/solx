//!
//! The benchmark analyzer utilities.
//!

///
/// Formats a percentage with a sign and one decimal.
///
pub fn percent(pct: f64) -> String {
    format!("{pct:+.1}%")
}

///
/// Formats an integer with thousands separators.
///
pub fn commas(n: impl Into<u128>) -> String {
    let digits = n.into().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    let bytes = digits.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if index > 0 && (bytes.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*byte as char);
    }
    out
}

///
/// Formats a signed total with an explicit sign and thousands separators.
///
pub fn signed_commas(n: i128) -> String {
    format!(
        "{}{}",
        if n.is_negative() { "-" } else { "+" },
        commas(n.unsigned_abs())
    )
}

///
/// A count with the noun it quantifies, agreeing in number.
///
pub fn count_noun(n: u64, noun: &str) -> String {
    format!("{} {noun}{}", commas(n), if n == 1 { "" } else { "s" })
}

///
/// The verb form agreeing with a count, for the clause a `count_noun` heads.
///
pub fn agreeing<'a>(n: u64, singular: &'a str, plural: &'a str) -> &'a str {
    if n == 1 { singular } else { plural }
}

///
/// The relative PR-vs-base percentage, `None` on a zero base. Every
/// percentage in the summary comes from here, so zero-base handling cannot
/// drift between columns.
///
pub fn relative_percent(pr: u64, base: u64) -> Option<f64> {
    (base != 0).then(|| (pr as f64 - base as f64) / base as f64 * 100.0)
}

///
/// The median of the given percentages, if any were collected. Even-length
/// input averages the two middle elements: at n=2 the upper-middle would be
/// the maximum, not a median.
///
pub fn median(pcts: &[f64]) -> Option<f64> {
    if pcts.is_empty() {
        return None;
    }
    let mut pcts = pcts.to_vec();
    pcts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = pcts.len() / 2;
    Some(if pcts.len().is_multiple_of(2) {
        (pcts[mid - 1] + pcts[mid]) / 2.0
    } else {
        pcts[mid]
    })
}
