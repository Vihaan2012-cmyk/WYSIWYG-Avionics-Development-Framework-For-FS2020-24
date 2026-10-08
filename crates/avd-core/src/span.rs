//! Byte spans, source files and UTF-8 ↔ UTF-16 line/column mapping.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Half-open byte range `[start, end)` into UTF-8 source text.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub const fn new(start: u32, end: u32) -> Self {
        debug_assert!(start <= end);
        Self { start, end }
    }

    pub const fn empty(at: u32) -> Self {
        Self { start: at, end: at }
    }

    pub const fn len(self) -> u32 {
        self.end - self.start
    }

    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// True if `offset` lies inside the span (`start <= offset < end`).
    pub const fn contains(self, offset: u32) -> bool {
        self.start <= offset && offset < self.end
    }

    /// True if `other` lies entirely within `self`.
    pub const fn contains_span(self, other: Span) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    /// True if the spans share at least one byte.
    pub const fn overlaps(self, other: Span) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Smallest span covering both.
    pub fn merge(self, other: Span) -> Span {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }

    /// Moves the span by `delta` bytes (saturating at 0).
    pub fn shift(self, delta: i64) -> Span {
        let mv = |v: u32| (i64::from(v) + delta).clamp(0, i64::from(u32::MAX)) as u32;
        Span::new(mv(self.start), mv(self.end))
    }

    pub fn range(self) -> std::ops::Range<usize> {
        self.start as usize..self.end as usize
    }

    /// Returns the text covered by the span. Panics if the span is out of bounds or not on
    /// char boundaries, which would indicate a bug in the producer of the span.
    pub fn slice(self, src: &str) -> &str {
        &src[self.range()]
    }
}

/// 1-based line and 1-based column counted in UTF-16 code units (Monaco/LSP convention).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct Pos {
    pub line: u32,
    pub col: u32,
}

/// Precomputed line start offsets for fast offset ↔ position conversion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineIndex {
    line_starts: Vec<u32>,
    len: u32,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut line_starts = vec![0u32];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push((i + 1) as u32);
            }
        }
        Self {
            line_starts,
            len: text.len() as u32,
        }
    }

    pub fn line_count(&self) -> u32 {
        self.line_starts.len() as u32
    }

    /// 0-based line containing `offset` (clamped to the text length).
    pub fn line_of(&self, offset: u32) -> u32 {
        let offset = offset.min(self.len);
        match self.line_starts.binary_search(&offset) {
            Ok(line) => line as u32,
            Err(next) => (next - 1) as u32,
        }
    }

    /// Byte offset of the start of 0-based `line`, or the text length if out of range.
    pub fn line_start(&self, line0: u32) -> u32 {
        self.line_starts
            .get(line0 as usize)
            .copied()
            .unwrap_or(self.len)
    }

    /// Byte offset of the end of 0-based `line`, excluding the `\n` and any `\r` before it.
    pub fn line_end(&self, text: &str, line0: u32) -> u32 {
        let start = self.line_start(line0) as usize;
        let next = self
            .line_starts
            .get(line0 as usize + 1)
            .map_or(self.len as usize, |&n| n as usize);
        let mut end = next;
        let bytes = text.as_bytes();
        if end > start && bytes.get(end - 1) == Some(&b'\n') {
            end -= 1;
        }
        if end > start && bytes.get(end - 1) == Some(&b'\r') {
            end -= 1;
        }
        end as u32
    }

    /// Position of `offset`, clamped to the text and snapped back to a char boundary.
    pub fn pos(&self, text: &str, offset: u32) -> Pos {
        let mut offset = offset.min(self.len) as usize;
        while offset > 0 && !text.is_char_boundary(offset) {
            offset -= 1;
        }
        let line0 = self.line_of(offset as u32);
        let start = self.line_start(line0) as usize;
        let col16: usize = text[start..offset].chars().map(char::len_utf16).sum();
        Pos {
            line: line0 + 1,
            col: col16 as u32 + 1,
        }
    }

    /// Byte offset for a 1-based line and 1-based UTF-16 column.
    /// A column past the end of the line returns the line end; a line out of range or a column
    /// inside a surrogate pair returns `None`.
    pub fn offset(&self, text: &str, pos: Pos) -> Option<u32> {
        if pos.line == 0 || pos.col == 0 || pos.line > self.line_count() {
            return None;
        }
        let line0 = pos.line - 1;
        let start = self.line_start(line0) as usize;
        let end = self.line_end(text, line0) as usize;
        let mut remaining = (pos.col - 1) as usize;
        for (i, ch) in text[start..end].char_indices() {
            if remaining == 0 {
                return Some((start + i) as u32);
            }
            let w = ch.len_utf16();
            if remaining < w {
                return None;
            }
            remaining -= w;
        }
        Some(end as u32)
    }
}

