//!
//! A listing capped at a fixed length, with a "+N more" overflow pointer.
//!

///
/// A listing capped at `MAX_LISTED`: what to show, and how many the cap left
/// out. The cap, the arithmetic, and the overflow wording live here, so the
/// listings cannot drift into several spellings of one rule.
///
pub struct Truncated<'a, T> {
    pub shown: &'a [T],
    pub extra: usize,
}

impl<'a, T> Truncated<'a, T> {
    /// Cap on individually-listed items, such as outliers, movers, and new
    /// failures, before the listing folds the rest into a "+N more" pointer.
    pub const MAX_LISTED: usize = 5;

    /// Caps a listing at `MAX_LISTED`, keeping the leading items and counting
    /// the rest as overflow. Callers that want the surviving items to be the
    /// most significant rank the slice before passing it.
    pub fn new(items: &'a [T]) -> Self {
        Self {
            shown: &items[..items.len().min(Self::MAX_LISTED)],
            extra: items.len().saturating_sub(Self::MAX_LISTED),
        }
    }

    /// The bullet closing a truncated listing, pointing at the full report.
    pub fn more_bullet(&self, report_file: &str) -> Option<String> {
        (self.extra > 0).then(|| format!("+{} more — see {report_file}", self.extra))
    }

    /// The suffix closing a truncated inline list.
    pub fn more_suffix(&self) -> String {
        match self.extra {
            0 => String::new(),
            extra => format!(" (+{extra} more)"),
        }
    }
}
