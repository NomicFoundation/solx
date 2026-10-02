//!
//! Source code line index.
//!

///
/// Source code line index.
///
/// The byte offset each line of a source starts at, for resolving an offset to its line and column
/// with a binary search. A line ends at `\n`, `\r\n` or a lone `\r`, the terminators Slang's lexer
/// recognizes.
///
pub struct LineIndex {
    /// The offset of each line's first byte, in order.
    line_starts: Vec<usize>,
}

impl LineIndex {
    ///
    /// Builds the line index for the source code.
    ///
    pub fn new(source_code: &str) -> Self {
        let bytes = source_code.as_bytes();
        let line_starts = std::iter::once(0)
            .chain(
                bytes
                    .iter()
                    .enumerate()
                    .filter_map(|(offset, &byte)| match byte {
                        b'\n' => Some(offset + 1),
                        // The `\r` of a `\r\n` belongs to the line its `\n` ends.
                        b'\r' if bytes.get(offset + 1) != Some(&b'\n') => Some(offset + 1),
                        _ => None,
                    }),
            )
            .collect();
        Self { line_starts }
    }

    ///
    /// The 1-based line and byte column of `offset`.
    ///
    #[inline]
    pub fn line_and_column(&self, offset: usize) -> (usize, usize) {
        let line = self.line_starts.partition_point(|&start| start <= offset);
        (line, offset - self.line_starts[line - 1] + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::LineIndex;

    /// Asserts that every byte of the source `lines` make up, the terminators included, resolves
    /// to its line and column, and the end of the source to the column after the last line.
    fn assert_resolves(lines: &[&str]) {
        let source = lines.concat();
        let index = LineIndex::new(source.as_str());
        let mut offset = 0;
        for (line, text) in lines.iter().enumerate() {
            for column in 0..text.len() {
                assert_eq!(
                    index.line_and_column(offset + column),
                    (line + 1, column + 1),
                    "{source:?} at {}",
                    offset + column,
                );
            }
            offset += text.len();
        }
        let last = lines.last().expect("a source has a line");
        assert_eq!(
            index.line_and_column(source.len()),
            (lines.len(), last.len() + 1),
            "{source:?} at its end",
        );
    }

    #[test]
    fn every_byte_resolves_to_its_line_and_column() {
        assert_resolves(&["ab\n", "c\n", ""]);
        assert_resolves(&["ab\r\n", "c\r\n", ""]);
        assert_resolves(&["a\r", "b\r", "\r\n", "c\n", "d"]);
    }
}
