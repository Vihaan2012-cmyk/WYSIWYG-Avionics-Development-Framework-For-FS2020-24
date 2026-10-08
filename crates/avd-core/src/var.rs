//! Variable keys and specs.
//!
//! Text forms:
//! - **Expression form** (spec §7.1), used in `ex`/`act`, vars.json and the UI:
//!   - `L:A32NX_SPEED`, `L:A32NX_SPEED:2`
//!   - `A:'AIRSPEED INDICATED'@knots`
//!   - `A:'COM STANDBY FREQUENCY:2'@mhz`
//!   - `E:'ZULU TIME'@seconds`
//!   - `K:'A32NX.ATHR_RESET_DISABLE'`
//! - **Sim form**, what the MSFS `SimVar` API takes: `L:A32NX_SPEED`, `A:AIRSPEED INDICATED:2`.
//!
//! Parsing is lenient: an unquoted name may contain spaces and dots. It ends at an `@`
//! that is followed by a unit (`@knots`, `@'feet per minute'`), or at the end of the string.
//!
//! Identity: names of kinds L, A, E, H and K are case-insensitive (compared upper-cased),
//! as in MSFS. B (bus topics) and C (Coherent events) are case-sensitive.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum VarKind {
    /// Local variable (LVAR).
    L,
    /// Aircraft SimVar.
    A,
    /// Environment variable.
    E,
    /// HTML event (fire-only).
    H,
    /// Key event (fire-only).
    K,
    /// msfs-sdk EventBus topic.
    B,
    /// Coherent event.
    C,
}

impl VarKind {
    pub const ALL: [VarKind; 7] = [
        Self::L,
        Self::A,
        Self::E,
        Self::H,
        Self::K,
        Self::B,
        Self::C,
    ];

    pub fn prefix(self) -> char {
        match self {
            Self::L => 'L',
            Self::A => 'A',
            Self::E => 'E',
            Self::H => 'H',
            Self::K => 'K',
            Self::B => 'B',
            Self::C => 'C',
        }
    }

    pub fn from_prefix(c: char) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.prefix() == c)
    }

    /// H: and K: are events: they can be fired, never read.
    pub fn is_event(self) -> bool {
        matches!(self, Self::H | Self::K)
    }

    pub fn case_insensitive(self) -> bool {
        !matches!(self, Self::B | Self::C)
    }

    /// Whether the MSFS SimVar API can read this kind.
    pub fn is_simvar_readable(self) -> bool {
        matches!(self, Self::L | Self::A | Self::E)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum VarParseError {
    #[error("variable reference is empty")]
    Empty,
    #[error("unknown variable kind `{0}`; expected one of L, A, E, H, K, B, C")]
    UnknownKind(String),
    #[error("expected `:` after variable kind in `{0}`")]
    MissingColon(String),
    #[error("variable name is empty in `{0}`")]
    EmptyName(String),
    #[error("unterminated quote in `{0}`")]
    UnterminatedQuote(String),
    #[error("unexpected text after variable reference in `{0}`")]
    Trailing(String),
}

/// A variable identity: kind + name + optional SimVar index.
#[derive(Clone, Debug)]
pub struct VarKey {
    pub kind: VarKind,
    pub name: String,
    pub index: Option<u32>,
}

impl VarKey {
    pub fn new(kind: VarKind, name: impl Into<String>, index: Option<u32>) -> Self {
        Self {
            kind,
            name: name.into(),
            index,
        }
    }

    /// Builds a key from a name that may carry a trailing `:N` index.
    pub fn with_name_and_index(kind: VarKind, raw: &str) -> Self {
        let (name, index) = split_index(raw);
        Self::new(kind, name, index)
    }

    /// Name used for identity comparisons.
    pub fn normalized_name(&self) -> Cow<'_, str> {
        if self.kind.case_insensitive() && self.name.bytes().any(|b| b.is_ascii_lowercase()) {
            Cow::Owned(self.name.to_ascii_uppercase())
        } else {
            Cow::Borrowed(&self.name)
        }
    }

    /// True if the expression form must quote the name.
    pub fn needs_quotes(&self) -> bool {
        self.name.is_empty()
            || !self
                .name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
            || self.name.as_bytes()[0].is_ascii_digit()
    }

    /// The string MSFS APIs expect, e.g. `L:A32NX_SPEED`, `A:AIRSPEED INDICATED:2`.
    pub fn sim_name(&self) -> String {
        match self.index {
            Some(i) => format!("{}:{}:{}", self.kind.prefix(), self.name, i),
            None => format!("{}:{}", self.kind.prefix(), self.name),
        }
    }

    /// Parses MSFS sim form. Names without a prefix default to `default_kind` (normally `A`).
    pub fn parse_sim(s: &str, default_kind: VarKind) -> Result<Self, VarParseError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(VarParseError::Empty);
        }
        let bytes = s.as_bytes();
        let (kind, rest) = if bytes.len() >= 2 && bytes[1] == b':' {
            let c = bytes[0] as char;
            match VarKind::from_prefix(c) {
                Some(k) => (k, &s[2..]),
                None => return Err(VarParseError::UnknownKind(c.to_string())),
            }
        } else {
            (default_kind, s)
        };
        if rest.trim().is_empty() {
            return Err(VarParseError::EmptyName(s.to_string()));
        }
        Ok(Self::with_name_and_index(kind, rest.trim()))
    }
}

