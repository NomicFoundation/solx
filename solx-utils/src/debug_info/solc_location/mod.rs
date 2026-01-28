//!
//! solc AST source code location.
//!

pub mod ordering;

use self::ordering::Ordering;

///
/// solc AST source code location.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SolcLocation {
    /// Source file identifier.
    pub source_id: usize,
    /// Start location.
    pub start: isize,
    /// End location.
    pub end: isize,
}

impl SolcLocation {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(source_id: usize) -> Self {
        Self::new_with_offsets(source_id, -1, -1)
    }

    ///
    /// A shortcut constructor.
    ///
    /// Please note that `start` and `end` are not line and column,
    /// but absolute char offsets in the source code file.
    ///
    pub fn new_with_offsets(source_id: usize, start: isize, end: isize) -> Self {
        Self {
            source_id,
            start,
            end,
        }
    }

    ///
    /// Parses a `solc` source location string with given ordering.
    ///
    pub fn parse<S>(string: S, ordering: Ordering) -> anyhow::Result<Self>
    where
        S: AsRef<str>,
    {
        let parts: Vec<&str> = string.as_ref().splitn(3, ':').collect();
        match ordering {
            Ordering::Ast => {
                let start = parts[0].parse::<isize>()?;
                let length = parts[1].parse::<isize>()?;
                let source_id = parts[2].parse::<usize>()?;
                Ok(Self::new_with_offsets(source_id, start, start + length))
            }
            Ordering::Yul => {
                let source_id = parts[0].parse::<usize>()?;
                let start = parts[1].parse::<isize>()?;
                let end = parts[2].parse::<isize>()?;
                Ok(Self::new_with_offsets(source_id, start, end))
            }
        }
    }
}
