//!
//! `solx` Sourcify sweep report contract listing.
//!

use solx_benchmark_converter::SummaryTruncated as Truncated;

///
/// A capped inline list of contract identifiers with its total.
///
pub struct Listing {
    /// The number of contracts listed, before the cap.
    pub count: String,
    /// The identifiers up to the cap, with the overflow pointer.
    pub items: String,
}

impl Listing {
    ///
    /// Caps `identifiers` at `cap`.
    ///
    pub fn new(identifiers: &[&str], cap: usize) -> Self {
        let truncated = Truncated::with_cap(identifiers, cap);
        Self {
            count: identifiers.len().to_string(),
            items: format!(
                "{}{}",
                truncated
                    .shown
                    .iter()
                    .map(|identifier| format!("`{identifier}`"))
                    .collect::<Vec<_>>()
                    .join(", "),
                truncated.more_suffix()
            ),
        }
    }
}
