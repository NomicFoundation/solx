//!
//! The semantic test instance.
//!

pub mod function_call;
pub mod params;

use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use regex::Regex;

use self::function_call::FunctionCall;
use self::params::Params;

///
/// The semantic test instance.
///
#[derive(Debug, PartialEq, Eq)]
pub struct Test {
    /// The source code files.
    pub sources: Vec<(String, String)>,
    /// The test params.
    pub params: Params,
    /// The function calls.
    pub calls: Vec<FunctionCall>,
}

impl TryFrom<&Path> for Test {
    type Error = anyhow::Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let data = std::fs::read_to_string(path)
            .map_err(|error| anyhow::anyhow!("Failed to read test file {path:?}: {error}"))?;

        let comment_start = "// ".to_owned();

        let sources = process_sources(&data, path)?;

        let (data, function_calls) = data
            .split_once(&format!("{comment_start}----{}", crate::NEW_LINE))
            .ok_or_else(|| anyhow::anyhow!("Invalid test format"))?;

        let params = data
            .split_once(&format!("{comment_start}===={}", crate::NEW_LINE))
            .map(|parts| parts.1)
            .unwrap_or_default();

        let params = params
            .lines()
            .filter_map(|line| line.strip_prefix(&comment_start))
            .map(|line| {
                let mut line = line.to_owned();
                line.push_str(crate::NEW_LINE);
                line
            })
            .collect::<Vec<String>>()
            .join("");

        let params = Params::try_from(params.as_str())
            .map_err(|error| anyhow::anyhow!("Failed to parse params: {error}"))?;

        let function_calls = function_calls
            .lines()
            .filter_map(|line| line.strip_prefix(&comment_start))
            .map(|line| {
                let mut line = line.to_owned();
                line.push_str(crate::NEW_LINE);
                line
            })
            .collect::<Vec<String>>()
            .join("");

        let calls = FunctionCall::parse_calls(function_calls.as_str())
            .map_err(|error| anyhow::anyhow!("Failed to parse function calls: {error}"))?;

        Ok(Self {
            sources,
            params,
            calls,
        })
    }
}

///
/// Returns sources.
///
fn process_sources(data: &str, path: &Path) -> anyhow::Result<Vec<(String, String)>> {
    let mut sources = Vec::new();

    let mut source_name = None;
    let mut source = String::new();

    let line_regex = Regex::new(r"^==== (.*): (.*) ====$").expect("regex is compile-time constant");
    let path_regex = Regex::new("^([^=]*)=(.*)$").expect("regex is compile-time constant");

    for (index, line) in data.lines().enumerate() {
        let captures = match line_regex.captures(line) {
            Some(captures) => captures,
            None => {
                source.push_str(line);
                source.push_str(crate::NEW_LINE);
                continue;
            }
        };

        match captures
            .get(1)
            .expect("capture group 1 always present in matched pattern")
            .as_str()
        {
            "ExternalSource" => {
                let data = captures
                    .get(2)
                    .expect("capture group 2 always present in matched pattern")
                    .as_str();
                let (name, relative_path) = match path_regex.captures(data) {
                    Some(captures) => (
                        captures
                            .get(1)
                            .expect("capture group 1 always present in matched pattern")
                            .as_str()
                            .to_owned(),
                        PathBuf::from(
                            captures
                                .get(2)
                                .expect("capture group 2 always present in matched pattern")
                                .as_str(),
                        ),
                    ),
                    None => (data.to_owned(), PathBuf::from(data)),
                };

                let path = path
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("Failed to get parent directory of file"))?
                    .join(relative_path);

                let mut file = fs::File::open(path)?;

                let mut data = String::new();
                file.read_to_string(&mut data)
                    .map_err(|error| anyhow::anyhow!("Failed to read source code file: {error}"))?;

                sources.push((name.replace(std::path::MAIN_SEPARATOR, "/"), data));
            }
            "Source" => {
                let name = captures
                    .get(2)
                    .expect("capture group 2 always present in matched pattern")
                    .as_str()
                    .to_owned();

                if let Some(source_name) = source_name {
                    sources.push((source_name, source));
                }
                // Discard preamble content (comments before any named source)
                source = String::new();

                source_name = Some(name);
            }
            word => anyhow::bail!(
                "Expected \"Source\" or \"ExternalSource\" on line {}, found: {word}",
                index + 1
            ),
        }
    }

    if !source.is_empty() {
        let name = match source_name {
            Some(source_name) => source_name,
            None => path
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "/"),
        };
        sources.push((name, source));
    }

    Ok(sources)
}
