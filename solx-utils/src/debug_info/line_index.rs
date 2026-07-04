//!
//! Source code line index.
//!

use crate::debug_info::mapped_location::MappedLocation;

///
/// Source code line index.
///
/// Maps each source line to its starting byte offset, so that a `solc` byte offset can be resolved
/// to a line and column with a binary search instead of a scan from the beginning of the file.
///
pub struct LineIndex<'a> {
    /// Byte offset and text of each source line.
    lines: Vec<(usize, &'a str)>,
}

impl<'a> LineIndex<'a> {
    ///
    /// Builds the line index for the source code.
    ///
    pub fn new(source_code: &'a str) -> Self {
        let lines = source_code
            .lines()
            .scan(0usize, |cursor, line| {
                let start = *cursor;
                *cursor += line.len() + 1;
                Some((start, line))
            })
            .collect();
        Self { lines }
    }

    ///
    /// Resolves a `solc` byte offset range to a line-column location.
    ///
    pub fn mapped_location(&self, path: String, start: isize, end: isize) -> MappedLocation {
        if start < 0 || end < 0 || end < start {
            return MappedLocation::new(path);
        }
        let start = start as usize;
        let end = end as usize;

        let line = self
            .lines
            .partition_point(|&(offset, _)| offset < start)
            .saturating_sub(1);
        let &(cursor, source_line) = match self.lines.get(line) {
            Some(entry) => entry,
            None => return MappedLocation::new(path),
        };
        if start > cursor + source_line.len() + 1 {
            return MappedLocation::new(path);
        }

        let column = start - cursor;
        let (line, column) = if column == source_line.len() + 1 {
            (line + 2, 1)
        } else {
            (line + 1, column + 1)
        };
        let length = end - start;
        MappedLocation::new_with_location(path, line, column, length, Some(source_line.to_owned()))
    }
}
