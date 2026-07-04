//!
//! Line-column source code location.
//!

use crate::debug_info::line_index::LineIndex;

///
/// Line-column source code location.
///
/// It can be resolved from a solc AST source code location if the source code is provided.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MappedLocation {
    /// The contract path.
    pub path: String,
    /// The line number.
    pub line: Option<usize>,
    /// The column number.
    pub column: Option<usize>,
    /// The error area length.
    pub length: Option<usize>,
    /// The source code line to print.
    pub source_code_line: Option<String>,
}

impl MappedLocation {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(path: String) -> Self {
        Self {
            path,
            line: None,
            column: None,
            length: None,
            source_code_line: None,
        }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn new_with_location(
        path: String,
        line: usize,
        column: usize,
        length: usize,
        source_code_line: Option<String>,
    ) -> Self {
        Self {
            path,
            line: Some(line),
            column: Some(column),
            length: Some(length),
            source_code_line,
        }
    }

    ///
    /// A shortcut constructor from `solc` AST source location.
    ///
    pub fn from_solc_location(
        path: String,
        start: isize,
        end: isize,
        source_code: Option<&str>,
    ) -> Self {
        match source_code {
            Some(source_code) => LineIndex::new(source_code).mapped_location(path, start, end),
            None => Self::new(path),
        }
    }
}

impl std::fmt::Display for MappedLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut path = self.path.clone();
        if let Some(line) = self.line {
            path.push(':');
            path.push_str(line.to_string().as_str());
            if let Some(column) = self.column {
                path.push(':');
                path.push_str(column.to_string().as_str());
                if let (Some(source_code_line), Some(length)) =
                    (self.source_code_line.as_ref(), self.length)
                {
                    let line_number_length = line.to_string().len();
                    writeln!(f, "{} --> {path}", " ".repeat(line_number_length))?;
                    writeln!(f, " {} |", " ".repeat(line_number_length))?;
                    writeln!(f, " {line} | {source_code_line}")?;
                    writeln!(
                        f,
                        " {} | {} {}",
                        " ".repeat(line_number_length),
                        " ".repeat(column),
                        "^".repeat(std::cmp::min(
                            length,
                            source_code_line.len().saturating_sub(column),
                        ))
                    )?;
                }
            }
        } else {
            writeln!(f, "--> {path}")?;
        }
        Ok(())
    }
}
