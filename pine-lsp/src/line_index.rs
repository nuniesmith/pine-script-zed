use tower_lsp::lsp_types::{Position, Range};

use crate::ast::Span;

/// Pre-computed line-start table for O(log n) byte-offset → Position lookup.
#[derive(Debug, Clone)]
pub struct LineIndex {
    /// Byte offset of the start of each line (index 0 = line 0, etc.).
    line_starts: Vec<usize>,
    /// Total length of the source in bytes.
    len: usize,
}

impl LineIndex {
    /// Build a `LineIndex` from the full source text.
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0usize];
        for (i, b) in source.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Self {
            line_starts,
            len: source.len(),
        }
    }

    /// Convert a byte offset into an LSP `Position` (0-based line & character).
    ///
    /// Offsets beyond the end of the source are clamped to the last position.
    pub fn position(&self, offset: usize) -> Position {
        let offset = offset.min(self.len);

        // Binary search for the line that contains `offset`.
        let line = match self.line_starts.binary_search(&offset) {
            Ok(exact) => exact,                // offset is exactly at a line start
            Err(ins) => ins.saturating_sub(1), // offset is inside this line
        };

        let line_start = self.line_starts[line];
        let col = offset.saturating_sub(line_start);

        Position {
            line: line as u32,
            character: col as u32,
        }
    }

    /// Convert an LSP `Position` back to a byte offset.
    ///
    /// Returns `None` if the position is out of range.
    pub fn offset(&self, pos: Position) -> Option<usize> {
        let line = pos.line as usize;
        if line >= self.line_starts.len() {
            return None;
        }
        let start = self.line_starts[line];
        let offset = start + pos.character as usize;
        if offset > self.len {
            None
        } else {
            Some(offset)
        }
    }

    /// Convert an AST `Span` into an LSP `Range`.
    pub fn range(&self, span: &Span) -> Range {
        Range {
            start: self.position(span.start),
            end: self.position(span.end),
        }
    }

    /// Convenience: build an LSP `Range` from raw byte offsets.
    pub fn range_from_offsets(&self, start: usize, end: usize) -> Range {
        Range {
            start: self.position(start),
            end: self.position(end),
        }
    }

    /// Return the number of lines in the source.
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Return the byte offset where the given line starts, if valid.
    pub fn line_start(&self, line: usize) -> Option<usize> {
        self.line_starts.get(line).copied()
    }

    /// Return the byte range of a full line (not including the trailing newline).
    pub fn line_range(&self, line: usize) -> Option<(usize, usize)> {
        let start = *self.line_starts.get(line)?;
        let end = self
            .line_starts
            .get(line + 1)
            .map(|&s| s.saturating_sub(1)) // exclude the '\n'
            .unwrap_or(self.len);
        Some((start, end))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_source() {
        let idx = LineIndex::new("");
        assert_eq!(idx.line_count(), 1);
        assert_eq!(
            idx.position(0),
            Position {
                line: 0,
                character: 0
            }
        );
    }

    #[test]
    fn single_line() {
        let src = "hello world";
        let idx = LineIndex::new(src);
        assert_eq!(idx.line_count(), 1);
        assert_eq!(
            idx.position(0),
            Position {
                line: 0,
                character: 0
            }
        );
        assert_eq!(
            idx.position(5),
            Position {
                line: 0,
                character: 5
            }
        );
        assert_eq!(
            idx.position(11),
            Position {
                line: 0,
                character: 11
            }
        );
    }

    #[test]
    fn multi_line() {
        let src = "aaa\nbb\nc";
        // line 0: "aaa"  starts at 0
        // line 1: "bb"   starts at 4
        // line 2: "c"    starts at 7
        let idx = LineIndex::new(src);
        assert_eq!(idx.line_count(), 3);

        assert_eq!(
            idx.position(0),
            Position {
                line: 0,
                character: 0
            }
        );
        assert_eq!(
            idx.position(2),
            Position {
                line: 0,
                character: 2
            }
        );
        assert_eq!(
            idx.position(4),
            Position {
                line: 1,
                character: 0
            }
        );
        assert_eq!(
            idx.position(5),
            Position {
                line: 1,
                character: 1
            }
        );
        assert_eq!(
            idx.position(7),
            Position {
                line: 2,
                character: 0
            }
        );
    }

    #[test]
    fn roundtrip() {
        let src = "line one\nline two\nline three\n";
        let idx = LineIndex::new(src);

        for offset in 0..src.len() {
            let pos = idx.position(offset);
            let back = idx.offset(pos).unwrap();
            assert_eq!(back, offset, "roundtrip failed for offset {offset}");
        }
    }

    #[test]
    fn span_to_range() {
        let src = "aaa\nbbbb\ncc";
        let idx = LineIndex::new(src);
        let span = Span::new(4, 8); // "bbbb"
        let range = idx.range(&span);
        assert_eq!(
            range.start,
            Position {
                line: 1,
                character: 0
            }
        );
        assert_eq!(
            range.end,
            Position {
                line: 1,
                character: 4
            }
        );
    }

    #[test]
    fn clamping_beyond_end() {
        let src = "ab";
        let idx = LineIndex::new(src);
        // offset 999 should clamp to end
        let pos = idx.position(999);
        assert_eq!(
            pos,
            Position {
                line: 0,
                character: 2
            }
        );
    }
}
