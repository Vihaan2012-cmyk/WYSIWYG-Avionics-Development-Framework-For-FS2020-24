//! Lint levels and groups (spec §8.4).

use serde::{Deserialize, Serialize};
use std::fmt;

/// Level configured for a lint code or group in `avdev.json` `lints`, or via an element's
/// `allow={[...]}` prop.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LintLevel {
    Allow,
    Warn,
    Deny,
    /// Like `Deny`, but element-level `allow` cannot lower it.
    Forbid,
}

impl LintLevel {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "allow" => Some(Self::Allow),
            "warn" => Some(Self::Warn),
            "deny" => Some(Self::Deny),
            "forbid" => Some(Self::Forbid),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Warn => "warn",
            Self::Deny => "deny",
            Self::Forbid => "forbid",
        }
    }
}

impl fmt::Display for LintLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Lint groups. Every lint code belongs to exactly one group (see [`crate::codes`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LintGroup {
    Perf,
    States,
    Units,
    Style,
    Assets,
}

impl LintGroup {
    pub const ALL: [LintGroup; 5] = [
        Self::Perf,
        Self::States,
        Self::Units,
        Self::Style,
        Self::Assets,
    ];

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|g| g.as_str() == s)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Perf => "perf",
            Self::States => "states",
            Self::Units => "units",
            Self::Style => "style",
            Self::Assets => "assets",
        }
    }
}

impl fmt::Display for LintGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        for l in [
            LintLevel::Allow,
            LintLevel::Warn,
            LintLevel::Deny,
            LintLevel::Forbid,
        ] {
            assert_eq!(LintLevel::parse(l.as_str()), Some(l));
        }
        for g in LintGroup::ALL {
            assert_eq!(LintGroup::parse(g.as_str()), Some(g));
        }
        assert_eq!(
            serde_json::to_string(&LintLevel::Forbid).unwrap(),
            "\"forbid\""
        );
        assert_eq!(LintGroup::parse("nope"), None);
    }
}
