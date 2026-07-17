//!
//! One bulleted listing under a bold heading, behind an "Output changed"
//! verdict.
//!

use crate::output::summary::suite_stats::SuiteStats;
use crate::output::summary::summary_template::truncated::Truncated;
use crate::utils::commas;
use crate::utils::percent;
use crate::utils::relative_percent;

///
/// One bulleted listing under a bold heading, already truncated: a "+N more"
/// pointer is its last bullet.
///
pub struct ListingSection {
    pub heading: String,
    pub bullets: Vec<String>,
}

impl ListingSection {
    /// The listings behind an "Output changed" verdict, inline — a bytecode
    /// size change means semantics possibly changed, so it is never folded
    /// away.
    pub fn from_stats(stats: &[SuiteStats]) -> Vec<Self> {
        let mut sections = Vec::new();
        for s in stats {
            for (title, unit, movers) in [
                ("largest size changes", " B", &s.top_size_movers),
                ("largest gas changes", "", &s.top_gas_movers),
            ] {
                if movers.is_empty() {
                    continue;
                }
                let ranked = movers.ranked();
                let truncated = Truncated::new(ranked.as_slice());
                let mut bullets: Vec<String> = truncated
                    .shown
                    .iter()
                    .map(|m| {
                        let pct = match relative_percent(m.pr, m.main) {
                            Some(pct) => format!(" ({})", percent(pct)),
                            None => String::new(),
                        };
                        format!(
                            "`{}` [{}] {} → {}{unit}{pct}",
                            m.label,
                            m.mode,
                            commas(m.main),
                            commas(m.pr)
                        )
                    })
                    .collect();
                bullets.extend(truncated.more_bullet(s.report_file.as_str()));
                sections.push(Self {
                    heading: format!("{} — {title}", s.label),
                    bullets,
                });
            }
        }
        sections
    }
}
