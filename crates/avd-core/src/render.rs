//! rustc-style plain-text rendering of diagnostics (terminal pane, logs, snapshot tests).
//!
//! ```text
//! error[AV2003]: mismatched units of the same dimension
//!   --> avdev/displays/PFD.tsx:3:12
//!    |
//!  3 |   y={ex`L:A@knots + L:B@feet per minute`}
//!    |         ^^^^^^^^^ expected `knots`
//!    |
//!    = note: ...
//!    = help: ...
//! ```

use crate::diag::{Diagnostic, Label};
use crate::span::SourceFile;
use std::collections::HashMap;
use std::fmt::Write as _;

/// Resolves a diagnostic's file path to its source text.
pub trait SourceLookup {
    fn source(&self, path: &str) -> Option<&SourceFile>;
}

impl SourceLookup for HashMap<String, SourceFile> {
    fn source(&self, path: &str) -> Option<&SourceFile> {
        self.get(path)
    }
}

impl SourceLookup for [SourceFile] {
    fn source(&self, path: &str) -> Option<&SourceFile> {
        self.iter().find(|f| f.path == path)
    }
}

impl SourceLookup for SourceFile {
    fn source(&self, path: &str) -> Option<&SourceFile> {
        (self.path == path).then_some(self)
    }
}

/// Renders one diagnostic as rustc-style text. Labels whose file is not found by `sources`
/// are rendered as a bare `--> path:line:col` location without a snippet.
pub fn render(diag: &Diagnostic, sources: &dyn SourceLookup) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{}[{}]: {}",
        diag.severity.as_str(),
        diag.code,
        diag.message
    );

    let labels: Vec<(&Label, char)> = diag
        .primary
        .iter()
        .map(|l| (l, '^'))
        .chain(diag.secondary.iter().map(|l| (l, '-')))
        .collect();

    let max_line = labels.iter().map(|(l, _)| l.start.line).max().unwrap_or(1);
    let width = max_line.to_string().len();
    let pad = " ".repeat(width);

    let mut last_file: Option<&str> = None;
    for (label, mark) in &labels {
        if last_file != Some(label.file.as_str()) {
            let _ = writeln!(
                out,
                "{pad}--> {}:{}:{}",
                label.file, label.start.line, label.start.col
            );
            last_file = Some(label.file.as_str());
        }
        if let Some(file) = sources.source(&label.file) {
            render_snippet(&mut out, file, label, *mark, width);
        }
    }

    let has_trailer = !diag.notes.is_empty()
        || diag.help.is_some()
        || !diag.fixes.is_empty()
        || diag.lint.is_some();
    if has_trailer && !labels.is_empty() {
        let _ = writeln!(out, "{pad} |");
    }
    for note in &diag.notes {
        let _ = writeln!(out, "{pad} = note: {note}");
    }
    if let Some(lint) = &diag.lint {
        let _ = writeln!(
            out,
            "{pad} = note: `{}` is in lint group `{lint}`; set its level in avdev.json \"lints\"",
            diag.code
        );
    }
    if let Some(help) = &diag.help {
        let _ = writeln!(out, "{pad} = help: {help}");
    }
    for fix in &diag.fixes {
        if diag.help.as_deref() != Some(fix.title.as_str()) {
            let _ = writeln!(out, "{pad} = help: {}", fix.title);
        }
    }
    out
}

fn render_snippet(out: &mut String, file: &SourceFile, label: &Label, mark: char, width: usize) {
    let pad = " ".repeat(width);
    let line_no = label.start.line;
    let line = file.line_text(line_no);
    let line_start = file.index.line_start(line_no - 1) as usize;

    // Terminal columns count chars, not UTF-16 units.
    let start = (label.span.start as usize).max(line_start);
    let start_col = file
        .text
        .get(line_start..start)
        .map_or(0, |s| s.chars().count());
    let line_end = line_start + line.len();
    let end = (label.span.end as usize).clamp(start, line_end);
    let underline_len = file
        .text
        .get(start..end)
        .map_or(0, |s| s.chars().count())
        .max(1);
    let continues = (label.span.end as usize) > line_end;

    let _ = writeln!(out, "{pad} |");
    let _ = writeln!(out, "{line_no:>width$} | {line}");
    let mut underline = format!(
        "{pad} | {}{}",
        " ".repeat(start_col),
        mark.to_string().repeat(underline_len)
    );
    if continues {
        underline.push_str("...");
    }
    if let Some(text) = &label.text {
        underline.push(' ');
        underline.push_str(text);
    }
    let _ = writeln!(out, "{underline}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codes;
    use crate::diag::{Applicability, Fix};
    use crate::span::Span;

    #[test]
    fn renders_like_rustc() {
        let src = "<Rect id=\"a\"\n  y={ex`L:A@knots + L:B@feet`} />\n";
        let file = SourceFile::new("avdev/displays/PFD.tsx", src);
        let start = src.find("L:B@feet").unwrap() as u32;
        let first = src.find("L:A@knots").unwrap() as u32;
        let d = Diagnostic::new(
            &codes::AV2004,
            "mismatched dimensions: `knots` (speed) vs `feet` (length)",
        )
        .at(&file, Span::new(start, start + 8), "this is a length")
        .secondary_at(&file, Span::new(first, first + 9), "this is a speed")
        .with_note("dimensions must match for `+`")
        .with_help("did you mean `L:B@feet per minute`?")
        .with_fix(Fix::new(
            "use `feet per minute`",
            Applicability::MaybeIncorrect,
            vec![],
        ));
        let text = render(&d, &file);
        let expected = "\
error[AV2004]: mismatched dimensions: `knots` (speed) vs `feet` (length)
 --> avdev/displays/PFD.tsx:2:21
  |
2 |   y={ex`L:A@knots + L:B@feet`} />
  |                     ^^^^^^^^ this is a length
  |
2 |   y={ex`L:A@knots + L:B@feet`} />
  |         --------- this is a speed
  |
  = note: dimensions must match for `+`
  = help: did you mean `L:B@feet per minute`?
  = help: use `feet per minute`
";
        assert_eq!(text, expected);
    }

    #[test]
    fn renders_without_source_and_lint_note() {
        let d = Diagnostic::new(
            &codes::AV5003,
            "repeater `ticks` has 120 instances and no `cull`",
        );
        let lookup: HashMap<String, SourceFile> = HashMap::new();
        let text = render(&d, &lookup);
        assert!(text.starts_with("warning[AV5003]: repeater"));
        assert!(text.contains("lint group `perf`"));
    }
}
