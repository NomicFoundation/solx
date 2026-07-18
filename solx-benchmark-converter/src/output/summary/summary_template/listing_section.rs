//!
//! One bulleted listing under a bold heading, behind an "Output changed"
//! verdict.
//!

use crate::output::summary::suite_stats::SuiteStats;
use crate::output::summary::summary_template::truncated::Truncated;

///
/// One bulleted listing under a bold heading, already truncated: a "+N more"
/// pointer is its last bullet.
///
pub struct ListingSection {
    /// The bold heading introducing the listing.
    pub heading: String,
    /// The rendered bullets, the last a "+N more" pointer when truncated.
    pub bullets: Vec<String>,
}

impl ListingSection {
    /// The listings behind an "Output changed" verdict, inline. A bytecode
    /// size change means semantics possibly changed, so it is never folded
    /// away.
    pub fn from_stats(stats: &[SuiteStats]) -> Vec<Self> {
        let mut sections = Vec::new();
        for suite in stats {
            for (title, unit, movers) in [
                ("largest size changes", " B", &suite.top_size_movers),
                ("largest gas changes", "", &suite.top_gas_movers),
            ] {
                if movers.is_empty() {
                    continue;
                }
                let ranked = movers.ranked();
                let truncated = Truncated::new(ranked.as_slice());
                let mut bullets: Vec<String> = truncated
                    .shown
                    .iter()
                    .map(|movement| {
                        format!(
                            "`{}` [{}] {} → {}{unit}{}",
                            movement.label,
                            movement.mode,
                            crate::utils::commas(movement.main),
                            crate::utils::commas(movement.pr),
                            match crate::utils::relative_percent(movement.pr, movement.main) {
                                Some(percentage) => {
                                    format!(" ({})", crate::utils::percent(percentage))
                                }
                                None => String::new(),
                            }
                        )
                    })
                    .collect();
                bullets.extend(truncated.more_bullet(suite.report_file.as_str()));
                sections.push(Self {
                    heading: format!("{} — {title}", suite.label),
                    bullets,
                });
            }
        }
        sections
    }
}
