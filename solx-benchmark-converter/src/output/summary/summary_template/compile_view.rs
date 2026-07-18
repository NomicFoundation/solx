//!
//! The compile-time table and its threshold verdict lines.
//!

use std::collections::BTreeSet;

use crate::output::summary::compile_aggregate::CompileAggregate;
use crate::output::summary::suite_stats::SuiteStats;
use crate::output::summary::summary_template::truncated::Truncated;
use crate::pipeline::Pipeline;

///
/// The compile-time table and its threshold verdict lines; the columns are
/// data-driven, so the header repeats per pipeline.
///
pub struct CompileView {
    /// The pipeline columns the table repeats its header over.
    pub pipelines: Vec<Pipeline>,
    /// The rendered rows, one per suite, cells in `pipelines` order.
    pub rows: Vec<Vec<String>>,
    /// The within-noise or no-data conclusion line, when one applies.
    pub conclusion_line: Option<String>,
    /// The project-outlier line, when any project crossed the threshold.
    pub outliers_line: Option<String>,
}

impl CompileView {
    /// A compile-time move on one project at least this large is surfaced
    /// individually.
    const PROJECT_THRESHOLD_PERCENT: f64 = 15.0;

    /// The compile-time table and its verdict lines, or `None` when no suite
    /// collected compile times at all.
    pub fn from_stats(stats: &[SuiteStats]) -> Option<Self> {
        let with_compile_times: Vec<&SuiteStats> = stats
            .iter()
            .filter(|suite| !suite.compile.is_empty())
            .collect();
        if with_compile_times.is_empty() {
            return None;
        }
        let pipelines: Vec<Pipeline> = with_compile_times
            .iter()
            .flat_map(|suite| suite.compile.keys())
            .copied()
            .collect::<BTreeSet<Pipeline>>()
            .into_iter()
            .collect();

        let mut any_paired = false;
        let mut any_suite_flag = false;
        let mut outlier_entries: Vec<(String, Pipeline, f64)> = Vec::new();
        let mut rows = Vec::new();
        for suite in &with_compile_times {
            let mut row = vec![suite.suite_cell()];
            for pipeline in pipelines.iter() {
                let paired = suite.compile.get(pipeline).and_then(|aggregate| {
                    crate::utils::relative_percent(aggregate.pr_total_ms, aggregate.main_total_ms)
                        .map(|percentage| (aggregate, percentage))
                });
                row.push(match paired {
                    Some((aggregate, percentage)) => {
                        any_paired = true;
                        let (cell, flagged) = aggregate.cell(percentage);
                        any_suite_flag |= flagged;
                        cell
                    }
                    None => "—".to_owned(),
                });
                if let Some(aggregate) = suite.compile.get(pipeline) {
                    for (project, percentage) in aggregate.per_project.iter() {
                        if percentage.abs() >= Self::PROJECT_THRESHOLD_PERCENT {
                            outlier_entries.push((project.clone(), *pipeline, *percentage));
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

    /// The project-outlier line, worst first and truncated past `MAX_LISTED`,
    /// sirened when any outlier is a slowdown.
    fn outliers_line(outliers: &mut [(String, Pipeline, f64)]) -> String {
        outliers.sort_by(|left, right| right.2.abs().total_cmp(&left.2.abs()));
        let siren = if outliers.iter().any(|(_, _, percentage)| *percentage > 0.0) {
            "⚠️ "
        } else {
            ""
        };
        let truncated = Truncated::new(outliers);
        let shown: Vec<String> = truncated
            .shown
            .iter()
            .map(|(project, pipeline, percentage)| {
                format!(
                    "`{project}` {pipeline} **{}**",
                    crate::utils::percent(*percentage)
                )
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
