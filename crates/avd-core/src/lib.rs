//! Shared core types for WYSIWYG Avionics Builder.
//!
//! Every other crate builds on these:
//! - byte [`Span`]s with UTF-16 [`Pos`] mapping for the UI;
//! - rustc-style [`Diagnostic`]s with stable codes from [`codes`];
//! - the MSFS [`units`] table;
//! - variable keys ([`var`]).

pub mod codes;
pub mod diag;
pub mod lint;
pub mod render;
pub mod span;
pub mod units;
pub mod var;

pub use codes::CodeInfo;
pub use diag::{Applicability, Diagnostic, Edit, ElementRef, Fix, Label, Severity};
pub use lint::{LintGroup, LintLevel};
pub use render::{SourceLookup, render};
pub use span::{LineIndex, Pos, SourceFile, Span, normalize_path};
pub use units::{Dimension, Unit, UnitError};
pub use var::{VarKey, VarKind, VarParseError, VarSpec};
