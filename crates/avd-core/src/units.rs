//! MSFS unit table, grouped by physical dimension, with linear/affine conversion (spec §7.2).
//!
//! Names follow the MSFS SDK "Simulation Variable Units" list plus the string values of
//! msfs-sdk `SimVarValueType`. Lookup is case-insensitive and collapses whitespace.

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dimension {
    /// Plain numbers ("number", "position", "mask", BCD values...). Freely interconvertible.
    Unitless,
    Bool,
    Enum,
    String,
    /// Structured values (LLA, PBH, XYZ) that are not scalars.
    Struct,
    Length,
    Speed,
    /// Mach number: a speed ratio that cannot be converted to a speed without air data.
    Mach,
    Acceleration,
    Angle,
    AngularVelocity,
    Pressure,
    Temperature,
    Mass,
    Time,
    Frequency,
    /// Ratios: percent, percent over 100, part.
    Ratio,
    Force,
    Volume,
    MassFlow,
    VolumeFlow,
    Voltage,
    Current,
    Power,
    Density,
    Torque,
}

impl Dimension {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unitless => "unitless",
            Self::Bool => "bool",
            Self::Enum => "enum",
            Self::String => "string",
            Self::Struct => "struct",
            Self::Length => "length",
            Self::Speed => "speed",
            Self::Mach => "mach",
            Self::Acceleration => "acceleration",
            Self::Angle => "angle",
            Self::AngularVelocity => "angular velocity",
            Self::Pressure => "pressure",
            Self::Temperature => "temperature",
            Self::Mass => "mass",
            Self::Time => "time",
            Self::Frequency => "frequency",
            Self::Ratio => "ratio",
            Self::Force => "force",
            Self::Volume => "volume",
            Self::MassFlow => "mass flow",
            Self::VolumeFlow => "volume flow",
            Self::Voltage => "voltage",
            Self::Current => "current",
            Self::Power => "power",
            Self::Density => "density",
            Self::Torque => "torque",
        }
    }

    /// Dimensions whose values are numbers that can carry a unit in expressions.
    pub fn is_numeric(self) -> bool {
        !matches!(self, Self::Bool | Self::Enum | Self::String | Self::Struct)
    }
}

/// A unit: `value_in_base = value * scale + offset`.
#[derive(Debug, PartialEq)]
pub struct Unit {
    /// Canonical name, used when printing and in generated code.
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub dimension: Dimension,
    pub scale: f64,
    pub offset: f64,
}

impl Unit {
    pub fn to_base(&self, v: f64) -> f64 {
        v * self.scale + self.offset
    }

