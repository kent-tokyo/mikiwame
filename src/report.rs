//! The public report contract: [`MaterialDiagnosticReport`] and its parts.

use serde::{Deserialize, Serialize};

use crate::finding::{Finding, FindingCode};
use crate::model::{ApplicabilityLevel, Score01, Verdict};
use crate::provenance::Provenance;

/// The current report schema version. Bump only on a breaking change to the
/// report's shape or the meaning of an existing field (AGENTS.md §8, §19).
pub const SCHEMA_VERSION: u32 = 1;

/// A basic summary of the structure that was analyzed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InputSummary {
    /// Number of sites in the structure.
    pub site_count: usize,
    /// Number of distinct element symbols across all sites.
    pub distinct_element_count: usize,
}

/// The headline judgment for a structure, kept separate from applicability,
/// per-finding detail, and provenance (AGENTS.md §6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OverallAssessment {
    /// The headline verdict.
    pub verdict: Verdict,
    /// Overall anomaly burden, if a scientifically-grounded way to compute it
    /// was available. `None` in v0.1: AGENTS.md §9 forbids inventing a
    /// weighted-sum score without validation, so this is left unset rather
    /// than guessed.
    pub anomaly_burden: Option<Score01>,
    /// How confident mikiwame is in `verdict`, independent of `anomaly_burden`.
    pub confidence: Score01,
    /// The finding codes that most influenced `verdict`.
    pub dominant_findings: Vec<FindingCode>,
}

/// How applicable mikiwame's checks were to this structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ApplicabilityAssessment {
    /// The applicability bucket.
    pub level: ApplicabilityLevel,
    /// Human-readable reasons for `level`.
    pub reasons: Vec<String>,
}

/// Identifies a diagnostic component (a self-contained group of checks).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ComponentName {
    /// Structural/numerical input validity checks (AGENTS.md §7.1).
    InputQuality,
    /// Site separation / duplicate-site checks (AGENTS.md §7.3, partial).
    SiteSeparation,
}

/// Whether a diagnostic component ran.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ComponentStatus {
    /// The component ran to completion.
    Ran,
    /// The component did not run.
    Skipped {
        /// Why it was skipped.
        reason: String,
    },
}

/// Whether a diagnostic component ran, and on what.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ComponentAssessment {
    /// Which component this is.
    pub name: ComponentName,
    /// Whether it ran.
    pub status: ComponentStatus,
}

/// A human-readable, non-binding suggestion tied to one or more findings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Suggestion {
    /// The suggestion text.
    pub message: String,
    /// The findings this suggestion responds to.
    pub related_findings: Vec<FindingCode>,
}

/// The complete, explainable diagnostic report mikiwame produces for one
/// structure. See AGENTS.md §6 for the design rationale: no single opaque
/// score, anomaly/confidence/applicability/input-quality always separate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MaterialDiagnosticReport {
    /// The report schema version ([`SCHEMA_VERSION`] at the time of writing).
    pub schema_version: u32,
    /// A summary of the input structure.
    pub input: InputSummary,
    /// The headline verdict and its confidence.
    pub overall: OverallAssessment,
    /// Whether mikiwame's checks are applicable to this structure at all.
    pub applicability: ApplicabilityAssessment,
    /// Which diagnostic components ran or were skipped, and why.
    pub components: Vec<ComponentAssessment>,
    /// Every finding produced across all components that ran.
    pub findings: Vec<Finding>,
    /// Non-binding suggestions derived from `findings`. Always empty in v0.1:
    /// no suggestion-generation logic has been implemented yet.
    pub suggestions: Vec<Suggestion>,
    /// What produced this report and with what reference data.
    pub provenance: Provenance,
}
