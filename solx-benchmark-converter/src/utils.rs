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
/// The relative PR-vs-base percentage, `None` on a zero base — every
/// percentage in the summary comes from here, so zero-base handling cannot
/// drift between columns.
///
pub fn relative_percent(pr: u64, base: u64) -> Option<f64> {
    (base != 0).then(|| (pr as f64 - base as f64) / base as f64 * 100.0)
}

///
/// The median of the given percentages, if any were collected. Even-length
/// input averages the two middle elements — at n=2 the upper-middle would be
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

#[cfg(test)]
mod tests {
    use super::agreeing;
    use super::commas;
    use super::count_noun;
    use super::median;
    use super::signed_commas;

    #[test]
    fn commas_group_thousands() {
        assert_eq!(commas(0u64), "0");
        assert_eq!(commas(42u64), "42");
        assert_eq!(commas(47_660u64), "47,660");
        assert_eq!(commas(101_098u64), "101,098");
    }

    #[test]
    fn signed_commas_keep_the_sign_and_group() {
        assert_eq!(signed_commas(0), "+0");
        assert_eq!(signed_commas(139), "+139");
        assert_eq!(signed_commas(139_432), "+139,432");
        assert_eq!(signed_commas(-22_104), "-22,104");
    }

    #[test]
    fn count_noun_agrees_with_its_count() {
        assert_eq!(count_noun(0, "run"), "0 runs");
        assert_eq!(count_noun(1, "run"), "1 run");
        assert_eq!(count_noun(1_500, "failure"), "1,500 failures");
    }

    #[test]
    fn agreeing_picks_the_form_matching_its_count() {
        assert_eq!(agreeing(0, "differs", "differ"), "differ");
        assert_eq!(agreeing(1, "differs", "differ"), "differs");
        assert_eq!(agreeing(2, "differs", "differ"), "differ");
    }

    #[test]
    fn median_averages_the_two_middles_for_even_input() {
        assert_eq!(median(&[]), None);
        assert_eq!(median(&[3.0]), Some(3.0));
        assert_eq!(median(&[1.0, 3.0]), Some(2.0));
        assert_eq!(median(&[1.0, 2.0, 30.0]), Some(2.0));
    }
}