/// An in-memory source file: project-relative path (forward slashes), text, line index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFile {
    pub path: String,
    pub text: String,
    pub index: LineIndex,
}

impl SourceFile {
    pub fn new(path: impl Into<String>, text: impl Into<String>) -> Self {
        let text = text.into();
        let index = LineIndex::new(&text);
        Self {
            path: path.into(),
            text,
            index,
        }
    }

    pub fn pos(&self, offset: u32) -> Pos {
        self.index.pos(&self.text, offset)
    }

    /// Text of 1-based `line` without its line terminator.
    pub fn line_text(&self, line1: u32) -> &str {
        if line1 == 0 || line1 > self.index.line_count() {
            return "";
        }
        let line0 = line1 - 1;
        let start = self.index.line_start(line0) as usize;
        let end = self.index.line_end(&self.text, line0) as usize;
        &self.text[start..end]
    }

    /// Replaces the text and rebuilds the line index.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.index = LineIndex::new(&self.text);
    }
}

/// Converts a path to a forward-slash string, as used in diagnostics and project files.
pub fn normalize_path(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_basics() {
        let s = Span::new(2, 5);
        assert_eq!(s.len(), 3);
        assert!(s.contains(2) && s.contains(4) && !s.contains(5));
        assert!(s.overlaps(Span::new(4, 9)) && !s.overlaps(Span::new(5, 9)));
        assert_eq!(s.merge(Span::new(7, 9)), Span::new(2, 9));
        assert_eq!(s.shift(-3), Span::new(0, 2));
        assert_eq!(Span::new(0, 5).slice("hello world"), "hello");
        assert!(Span::new(0, 10).contains_span(s));
    }

    #[test]
    fn positions_are_utf16_and_one_based() {
        // "é" is 2 bytes / 1 UTF-16 unit; "𝄞" is 4 bytes / 2 UTF-16 units.
        let text = "ab\nxé𝄞z\r\nlast";
        let idx = LineIndex::new(text);
        assert_eq!(idx.line_count(), 3);
        assert_eq!(idx.pos(text, 0), Pos { line: 1, col: 1 });
        assert_eq!(idx.pos(text, 3), Pos { line: 2, col: 1 });
        let z = text.find('z').unwrap() as u32;
        assert_eq!(idx.pos(text, z), Pos { line: 2, col: 5 });
        assert_eq!(idx.offset(text, Pos { line: 2, col: 5 }), Some(z));
        // Column inside the surrogate pair is rejected.
        assert_eq!(idx.offset(text, Pos { line: 2, col: 4 }), None);
        // Past end of line clamps to line end (before \r\n).
        let line2_end = idx.line_end(text, 1);
        assert_eq!(&text[line2_end as usize..line2_end as usize + 2], "\r\n");
        assert_eq!(idx.offset(text, Pos { line: 2, col: 99 }), Some(line2_end));
        assert_eq!(idx.offset(text, Pos { line: 9, col: 1 }), None);
    }

    #[test]
    fn pos_snaps_to_char_boundary() {
        let text = "é";
        let idx = LineIndex::new(text);
        assert_eq!(idx.pos(text, 1), Pos { line: 1, col: 1 });
        assert_eq!(idx.pos(text, 99), Pos { line: 1, col: 2 });
    }

    #[test]
    fn source_file_line_text() {
        let f = SourceFile::new("a.tsx", "one\r\ntwo\nthree");
        assert_eq!(f.line_text(1), "one");
        assert_eq!(f.line_text(2), "two");
        assert_eq!(f.line_text(3), "three");
        assert_eq!(f.line_text(4), "");
    }

    #[test]
    fn normalize_path_uses_forward_slashes() {
        assert_eq!(
            normalize_path(Path::new(r"avdev\displays\PFD.tsx")),
            "avdev/displays/PFD.tsx"
        );
    }
}