fn split_index(raw: &str) -> (&str, Option<u32>) {
    if let Some((name, idx)) = raw.rsplit_once(':') {
        if !name.is_empty() && !idx.is_empty() && idx.bytes().all(|b| b.is_ascii_digit()) {
            if let Ok(i) = idx.parse() {
                return (name, Some(i));
            }
        }
    }
    (raw, None)
}

impl PartialEq for VarKey {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.index == other.index
            && self.normalized_name() == other.normalized_name()
    }
}

impl Eq for VarKey {}

impl Hash for VarKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.normalized_name().hash(state);
        self.index.hash(state);
    }
}

impl Ord for VarKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.kind
            .cmp(&other.kind)
            .then_with(|| self.normalized_name().cmp(&other.normalized_name()))
            .then_with(|| self.index.cmp(&other.index))
    }
}

impl PartialOrd for VarKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for VarKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let k = self.kind.prefix();
        if self.needs_quotes() {
            let escaped = self.name.replace('\\', "\\\\").replace('\'', "\\'");
            match self.index {
                Some(i) => write!(f, "{k}:'{escaped}:{i}'"),
                None => write!(f, "{k}:'{escaped}'"),
            }
        } else {
            match self.index {
                Some(i) => write!(f, "{k}:{}:{i}", self.name),
                None => write!(f, "{k}:{}", self.name),
            }
        }
    }
}

/// Parses a quoted string starting at `s[0] == '\''`. Returns the unescaped content and the
/// remaining text after the closing quote.
fn take_quoted(s: &str, whole: &str) -> Result<(String, String), VarParseError> {
    let mut out = String::new();
    let mut chars = s[1..].char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '\\' => {
                if let Some((_, n)) = chars.next() {
                    out.push(n);
                }
            }
            '\'' => return Ok((out, s[1 + i + 1..].to_string())),
            _ => out.push(c),
        }
    }
    Err(VarParseError::UnterminatedQuote(whole.to_string()))
}

