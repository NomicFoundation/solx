//!
//! Tests for the benchmark analyzer utilities.
//!

use crate::utils::agreeing;
use crate::utils::commas;
use crate::utils::count_noun;
use crate::utils::median;
use crate::utils::signed_commas;

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
