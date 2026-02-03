//!
//! The lexical token location.
//!

///
/// The token location in the source code file.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy, Eq)]
pub struct Location {
    /// The line number, starting from 1.
    pub line: usize,
    /// The column number, starting from 1.
    pub column: usize,
}

impl Default for Location {
    fn default() -> Self {
        Self { line: 1, column: 1 }
    }
}

impl Location {
    ///
    /// Creates a default location.
    ///
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    ///
    /// Mutates the location by shifting the original one down by `lines` and
    /// setting the column to `column`.
    ///
    pub fn shift_down(&mut self, lines: usize, column: usize) {
        if lines == 0 {
            self.shift_right(column);
            return;
        }

        self.line += lines;
        self.column = column;
    }

    ///
    /// Mutates the location by shifting the original one rightward by `columns`.
    ///
    pub fn shift_right(&mut self, columns: usize) {
        self.column += columns;
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        self.line == other.line && self.column == other.column
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}
