//! Diagnostics (spec §8.2). Serialized as camelCase JSON for the UI.

use crate::codes::CodeInfo;
use crate::span::{Pos, SourceFile, Span};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Help,
    Note,
    Warning,
    Error,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Help => "help",
            Self::Note => "note",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// A source location with an optional message, carrying both the UTF-8 byte span (for edits)
/// and the 1-based UTF-16 start/end positions (for Monaco and the UI).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub file: String,
    pub span: Span,
    pub start: Pos,
    pub end: Pos,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

impl Label {
    pub fn new(file: &SourceFile, span: Span, text: Option<String>) -> Self {
        Self {
            file: file.path.clone(),
            span,
            start: file.pos(span.start),
            end: file.pos(span.end),
            text,
        }
    }
}

/// A text replacement in a file. An empty span inserts; empty `text` deletes.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edit {
    pub file: String,
    pub span: Span,
    pub text: String,
}

impl Edit {
    pub fn replace(file: impl Into<String>, span: Span, text: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            span,
            text: text.into(),
        }
    }

    pub fn insert(file: impl Into<String>, at: u32, text: impl Into<String>) -> Self {
        Self::replace(file, Span::empty(at), text)
    }

    pub fn delete(file: impl Into<String>, span: Span) -> Self {
        Self::replace(file, span, "")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Applicability {
    /// Safe to apply automatically ("Fix all").
    MachineApplicable,
    /// Probably right; requires the user to choose it.
    MaybeIncorrect,
}

/// A quick fix: one or more edits applied atomically.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fix {
    pub title: String,
    pub applicability: Applicability,
    pub edits: Vec<Edit>,
}

impl Fix {
    pub fn new(title: impl Into<String>, applicability: Applicability, edits: Vec<Edit>) -> Self {
        Self {
            title: title.into(),
            applicability,
            edits,
        }
    }
}

/// Identifies the canvas element a diagnostic is about.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementRef {
    pub file: String,
    pub element_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<Label>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secondary: Vec<Label>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixes: Vec<Fix>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element: Option<ElementRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lint: Option<String>,
}

impl Diagnostic {
    /// Creates a diagnostic with the code's default severity and lint group.
    pub fn new(code: &CodeInfo, message: impl Into<String>) -> Self {
        Self {
            code: code.code.to_string(),
            severity: code.default_severity,
            message: message.into(),
            primary: None,
            secondary: Vec::new(),
            notes: Vec::new(),
            help: None,
            fixes: Vec::new(),
            element: None,
            lint: code.lint_group.map(|g| g.as_str().to_string()),
        }
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_primary(mut self, label: Label) -> Self {
        self.primary = Some(label);
        self
    }

    /// Sets the primary label at `span` in `file` with a message.
    pub fn at(self, file: &SourceFile, span: Span, text: impl Into<String>) -> Self {
        let label = Label::new(file, span, Some(text.into()));
        self.with_primary(label)
    }

    /// Sets the primary label at `span` in `file` without a message.
    pub fn at_span(self, file: &SourceFile, span: Span) -> Self {
        let label = Label::new(file, span, None);
        self.with_primary(label)
    }

    pub fn with_secondary(mut self, label: Label) -> Self {
        self.secondary.push(label);
        self
    }

    pub fn secondary_at(self, file: &SourceFile, span: Span, text: impl Into<String>) -> Self {
        let label = Label::new(file, span, Some(text.into()));
        self.with_secondary(label)
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_fix(mut self, fix: Fix) -> Self {
        self.fixes.push(fix);
        self
    }

    pub fn with_element(mut self, file: impl Into<String>, element_id: impl Into<String>) -> Self {
        self.element = Some(ElementRef {
            file: file.into(),
            element_id: element_id.into(),
        });
        self
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codes;

    #[test]
    fn builder_and_json_shape() {
        let file = SourceFile::new("avdev/displays/PFD.tsx", "let x = L:FOO@knots;\n");
        let d = Diagnostic::new(&codes::AV2003, "mismatched units")
            .at(&file, Span::new(8, 19), "expected `knots`")
            .with_note("note text")
            .with_help("help text")
            .with_fix(Fix::new(
                "wrap in convert",
                Applicability::MachineApplicable,
                vec![Edit::replace(
                    &file.path,
                    Span::new(8, 19),
                    "convert(L:FOO, 'knots')",
                )],
            ))
            .with_element(&file.path, "spdTape");
        assert!(d.is_error());
        let json = serde_json::to_value(&d).unwrap();
        assert_eq!(json["code"], "AV2003");
        assert_eq!(json["severity"], "error");
        assert_eq!(json["primary"]["start"]["line"], 1);
        assert_eq!(json["primary"]["start"]["col"], 9);
        assert_eq!(json["fixes"][0]["applicability"], "machineApplicable");
        assert_eq!(json["element"]["elementId"], "spdTape");
        assert!(json.get("secondary").is_none());
        let back: Diagnostic = serde_json::from_value(json).unwrap();
        assert_eq!(back, d);
    }

    #[test]
    fn lint_codes_carry_group() {
        let d = Diagnostic::new(&codes::AV5003, "repeater without cull");
        assert_eq!(d.severity, Severity::Warning);
        assert_eq!(d.lint.as_deref(), Some("perf"));
    }
}
