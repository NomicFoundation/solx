//!
//! `solx` Sourcify sweep report.
//!

pub mod census;
pub mod listing;
pub mod report_template;

use std::path::PathBuf;

use askama::Template;

use super::record::Record;

use self::report_template::ReportTemplate;

///
/// Renders the markdown report from sweep results: the outcome table and the failure census,
/// which ranks failures by signature with example contracts and so doubles as the frontend's
/// feature-gap backlog.
///
pub fn report(
    results_path: PathBuf,
    output_path: Option<PathBuf>,
    title: String,
) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(results_path.as_path())
        .map_err(|error| anyhow::anyhow!("Sweep results {results_path:?} reading: {error}"))?;
    let mut meta = serde_json::Map::new();
    let mut records = Vec::new();
    for line in content.lines() {
        let value: serde_json::Value = serde_json::from_str(line)
            .map_err(|error| anyhow::anyhow!("Sweep results {results_path:?} parsing: {error}"))?;
        match value.get("meta").and_then(serde_json::Value::as_object) {
            Some(line_meta) if meta.is_empty() => meta = line_meta.clone(),
            Some(line_meta) if line_meta != &meta => anyhow::bail!(
                "Sweep results {results_path:?} were produced by other compilers: {line_meta:?}, not {meta:?}"
            ),
            Some(_) => {}
            None => records.push(serde_json::from_value::<Record>(value).map_err(|error| {
                anyhow::anyhow!("Sweep results {results_path:?} parsing: {error}")
            })?),
        }
    }
    if records.is_empty() {
        anyhow::bail!("No sweep results found in {results_path:?}");
    }
    // Threads finish in any order; sorting fixes which contracts a bucket lists as examples.
    records.sort_by(|left, right| left.identifier.cmp(&right.identifier));

    let report = ReportTemplate::new(title, &meta, records.as_slice())
        .render()
        .expect("Always renders");
    match output_path {
        Some(output_path) => std::fs::write(output_path.as_path(), report)
            .map_err(|error| anyhow::anyhow!("Writing {output_path:?}: {error}"))?,
        None => println!("{report}"),
    }
    Ok(())
}
