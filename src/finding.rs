//! Machine-readable findings: the core unit of explanation mikiwame produces.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::model::{ClosedRange, MetricCode, Score01, Severity, Unit};

/// A stable, machine-readable finding identifier.
///
/// Per AGENTS.md §8: meaning is never changed once shipped, only added to
/// (hence `#[non_exhaustive]`); severity is never encoded in the name itself
/// (that lives in [`Finding::severity`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FindingCode {
    /// The structure has no sites.
    #[serde(rename = "INPUT_EMPTY_STRUCTURE")]
    InputEmptyStructure,
    /// The lattice matrix contains a non-finite (NaN/±inf) value.
    #[serde(rename = "INPUT_NONFINITE_LATTICE")]
    InputNonfiniteLattice,
    /// A site's fractional coordinate contains a non-finite value.
    #[serde(rename = "INPUT_NONFINITE_COORDINATE")]
    InputNonfiniteCoordinate,
    /// A site's occupancy is non-finite or outside `[0.0, 1.0]`.
    #[serde(rename = "INPUT_INVALID_OCCUPANCY")]
    InputInvalidOccupancy,
    /// A site's element symbol is not a recognized periodic-table symbol.
    #[serde(rename = "INPUT_UNKNOWN_ELEMENT")]
    InputUnknownElement,
    /// The lattice is singular or has non-positive volume, so periodicity is
    /// undefined or degenerate.
    #[serde(rename = "LATTICE_SINGULAR")]
    LatticeSingular,
    /// Two sites of the same element occupy the same position (within
    /// numerical tolerance) under periodic boundary conditions.
    #[serde(rename = "SITE_DUPLICATE")]
    SiteDuplicate,
    /// Two or more sites of different elements coincide under periodic
    /// boundary conditions, modeled as positional disorder. Informational —
    /// disorder is not itself an anomaly (AGENTS.md §7.7).
    #[serde(rename = "DISORDER_PRESENT")]
    DisorderPresent,
    /// A disordered site group's occupancies sum to more than 1.0 — a site
    /// cannot be more than fully occupied.
    #[serde(rename = "DISORDER_OCCUPANCY_SUM_EXCEEDS_ONE")]
    DisorderOccupancySumExceedsOne,
}

impl FindingCode {
    /// Returns the stable `SCREAMING_SNAKE_CASE` code string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InputEmptyStructure => "INPUT_EMPTY_STRUCTURE",
            Self::InputNonfiniteLattice => "INPUT_NONFINITE_LATTICE",
            Self::InputNonfiniteCoordinate => "INPUT_NONFINITE_COORDINATE",
            Self::InputInvalidOccupancy => "INPUT_INVALID_OCCUPANCY",
            Self::InputUnknownElement => "INPUT_UNKNOWN_ELEMENT",
            Self::LatticeSingular => "LATTICE_SINGULAR",
            Self::SiteDuplicate => "SITE_DUPLICATE",
            Self::DisorderPresent => "DISORDER_PRESENT",
            Self::DisorderOccupancySumExceedsOne => "DISORDER_OCCUPANCY_SUM_EXCEEDS_ONE",
        }
    }
}

impl fmt::Display for FindingCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What part of the structure a [`Finding`] is about.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FindingScope {
    /// The structure as a whole.
    WholeStructure,
    /// The lattice/unit cell.
    Lattice,
    /// A single site, by index into the structure's site list.
    Site {
        /// Index into the structure's site list.
        index: usize,
    },
    /// A pair of sites, by index.
    SitePair {
        /// Index of the first site.
        a: usize,
        /// Index of the second site.
        b: usize,
    },
    /// A group of more than two related sites, by index (e.g. a disorder
    /// group of three or more coincident, differently-occupied sites).
    SiteGroup {
        /// Indices into the structure's site list.
        indices: Vec<usize>,
    },
}

/// A single machine-readable numeric measurement backing a [`Finding`].
///
/// `observed` is guaranteed finite: mikiwame never puts NaN/±inf into a public
/// report (AGENTS.md §6). Findings about non-finite input values carry no
/// [`Evidence`] for that reason — the [`Finding::scope`] and
/// [`Finding::explanation`] carry the diagnosis instead.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NumericEvidence {
    /// Which quantity this is.
    pub metric: MetricCode,
    /// The measured value.
    pub observed: f64,
    /// The range this value was expected to fall within, if applicable.
    pub expected_range: Option<ClosedRange>,
    /// The threshold that was crossed, if this finding is threshold-based.
    pub threshold: Option<f64>,
    /// The unit `observed` is measured in.
    pub unit: Option<Unit>,
    /// Indices of the site(s) this measurement is about.
    pub site_indices: Vec<usize>,
}

/// A piece of evidence backing a [`Finding`].
///
/// Only one variant exists today; this is an enum (not `NumericEvidence`
/// directly) so non-numeric evidence kinds can be added later without
/// breaking [`Finding::evidence`]'s element type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Evidence {
    /// A numeric measurement.
    Numeric(NumericEvidence),
}

/// One machine-readable diagnostic finding, with its supporting evidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Finding {
    /// The stable finding identifier.
    pub code: FindingCode,
    /// How severe this finding is.
    pub severity: Severity,
    /// How confident mikiwame is that this specific finding is correct (not
    /// to be confused with the report-level `OverallAssessment::confidence`).
    pub confidence: Score01,
    /// What part of the structure this finding is about.
    pub scope: FindingScope,
    /// Machine-readable evidence, if any could be constructed (see
    /// [`NumericEvidence`]'s doc comment for why this can be empty).
    pub evidence: Vec<Evidence>,
    /// A human-readable explanation of the finding.
    pub explanation: String,
    /// Known limitations of this specific finding (e.g. simplifying
    /// assumptions in the check that produced it).
    pub limitations: Vec<String>,
}