    pub fn from_base(&self, v: f64) -> f64 {
        (v - self.offset) / self.scale
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum UnitError {
    #[error("unknown unit `{0}`")]
    Unknown(String),
    #[error("cannot convert `{from}` ({from_dim}) to `{to}` ({to_dim})")]
    DimensionMismatch {
        from: &'static str,
        from_dim: &'static str,
        to: &'static str,
        to_dim: &'static str,
    },
}

const DEG_PER_RAD: f64 = 180.0 / PI;
const F_OFFSET: f64 = 459.67 * 5.0 / 9.0;

macro_rules! u {
    ($name:literal, [$($alias:literal),*], $dim:ident, $scale:expr) => {
        u!($name, [$($alias),*], $dim, $scale, 0.0)
    };
    ($name:literal, [$($alias:literal),*], $dim:ident, $scale:expr, $offset:expr) => {
        Unit { name: $name, aliases: &[$($alias),*], dimension: Dimension::$dim, scale: $scale, offset: $offset }
    };
}

static UNITS: &[Unit] = &[
    // Unitless and non-scalar
    u!(
        "number",
        ["numbers", "scalar", "part", "ratio"],
        Unitless,
        1.0
    ),
    u!(
        "position",
        ["position 16k", "position 32k", "position 128"],
        Unitless,
        1.0
    ),
    u!("mask", ["flags"], Unitless, 1.0),
    u!(
        "bcd16",
        [
            "bco16",
            "bcd32",
            "frequency bcd16",
            "frequency bcd32",
            "frequency adf bcd32"
        ],
        Unitless,
        1.0
    ),
    u!("bool", ["boolean", "bools"], Bool, 1.0),
    u!("enum", ["enums"], Enum, 1.0),
    u!("string", ["strings"], String, 1.0),
    u!(
        "latlonalt",
        ["lla", "pbh", "xyz", "latlonaltpbh", "pid_struct"],
        Struct,
        1.0
    ),
    // Length (base: meter)
    u!("meters", ["meter", "m"], Length, 1.0),
    u!("centimeters", ["centimeter", "cm"], Length, 0.01),
    u!("millimeters", ["millimeter", "mm"], Length, 0.001),
    u!("kilometers", ["kilometer", "km"], Length, 1000.0),
    u!("feet", ["foot", "ft"], Length, 0.3048),
    u!("inches", ["inch", "in"], Length, 0.0254),
    u!("yards", ["yard", "yd"], Length, 0.9144),
    u!("miles", ["mile", "mi", "statute miles"], Length, 1609.344),
    u!("decimiles", ["decimile"], Length, 160.9344),
    u!(
        "nautical miles",
        ["nautical mile", "nmile", "nmiles", "nm"],
        Length,
        1852.0
    ),
    // Speed (base: m/s)
    u!("meters per second", ["meter per second", "m/s"], Speed, 1.0),
    u!(
        "meters per minute",
        ["meter per minute", "m/min"],
        Speed,
        1.0 / 60.0
    ),
    u!(
        "feet per second",
        ["foot per second", "ft/s"],
        Speed,
        0.3048
    ),
    u!(
        "feet per minute",
        ["foot per minute", "ft/min", "fpm"],
        Speed,
        0.3048 / 60.0
    ),
    u!(
        "kilometers per hour",
        ["kilometer per hour", "kph", "km/h"],
        Speed,
        1.0 / 3.6
    ),
    u!("knots", ["knot", "kts", "kt"], Speed, 1852.0 / 3600.0),
    u!("miles per hour", ["mile per hour", "mph"], Speed, 0.44704),
    u!("mach", ["machs"], Mach, 1.0),
    // Acceleration (base: m/s²)
    u!(
        "meters per second squared",
        ["meter per second squared"],
        Acceleration,
        1.0
    ),
    u!(
        "feet per second squared",
        ["foot per second squared"],
        Acceleration,
        0.3048
    ),
    u!("g force", ["gforce", "g"], Acceleration, 9.80665),
    // Angle (base: degree)
    u!(
        "degrees",
        [
            "degree",
            "deg",
            "degrees latitude",
            "degrees longitude",
            "degree latitude",
            "degree longitude"
        ],
        Angle,
        1.0
    ),
    u!("radians", ["radian", "rad"], Angle, DEG_PER_RAD),
    u!("grads", ["grad"], Angle, 0.9),
    // Angular velocity (base: degree per second)
    u!(
        "degrees per second",
        ["degree per second", "deg/s"],
        AngularVelocity,
        1.0
    ),
    u!(
        "radians per second",
        ["radian per second", "rad/s"],
        AngularVelocity,
        DEG_PER_RAD
    ),
    u!(
        "rpm",
        ["rpms", "revolutions per minute"],
        AngularVelocity,
        6.0
    ),
    // Pressure (base: pascal)
    u!("pascals", ["pascal", "pa"], Pressure, 1.0),
    u!("kilopascals", ["kilopascal", "kpa"], Pressure, 1000.0),
    u!(
        "millibars",
        [
            "millibar",
            "mbar",
            "mbars",
            "mb",
            "hectopascals",
            "hectopascal",
            "hpa"
        ],
        Pressure,
        100.0
    ),
    u!(
        "inches of mercury",
        ["inch of mercury", "inhg", "in hg"],
        Pressure,
        3386.389
    ),
    u!(
        "millimeters of mercury",
        ["millimeter of mercury", "mmhg"],
        Pressure,
        133.322_387
    ),
    u!(
        "psi",
        ["pounds per square inch", "pound per square inch"],
        Pressure,
        6894.757
    ),
    u!(
        "psf",
        ["pounds per square foot", "pound per square foot"],
        Pressure,
        47.880_26
    ),
    u!("atmospheres", ["atmosphere", "atm"], Pressure, 101_325.0),
    u!("bars", ["bar"], Pressure, 100_000.0),
    // Temperature (base: kelvin)
    u!("kelvin", ["k"], Temperature, 1.0),
    u!(
        "celsius",
        ["degrees celsius", "degree celsius", "c"],
        Temperature,
        1.0,
        273.15
    ),
    u!(
        "fahrenheit",
        ["farenheit", "degrees fahrenheit", "degree fahrenheit", "f"],
        Temperature,
        5.0 / 9.0,
        F_OFFSET
    ),
    u!(
        "rankine",
        ["degrees rankine", "degree rankine"],
        Temperature,
        5.0 / 9.0
    ),
    // Mass (base: kg)
    u!("kilograms", ["kilogram", "kg", "kgs"], Mass, 1.0),
    u!("grams", ["gram", "gr"], Mass, 0.001),
    u!("pounds", ["pound", "lb", "lbs"], Mass, 0.453_592_37),
    u!("slugs", ["slug"], Mass, 14.593_903),
    u!(
        "tonnes",
        ["tonne", "metric tons", "metric ton"],
        Mass,
        1000.0
    ),
    u!("tons", ["ton", "short tons"], Mass, 907.184_74),
    // Time (base: second)
    u!("seconds", ["second", "sec", "secs", "s"], Time, 1.0),
    u!("milliseconds", ["millisecond", "ms"], Time, 0.001),
    u!("minutes", ["minute", "min"], Time, 60.0),
    u!("hours", ["hour", "hr", "h"], Time, 3600.0),
    u!("days", ["day"], Time, 86_400.0),
    // Frequency (base: Hz)
    u!("hertz", ["hz"], Frequency, 1.0),
    u!("kilohertz", ["khz"], Frequency, 1.0e3),
    u!("megahertz", ["mhz"], Frequency, 1.0e6),
    // Ratio (base: 1)
    u!("percent", ["percentage"], Ratio, 0.01),
    u!("percent over 100", ["percent_over_100"], Ratio, 1.0),
    // Force (base: newton)
    u!("newtons", ["newton", "n"], Force, 1.0),
    u!("pounds force", ["pound force", "lbf"], Force, 4.448_221_6),
    // Volume (base: m³)
    u!("cubic meters", ["cubic meter", "m3"], Volume, 1.0),
    u!("liters", ["liter", "l"], Volume, 0.001),
    u!("gallons", ["gallon", "gal"], Volume, 0.003_785_411_784),
    u!("cubic feet", ["cubic foot", "ft3"], Volume, 0.028_316_846_6),
    u!(
        "cubic inches",
        ["cubic inch", "in3"],
        Volume,
        1.638_706_4e-5
    ),
    // Mass flow (base: kg/s)
    u!(
        "kilograms per second",
        ["kilogram per second", "kg/s"],
        MassFlow,
        1.0
    ),
    u!(
        "kilograms per hour",
        ["kilogram per hour", "kg/h", "kgph"],
        MassFlow,
        1.0 / 3600.0
    ),
    u!(
        "pounds per hour",
        ["pound per hour", "pph", "lb/h"],
        MassFlow,
        0.453_592_37 / 3600.0
    ),
    u!(
        "pounds per second",
        ["pound per second"],
        MassFlow,
        0.453_592_37
    ),
    // Volume flow (base: m³/s)
    u!(
        "gallons per hour",
        ["gallon per hour", "gph"],
        VolumeFlow,
        0.003_785_411_784 / 3600.0
    ),
    u!(
        "liters per hour",
        ["liter per hour"],
        VolumeFlow,
        0.001 / 3600.0
    ),
    // Electrical
    u!("volts", ["volt", "v"], Voltage, 1.0),
    u!("amperes", ["ampere", "amps", "amp", "a"], Current, 1.0),
    u!("watts", ["watt", "w"], Power, 1.0),
    u!("kilowatts", ["kilowatt", "kw"], Power, 1000.0),
    u!("horsepower", ["hp"], Power, 745.699_872),
    u!(
        "ft lb per second",
        ["foot pound per second"],
        Power,
        1.355_818
    ),
    // Density (base: kg/m³)
    u!(
        "kilograms per cubic meter",
        ["kilogram per cubic meter"],
        Density,
        1.0
    ),
    u!(
        "slugs per cubic feet",
        ["slug per cubic foot", "slugs per cubic foot"],
        Density,
        515.378_818
    ),
    // Torque (base: N·m)
    u!("newton meters", ["newton meter", "n m"], Torque, 1.0),
    u!(
        "foot pounds",
        ["foot pound", "ft-lbs", "ft lbs", "foot-pounds"],
        Torque,
        1.355_818
    ),
];

/// All known units.
pub fn all() -> &'static [Unit] {
    UNITS
}

fn normalize(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

/// Looks up a unit by canonical name or alias (case-insensitive, whitespace-collapsed).
pub fn lookup(name: &str) -> Option<&'static Unit> {
    let n = normalize(name);
    UNITS
        .iter()
        .find(|u| u.name == n || u.aliases.contains(&n.as_str()))
}

/// Converts `value` between two units of the same dimension.
pub fn convert(value: f64, from: &Unit, to: &Unit) -> Result<f64, UnitError> {
    if from.dimension != to.dimension {
        return Err(UnitError::DimensionMismatch {
            from: from.name,
            from_dim: from.dimension.as_str(),
            to: to.name,
            to_dim: to.dimension.as_str(),
        });
    }
    if std::ptr::eq(from, to) {
        return Ok(value);
    }
    Ok(to.from_base(from.to_base(value)))
}

/// Converts between unit names.
pub fn convert_named(value: f64, from: &str, to: &str) -> Result<f64, UnitError> {
    let f = lookup(from).ok_or_else(|| UnitError::Unknown(from.to_string()))?;
    let t = lookup(to).ok_or_else(|| UnitError::Unknown(to.to_string()))?;
    convert(value, f, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6 * b.abs().max(1.0)
    }

    #[test]
    fn names_and_aliases_are_unique_and_normalized() {
        let mut seen = std::collections::HashSet::new();
        for u in UNITS {
            assert_eq!(
                u.name,
                normalize(u.name),
                "canonical name must be normalized: {}",
                u.name
            );
            assert!(seen.insert(u.name), "duplicate name {}", u.name);
            for a in u.aliases {
                assert_eq!(*a, normalize(a), "alias must be normalized: {a}");
                assert!(seen.insert(a), "duplicate alias {a}");
            }
        }
    }

    #[test]
    fn lookup_is_case_and_space_insensitive() {
        assert_eq!(lookup("Knots").unwrap().name, "knots");
        assert_eq!(
            lookup("  Feet   per  Minute ").unwrap().name,
            "feet per minute"
        );
        assert_eq!(lookup("Millibars").unwrap().name, "millibars");
        assert_eq!(lookup("inHg").unwrap().name, "inches of mercury");
        assert_eq!(lookup("Bool").unwrap().dimension, Dimension::Bool);
        assert_eq!(lookup("NM").unwrap().name, "nautical miles");
        assert!(lookup("furlongs").is_none());
    }

    #[test]
    fn conversions() {
        assert!(close(
            convert_named(1013.25, "hPa", "inHg").unwrap(),
            29.921_26
        ));
        assert!(close(convert_named(100.0, "knots", "kph").unwrap(), 185.2));
        assert!(close(convert_named(1000.0, "fpm", "m/s").unwrap(), 5.08));
        assert!(close(
            convert_named(15.0, "celsius", "fahrenheit").unwrap(),
            59.0
        ));
        assert!(close(
            convert_named(-40.0, "fahrenheit", "celsius").unwrap(),
            -40.0
        ));
        assert!(close(
            convert_named(0.0, "celsius", "kelvin").unwrap(),
            273.15
        ));
        assert!(close(
            convert_named(PI, "radians", "degrees").unwrap(),
            180.0
        ));
        assert!(close(
            convert_named(1000.0, "kg", "lbs").unwrap(),
            2_204.622_6
        ));
        assert!(close(
            convert_named(50.0, "percent", "percent over 100").unwrap(),
            0.5
        ));
    }

    #[test]
    fn dimension_mismatch_is_an_error() {
        let err = convert_named(1.0, "knots", "feet").unwrap_err();
        assert_eq!(
            err.to_string(),
            "cannot convert `knots` (speed) to `feet` (length)"
        );
        assert!(matches!(
            convert_named(1.0, "mach", "knots"),
            Err(UnitError::DimensionMismatch { .. })
        ));
        assert_eq!(
            convert_named(1.0, "parsecs", "feet").unwrap_err(),
            UnitError::Unknown("parsecs".into())
        );
    }
}
