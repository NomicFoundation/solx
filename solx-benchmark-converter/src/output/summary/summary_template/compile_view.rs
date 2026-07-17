//!
//! The compile-time table and its threshold verdict lines.
//!

use std::collections::BTreeSet;

use crate::output::summary::compile_aggregate::CompileAggregate;
use crate::output::summary::suite_stats::SuiteStats;
use crate::output::summary::summary_template::truncated::Truncated;

///
/// The compile-time table and its threshold verdict lines; the columns are
/// data-driven, so the header repeats per pipeline.
///
pub struct CompileView {
    pub pipelines: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub conclusion_line: Option<String>,
    pub outliers_line: Option<String>,
}

impl CompileView {
    /// A compile-time move on one project at least this large is surfaced
    /// individually.
    const PROJECT_THRESHOLD_PERCENT: f64 = 15.0;

    /// The compile-time table and its verdict lines, or `None` when no suite
    /// collected compile times at all.
    pub fn from_stats(stats: &[SuiteStats]) -> Option<Self> {
        let with_ct: Vec<&SuiteStats> = stats.iter().filter(|s| !s.compile.is_empty()).collect();
        if with_ct.is_empty() {
            return None;
        }
        let pipelines: Vec<String> = with_ct
            .iter()
            .flat_map(|s| s.compile.keys())
            .map(String::clone)
            .collect::<BTreeSet<String>>()
            .into_iter()
            .collect();

        let mut any_paired = false;
        let mut any_suite_flag = false;
        let mut outlier_entries: Vec<(String, String, f64)> = Vec::new();
        let mut rows = Vec::new();
        for s in &with_ct {
            let mut row = vec![s.suite_cell()];
            for pipeline in pipelines.iter() {
                let paired = s.compile.get(pipeline).and_then(|agg| {
                    crate::utils::relative_percent(agg.pr_total_ms, agg.main_total_ms)
                        .map(|pct| (agg, pct))
                });
                row.push(match paired {
                    Some((agg, pct)) => {
                        any_paired = true;
                        let (cell, flagged) = agg.cell(pct);
                        any_suite_flag |= flagged;
                        cell
                    }
                    None => "—".to_owned(),
                });
                if let Some(agg) = s.compile.get(pipeline) {
                    for (project, pct) in agg.per_project.iter() {
                        if pct.abs() >= Self::PROJECT_THRESHOLD_PERCENT {
                            outlier_entries.push((project.clone(), pipeline.clone(), *pct));
                        }
                    }
                }
            }
            rows.push(row);
        }

        let conclusion_line = if !any_paired {
            Some(
                "_No paired compile-time data — no pipeline was measured on both `main` and the \
                 PR._"
                    .to_owned(),
            )
        } else if outlier_entries.is_empty() && !any_suite_flag {
            Some(format!(
                "_Within noise — no suite ≥ {}%, no project ≥ {}%._",
                CompileAggregate::SUITE_THRESHOLD_PERCENT as u64,
                Self::PROJECT_THRESHOLD_PERCENT as u64
            ))
        } else {
            None
        };
        let outliers_line =
            (!outlier_entries.is_empty()).then(|| Self::outliers_line(&mut outlier_entries));
        Some(Self {
            pipelines,
            rows,
            conclusion_line,
            outliers_line,
        })
    }

    fn outliers_line(outliers: &mut [(String, String, f64)]) -> String {
        outliers.sort_by(|a, b| b.2.abs().partial_cmp(&a.2.abs()).unwrap());
        let siren = if outliers.iter().any(|(_, _, pct)| *pct > 0.0) {
            "⚠️ "
        } else {
            ""
        };
        let truncated = Truncated::new(outliers);
        let shown: Vec<String> = truncated
            .shown
            .iter()
            .map(|(project, pipeline, pct)| {
                format!("`{project}` {pipeline} **{}**", crate::utils::percent(*pct))
            })
            .collect();
        format!(
            "{siren}**Project outliers (≥{}%):** {}{}",
            Self::PROJECT_THRESHOLD_PERCENT as u64,
            shown.join(" · "),
            truncated.more_suffix()
        )
    }
}
