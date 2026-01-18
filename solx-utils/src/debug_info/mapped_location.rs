//!
//! Line-column source code location.
//!

use crate::contract_name::ContractName;

///
/// Line-column source code location.
///
/// It can be resolved from a solc AST source code location if the source code is provided.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MappedLocation {
    /// The contract name.
    pub contract_name: ContractName,
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
    pub fn new(contract_name: ContractName) -> Self {
        Self {
            contract_name,
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
        contract_name: ContractName,
        line: usize,
        column: usize,
        length: usize,
        source_code_line: Option<String>,
    ) -> Self {
        Self {
            contract_name,
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
        contract_name: &ContractName,
        start: Option<isize>,
        end: Option<isize>,
        source_code: Option<&str>,
    ) -> Self {
        let source_code = match source_code {
            Some(source_code) => source_code,
            None => return Self::new(contract_name.to_owned()),
        };
        let start = start.unwrap_or_default();
        let end = end.unwrap_or_default();
        if start <= 0 || end <= 0 {
            return Self::new(contract_name.to_owned());
        }
        let start = start as usize;
        let end = end as usize;

        let mut cursor = 0;
        for (line, source_line) in source_code.lines().enumerate() {
            let cursor_next = cursor + source_line.len() + 1;

            if cursor <= start && start <= cursor_next {
                let column = start - cursor;
                let (line, column) = if column - 1 == source_line.len() {
                    (line + 2, 1)
                } else {
                    (line + 1, start - cursor + 1)
                };
                let length = end - start;
                return Self::new_with_location(
                    contract_name.to_owned(),
                    line,
                    column,
                    length,
                    Some(source_line.to_owned()),
                );
            }

            cursor = cursor_next;
        }

        Self::new(contract_name.to_owned())
    }
}

impl std::fmt::Display for MappedLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut path = self.contract_name.path.clone();
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
                        "^".repeat(std::cmp::min(length, source_code_line.len() - column))
                    )?;
                }
            }
        } else {
            writeln!(f, "--> {path}")?;
        }
        Ok(())
    }
}
