//! Stable byte ranges and human-readable coordinates for source diagnostics.
//! Ranges are UTF-8 byte offsets; lines and columns are one-based Unicode scalar positions.

use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpan {
    pub bytes: Range<usize>,
}

impl SourceSpan {
    pub fn new(start: usize, end: usize, source: &str) -> Option<Self> {
        if start > end || end > source.len()
            || !source.is_char_boundary(start)
            || !source.is_char_boundary(end)
        {
            return None;
        }
        Some(Self { bytes: start..end })
    }

    pub fn positions(&self, source: &str) -> Option<(Position, Position)> {
        let span = Self::new(self.bytes.start, self.bytes.end, source)?;
        Some((
            byte_position(source, span.bytes.start)?,
            byte_position(source, span.bytes.end)?,
        ))
    }
}

pub fn byte_position(source: &str, byte: usize) -> Option<Position> {
    if byte > source.len() || !source.is_char_boundary(byte) {
        return None;
    }
    let prefix = &source[..byte];
    let line = prefix.bytes().filter(|value| *value == b'\n').count() + 1;
    let current_line = prefix.rsplit('\n').next().unwrap_or("");
    let column = current_line.trim_end_matches('\r').chars().count() + 1;
    Some(Position { line, column })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_offsets_are_bytes_but_columns_are_characters() {
        let source = "aéЖ\nβeta";
        assert_eq!(
            byte_position(source, 5),
            Some(Position { line: 1, column: 4 })
        );
        assert_eq!(
            byte_position(source, 6),
            Some(Position { line: 2, column: 1 })
        );
        assert!(byte_position(source, 2).is_none());
    }

    #[test]
    fn handles_crlf_and_end_exclusive_ranges() {
        let source = "ab\r\nдва\r\n";
        let start = source.find("два").unwrap();
        let span = SourceSpan::new(start, start + "два".len(), source).unwrap();
        assert_eq!(
            span.positions(source),
            Some((
                Position { line: 2, column: 1 },
                Position { line: 2, column: 4 }
            ))
        );
    }

    #[test]
    fn rejects_out_of_bounds_and_split_codepoints() {
        let source = "é";
        assert!(SourceSpan::new(0, 1, source).is_none());
        assert!(SourceSpan::new(2, 3, source).is_none());
        assert!(SourceSpan::new(2, 1, source).is_none());
    }
}