/// Parses `X:name` or `X:'quoted name'` and returns the key plus the unparsed remainder.
fn parse_key_prefix(s: &str) -> Result<(VarKey, String), VarParseError> {
    let s = s.trim_start();
    if s.is_empty() {
        return Err(VarParseError::Empty);
    }
    let mut it = s.chars();
    let kc = it.next().ok_or(VarParseError::Empty)?;
    let kind =
        VarKind::from_prefix(kc).ok_or_else(|| VarParseError::UnknownKind(kc.to_string()))?;
    let rest = it.as_str();
    let rest = rest
        .strip_prefix(':')
        .ok_or_else(|| VarParseError::MissingColon(s.to_string()))?;
    if let Some(stripped) = rest.strip_prefix('\'') {
        let (name, remainder) = take_quoted(&format!("'{stripped}"), s)?;
        if name.trim().is_empty() {
            return Err(VarParseError::EmptyName(s.to_string()));
        }
        Ok((VarKey::with_name_and_index(kind, &name), remainder))
    } else {
        // Unquoted: everything up to an `@` that introduces a unit.
        let end = rest.find('@').unwrap_or(rest.len());
        let name = rest[..end].trim();
        if name.is_empty() {
            return Err(VarParseError::EmptyName(s.to_string()));
        }
        Ok((
            VarKey::with_name_and_index(kind, name),
            rest[end..].to_string(),
        ))
    }
}

impl FromStr for VarKey {
    type Err = VarParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (key, rest) = parse_key_prefix(s.trim())?;
        if !rest.trim().is_empty() {
            return Err(VarParseError::Trailing(s.to_string()));
        }
        Ok(key)
    }
}

impl Serialize for VarKey {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for VarKey {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// A variable reference with an optional unit, e.g. `L:A32NX_SPEED@knots`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VarSpec {
    pub key: VarKey,
    pub unit: Option<String>,
}

impl VarSpec {
    pub fn new(key: VarKey, unit: Option<String>) -> Self {
        Self { key, unit }
    }
}

impl fmt::Display for VarSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.key)?;
        if let Some(u) = &self.unit {
            if u.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') && !u.is_empty() {
                write!(f, "@{u}")?;
            } else {
                write!(f, "@'{}'", u.replace('\'', "\\'"))?;
            }
        }
        Ok(())
    }
}

impl FromStr for VarSpec {
    type Err = VarParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (key, rest) = parse_key_prefix(s.trim())?;
        let rest = rest.trim();
        if rest.is_empty() {
            return Ok(Self::new(key, None));
        }
        let unit_text = rest
            .strip_prefix('@')
            .ok_or_else(|| VarParseError::Trailing(s.to_string()))?
            .trim();
        let unit = if unit_text.starts_with('\'') {
            let (u, remainder) = take_quoted(unit_text, s)?;
            if !remainder.trim().is_empty() {
                return Err(VarParseError::Trailing(s.to_string()));
            }
            u
        } else {
            unit_text.to_string()
        };
        if unit.is_empty() {
            return Err(VarParseError::Trailing(s.to_string()));
        }
        Ok(Self::new(key, Some(unit)))
    }
}

impl Serialize for VarSpec {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for VarSpec {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn parses_expression_forms() {
        let k: VarKey = "L:A32NX_SPEED".parse().unwrap();
        assert_eq!(
            (k.kind, k.name.as_str(), k.index),
            (VarKind::L, "A32NX_SPEED", None)
        );

        let s: VarSpec = "A:'AIRSPEED INDICATED'@knots".parse().unwrap();
        assert_eq!(s.key.name, "AIRSPEED INDICATED");
        assert_eq!(s.unit.as_deref(), Some("knots"));

        let s: VarSpec = "A:'COM STANDBY FREQUENCY:2'@'feet per minute'"
            .parse()
            .unwrap();
        assert_eq!(s.key.index, Some(2));
        assert_eq!(s.unit.as_deref(), Some("feet per minute"));

        let s: VarSpec = "L:A32NX_ADIRS_ADR_1_BARO_CORRECTED_ALTITUDE_1:3@feet"
            .parse()
            .unwrap();
        assert_eq!(s.key.name, "A32NX_ADIRS_ADR_1_BARO_CORRECTED_ALTITUDE_1");
        assert_eq!(s.key.index, Some(3));

        let k: VarKey = "K:'A32NX.ATHR_RESET_DISABLE'".parse().unwrap();
        assert_eq!(k.name, "A32NX.ATHR_RESET_DISABLE");
    }

