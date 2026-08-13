//! Shared value types used across findings and reports.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::MikiwameError;

/// A score constrained to the closed interval `0.0..=1.0`, guaranteed finite.
///
/// Distinct `Score01` values are used for unrelated concepts (anomaly burden,
/// confidence, per-finding confidence) — AGENTS.md §6 requires these never be
/// collapsed into one number, so this type deliberately carries no semantics of
/// its own beyond "a valid probability-like value".
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Score01(f64);

impl Score01 {
    /// Builds a `Score01`, rejecting non-finite or out-of-range values.
    pub fn new(value: f64) -> Result<Self, MikiwameError> {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(MikiwameError::InvalidScore { value })
        }
    }

    /// Returns the underlying value.
    pub fn get(self) -> f64 {
        self.0
    }
}

impl Serialize for Score01 {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0)
    }
}

impl<'de> Deserialize<'de> for Score01 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = f64::deserialize(deserializer)?;
        Score01::new(value).map_err(serde::de::Error::custom)
    }
}

/// A closed numeric range `[min, max]`, guaranteed finite with `min <= max`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClosedRange {
    min: f64,
    max: f64,
}

impl ClosedRange {
    /// Builds a `ClosedRange`, rejecting non-finite bounds or `min > max`.
    pub fn new(min: f64, max: f64) -> Result<Self, MikiwameError> {
        if min.is_finite() && max.is_finite() && min <= max {
            Ok(Self { min, max })
        } else {
            Err(MikiwameError::InvalidRange { min, max })
        }
    }

    /// Returns the lower bound.
    pub fn min(self) -> f64 {
        self.min
    }

    /// Returns the upper bound.
    pub fn max(self) -> f64 {
        self.max
    }

    /// Returns whether `value` falls within `[min, max]`.
    pub fn contains(self, value: f64) -> bool {
        value >= self.min && value <= self.max
    }
}

impl Serialize for ClosedRange {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("ClosedRange", 2)?;
        s.serialize_field("min", &self.min)?;
        s.serialize_field("max", &self.max)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for ClosedRange {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            min: f64,
            max: f64,
        }
        let raw = Raw::deserialize(deserializer)?;
        ClosedRange::new(raw.min, raw.max).map_err(serde::de::Error::custom)
    }
}

/// The top-level, headline judgment of a [`crate::MaterialDiagnosticReport`].
///
/// None of these variants are a claim about thermodynamic stability or
/// synthesizability — mikiwame does not compute either (AGENTS.md §2, §17).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    /// No structural anomaly was found by the checks that ran. Not a claim of
    /// stability, synthesizability, or physical correctness.
    StructurallyConsistent,
    /// At least one moderate-severity finding warrants a human look.
    ReviewRecommended,
    /// At least one high-confidence, high-severity structural anomaly was found.
    StrongAnomalyDetected,
    /// The structure falls outside mikiwame's validated structural domain (see
    /// `docs/scientific_scope.md`); other findings may still be informative but
    /// are not accuracy-guaranteed.
    OutOfDomain,
    /// The input could not be validated well enough to run diagnostics; see the
    /// report's findings and skipped components for why.
    InvalidInput,
}

/// How severe a single finding is, independent of how confident mikiwame is
/// that the finding is correct (see [`crate::Finding::confidence`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Severity {
    /// Worth surfacing but not indicative of a problem on its own.
    Info,
    /// A minor, likely benign deviation.
    Low,
    /// A deviation worth a closer look.
    Medium,
    /// A serious, likely-real structural problem.
    High,
    /// A structural or input problem severe enough to block further diagnosis.
    Critical,
}

/// How applicable mikiwame's checks are to the structure that was analyzed.
///
/// Kept separate from [`Verdict`]: a structure can be fully applicable and
/// consistent, or out of domain regardless of what its (not accuracy-guaranteed)
/// findings say. See AGENTS.md §6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ApplicabilityLevel {
    /// Squarely within the v0.1 validated domain (see `docs/scientific_scope.md`).
    FullyApplicable,
    /// Within domain but with a known accuracy caveat (e.g. small-molecule
    /// crystals, MOFs — readable but not accuracy-guaranteed per AGENTS.md §5).
    PartiallyApplicable,
    /// Outside the validated domain; findings, if any, are exploratory only.
    LimitedApplicability,
    /// Could not be assessed for applicability at all (e.g. invalid input).
    NotApplicable,
}

/// Identifies which numeric quantity a [`crate::NumericEvidence`] reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MetricCode {
    /// Unit cell volume, computed as the scalar triple product of the lattice
    /// row vectors.
    CellVolume,
    /// A minimum-image (PBC-aware) distance between two sites, in Angstrom.
    PeriodicDistance,
    /// A site's fractional occupancy.
    Occupancy,
}

/// A physical unit attached to a [`crate::NumericEvidence`] value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Unit {
    /// Angstrom (Å), used for lengths/distances.
    Angstrom,
    /// Cubic Angstrom (Å³), used for cell volume.
    CubicAngstrom,
    /// No physical unit (e.g. occupancy, a ratio).
    Dimensionless,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Angstrom => "\u{c5}",
            Self::CubicAngstrom => "\u{c5}\u{b3}",
            Self::Dimensionless => "",
        })
    }
}
