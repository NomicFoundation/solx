//!
//! The benchmark analyzer utilities.
//!

///
/// Formats a percentage with a sign and one decimal.
///
pub fn percent(percentage: f64) -> String {
    format!("{percentage:+.1}%")
}

///
/// Formats an integer with thousands separators.
///
pub fn commas(number: impl Into<u128>) -> String {
    let digits = number.into().to_string();
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
pub fn signed_commas(total: i128) -> String {
    format!(
        "{}{}",
        if total.is_negative() { "-" } else { "+" },
        commas(total.unsigned_abs())
    )
}

///
/// A count with the noun it quantifies, agreeing in number.
///
pub fn count_noun(count: u64, noun: &str) -> String {
    format!(
        "{} {noun}{}",
        commas(count),
        if count == 1 { "" } else { "s" }
    )
}

///
/// The verb form agreeing with a count, for the clause a `count_noun` heads.
///
pub fn agreeing<'a>(count: u64, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
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
/// input averages the two middle elements: at length two the upper-middle
/// would be the maximum, not a median.
///
pub fn median(percentages: &[f64]) -> Option<f64> {
    if percentages.is_empty() {
        return None;
    }
    let mut percentages = percentages.to_vec();
    percentages.sort_unstable_by(f64::total_cmp);
    let middle = percentages.len() / 2;
    Some(if percentages.len().is_multiple_of(2) {
        (percentages[middle - 1] + percentages[middle]) / 2.0
    } else {
        percentages[middle]
    })
}
