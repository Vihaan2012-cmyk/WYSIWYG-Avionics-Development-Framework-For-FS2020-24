//! Registry of every diagnostic code (spec §8.3). Codes are stable public API: never renumber.
//! Every code must have a `docs/diagnostics/<CODE>.md` page (enforced by a test in avd-check).

use crate::diag::Severity;
use crate::lint::LintGroup;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct CodeInfo {
    pub code: &'static str,
    pub default_severity: Severity,
    /// `Some` for lints (level configurable); `None` for hard errors/notes.
    pub lint_group: Option<LintGroup>,
    pub title: &'static str,
}

macro_rules! codes {
    ($( $name:ident = ($sev:ident, $group:expr, $title:literal); )*) => {
        $( pub const $name: CodeInfo = CodeInfo {
            code: stringify!($name),
            default_severity: Severity::$sev,
            lint_group: $group,
            title: $title,
        }; )*
        /// All codes in ascending order.
        pub static ALL: &[&CodeInfo] = &[$(&$name),*];
    };
}

const PERF: Option<LintGroup> = Some(LintGroup::Perf);
const STATES: Option<LintGroup> = Some(LintGroup::States);
const UNITS: Option<LintGroup> = Some(LintGroup::Units);
const STYLE: Option<LintGroup> = Some(LintGroup::Style);
const ASSETS: Option<LintGroup> = Some(LintGroup::Assets);

codes! {
    // AV00xx — syntax and dialect; AV09xx — avdev.json
    AV0001 = (Error, None, "syntax error");
    AV0101 = (Error, None, "unknown dialect element");
    AV0102 = (Error, None, "element is missing an `id`");
    AV0103 = (Error, None, "duplicate element id");
    AV0104 = (Error, None, "invalid prop value shape");
    AV0105 = (Error, None, "unknown prop for element");
    AV0106 = (Error, None, "missing required prop");
    AV0201 = (Error, None, "`${}` substitution inside an `ex`/`act` template");
    AV0901 = (Warning, None, "unknown key in avdev.json");
    AV0902 = (Error, None, "invalid value in avdev.json");
    AV0903 = (Error, None, "duplicate display id in avdev.json");
    // AV1xxx — name resolution
    AV1001 = (Error, None, "unknown variable");
    AV1102 = (Error, None, "bound variable no longer exists in the aircraft");
    AV1201 = (Error, None, "unknown font");
    AV1202 = (Error, None, "unknown palette color");
    AV1203 = (Error, None, "unknown image");
    AV1204 = (Error, None, "unknown gradient");
    AV1301 = (Warning, STYLE, "variable name does not follow the aircraft's naming convention");
    AV1401 = (Error, None, "logic function signature is not annotated with number/boolean/string");
    AV1402 = (Error, None, "unknown logic function");
    AV1501 = (Error, None, "unknown override target");
    AV1502 = (Error, None, "unknown state, mode, page or graph");
    AV1503 = (Error, None, "unknown component");
    AV1504 = (Error, None, "unknown component prop reference");
    AV1505 = (Error, None, "unknown loop variable");
    AV1601 = (Error, None, "unknown unit");
    // AV2xxx — types and units
    AV2001 = (Error, None, "bool used as a number");
    AV2002 = (Error, None, "prop type mismatch");
    AV2003 = (Error, None, "mismatched units of the same dimension");
    AV2004 = (Error, None, "mismatched dimensions");
    AV2005 = (Warning, UNITS, "unitless value used where a unit is expected");
    AV2010 = (Error, None, "ARINC 429 word used as a number");
    AV2020 = (Error, None, "unknown enum label");
    AV2030 = (Error, None, "wrong number of arguments");
    AV2031 = (Error, None, "unknown function");
    AV2101 = (Warning, None, "write to a read-only A: variable");
    AV2102 = (Error, None, "event used in an expression");
    AV2103 = (Error, None, "target of set()/toggle() is not settable");
    // AV3xxx — states, pages, graphs, widgets
    AV3001 = (Warning, STATES, "overlapping state overrides");
    AV3002 = (Error, None, "conflicting overrides at equal priority");
    AV3003 = (Error, None, "circular state dependency");
    AV3010 = (Warning, STATES, "state machine mode is unreachable");
    AV3011 = (Error, None, "transition references an unknown mode");
    AV3012 = (Note, STATES, "mode has no outgoing transitions");
    AV3020 = (Error, None, "node graph contains a cycle");
    AV3101 = (Error, None, "widget on a non-interactive display");
    AV3201 = (Error, None, "display has no default page");
    // AV4xxx — assets
    AV4001 = (Error, None, "font file not found");
    AV4002 = (Error, None, "unsupported font format");
    AV4003 = (Error, None, "image file not found");
    AV4101 = (Warning, ASSETS, "image is larger than 4 MB");
    AV4201 = (Warning, ASSETS, "display size disagrees with panel.cfg gauge");
    // AV5xxx — layout and perf
    AV5001 = (Warning, PERF, "element is entirely outside the display");
    AV5002 = (Warning, PERF, "display materialises more than 2000 elements");
    AV5003 = (Warning, PERF, "repeater with more than 50 instances has no `cull`");
    AV5010 = (Warning, PERF, "timing function inside a culled repeater");
    AV5020 = (Note, PERF, "refresh rate above 60 Hz");
    // AV6xxx — build and toolchain
    AV6001 = (Error, None, "Mach is not linked");
    AV6002 = (Error, None, "Node version too old for the linked Mach");
    AV6003 = (Error, None, "unsupported Mach version");
    AV6010 = (Error, None, "bundler error");
    AV6011 = (Warning, None, "bundler warning");
    AV6020 = (Error, None, "ACE output folder is not writable");
    // AV9xxx — opaque code
    AV9001 = (Note, STYLE, "attribute value is opaque code");
    AV9002 = (Note, STYLE, "foreign JSX wrapped as a code block");
}

/// Looks up a code by its string, e.g. `"AV2003"`.
pub fn lookup(code: &str) -> Option<&'static CodeInfo> {
    ALL.iter().copied().find(|c| c.code == code)
}

/// All lint codes in `group`.
pub fn in_group(group: LintGroup) -> impl Iterator<Item = &'static CodeInfo> {
    ALL.iter()
        .copied()
        .filter(move |c| c.lint_group == Some(group))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_sorted_unique_and_well_formed() {
        let mut prev = "";
        for c in ALL {
            assert!(c.code.len() == 6 && c.code.starts_with("AV"), "{}", c.code);
            assert!(
                c.code[2..].chars().all(|d| d.is_ascii_digit()),
                "{}",
                c.code
            );
            assert!(
                c.code > prev,
                "codes must be ascending and unique: {} after {}",
                c.code,
                prev
            );
            prev = c.code;
        }
    }

    #[test]
    fn lookup_and_groups() {
        assert_eq!(
            lookup("AV2003").map(|c| c.title),
            Some("mismatched units of the same dimension")
        );
        assert!(lookup("AV9999").is_none());
        let perf: Vec<_> = in_group(LintGroup::Perf).map(|c| c.code).collect();
        assert_eq!(perf, ["AV5001", "AV5002", "AV5003", "AV5010", "AV5020"]);
        // Hard errors are never lints.
        assert!(
            ALL.iter()
                .filter(|c| c.lint_group.is_some())
                .all(|c| c.default_severity != Severity::Error)
        );
    }
}