    #[test]
    fn lenient_unquoted_names() {
        let s: VarSpec = "A:AIRSPEED INDICATED@knots".parse().unwrap();
        assert_eq!(s.key.name, "AIRSPEED INDICATED");
        let k: VarKey = "K:A32NX.ATHR_RESET_DISABLE".parse().unwrap();
        assert_eq!(k.name, "A32NX.ATHR_RESET_DISABLE");
    }

    #[test]
    fn display_round_trips() {
        for src in [
            "L:A32NX_SPEED",
            "L:A32NX_SPEED:2",
            "A:'AIRSPEED INDICATED'",
            "A:'COM STANDBY FREQUENCY:2'",
            "K:'A32NX.ATHR_RESET_DISABLE'",
            "B:speedAr",
            "C:'PLAY_INSTRUMENT_SOUND'",
        ] {
            let k: VarKey = src.parse().unwrap();
            assert_eq!(
                k.to_string(),
                src.replace("C:'PLAY_INSTRUMENT_SOUND'", "C:PLAY_INSTRUMENT_SOUND")
            );
        }
        let s: VarSpec = "A:'AIRSPEED INDICATED'@knots".parse().unwrap();
        assert_eq!(s.to_string(), "A:'AIRSPEED INDICATED'@knots");
        let s: VarSpec = "L:X@'feet per minute'".parse().unwrap();
        assert_eq!(s.to_string(), "L:X@'feet per minute'");
    }

    #[test]
    fn sim_form() {
        let k = VarKey::parse_sim("INDICATED ALTITUDE", VarKind::A).unwrap();
        assert_eq!(k.sim_name(), "A:INDICATED ALTITUDE");
        let k = VarKey::parse_sim("A:LIGHT POTENTIOMETER:88", VarKind::A).unwrap();
        assert_eq!(
            (k.name.as_str(), k.index),
            ("LIGHT POTENTIOMETER", Some(88))
        );
        assert_eq!(k.sim_name(), "A:LIGHT POTENTIOMETER:88");
        let k = VarKey::parse_sim("E:ZULU TIME", VarKind::A).unwrap();
        assert_eq!(k.kind, VarKind::E);
        assert!(VarKey::parse_sim("Q:FOO", VarKind::A).is_err());
    }

    #[test]
    fn identity_is_case_insensitive_except_bus_and_coherent() {
        let a: VarKey = "L:a32nx_speed".parse().unwrap();
        let b: VarKey = "L:A32NX_SPEED".parse().unwrap();
        assert_eq!(a, b);
        let set: HashSet<_> = [a, b].into_iter().collect();
        assert_eq!(set.len(), 1);
        let t1: VarKey = "B:speedAr".parse().unwrap();
        let t2: VarKey = "B:SPEEDAR".parse().unwrap();
        assert_ne!(t1, t2);
    }

    #[test]
    fn errors() {
        assert_eq!("".parse::<VarKey>().unwrap_err(), VarParseError::Empty);
        assert!(matches!(
            "Q:X".parse::<VarKey>(),
            Err(VarParseError::UnknownKind(_))
        ));
        assert!(matches!(
            "LX".parse::<VarKey>(),
            Err(VarParseError::MissingColon(_))
        ));
        assert!(matches!(
            "L:".parse::<VarKey>(),
            Err(VarParseError::EmptyName(_))
        ));
        assert!(matches!(
            "A:'OPEN".parse::<VarKey>(),
            Err(VarParseError::UnterminatedQuote(_))
        ));
        assert!(matches!(
            "L:X@knots".parse::<VarKey>(),
            Err(VarParseError::Trailing(_))
        ));
        assert!(matches!(
            "L:X@".parse::<VarSpec>(),
            Err(VarParseError::Trailing(_))
        ));
    }

    #[test]
    fn serde_uses_string_form() {
        let s: VarSpec = "A:'AIRSPEED INDICATED'@knots".parse().unwrap();
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, "\"A:'AIRSPEED INDICATED'@knots\"");
        let back: VarSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s);
    }
}
