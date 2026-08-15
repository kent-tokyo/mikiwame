//! The public report contract: [`MaterialDiagnosticReport`] and its parts.

use serde::{Deserialize, Serialize};

use crate::finding::{Finding, FindingCode};
use crate::model::{ApplicabilityLevel, Score01, Verdict};
use crate::provenance::Provenance;

/// The current report schema version. Bump only on a breaking change to the
/// report's shape or the meaning of an existing field (AGENTS.md §8, §19).
///
/// `2`: added [`MaterialDiagnosticReport::local_environment`].
/// `3`: added [`SiteLocalEnvironment::neighbors`] (per-neighbor detail:
/// which sites, at what distance, via which periodic image, and whether
/// each candidate was actually counted toward `coordination_number`).
pub const SCHEMA_VERSION: u32 = 3;

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
    /// Occupancy and disorder checks (AGENTS.md §7.7, partial).
    Disorder,
    /// Coordination number / local environment checks (AGENTS.md §7.4).
    Coordination,
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

/// One neighbor species and how many times it appears in a site's resolved
/// coordination shell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NeighborSpeciesCount {
    /// Element symbol, e.g. `"Cl"`.
    pub element: String,
    /// How many neighbors of this element are in the shell.
    pub count: usize,
}

/// One candidate neighbor considered for a site's coordination shell — not
/// just the ones that ended up counted. `included_in_first_shell = false`
/// entries are exactly what [`SiteLocalEnvironment::shell_gap_ratio`]
/// measures separation against: what was just outside the resolved shell
/// boundary. Before schema version 3 this data was computed internally by
/// `diagnostics::coordination` and discarded before the report was built.
///
/// The unique key for a `NeighborRecord` within one site's `neighbors` list
/// is `(neighbor_site_index, image)`, not `neighbor_site_index` alone: the
/// same neighbor site can legitimately appear more than once, reached via
/// different periodic images (e.g. in a simple-cubic single-site primitive
/// cell, every neighbor *is* the center's own site, reached via six
/// different non-zero images). This is expected, not a duplicate to merge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NeighborRecord {
    /// Index into the structure's site list — the neighbor, not the center
    /// (see the containing [`SiteLocalEnvironment::site_index`] for that).
    /// Not unique across one site's `neighbors` list on its own; see this
    /// type's doc comment.
    pub neighbor_site_index: usize,
    /// Element symbol of the neighbor.
    pub element: String,
    /// Which periodic image of `neighbor_site_index` this specific neighbor
    /// instance is (matches `chematic_crystal::PeriodicNeighbor::image`).
    pub image: [i32; 3],
    /// Euclidean distance from the center to this neighbor image, in
    /// Angstrom.
    pub distance_angstrom: f64,
    /// The neighbor's occupancy, taken from the original input site (not
    /// from any internally-constructed geometry object, which may assign an
    /// artificial occupancy for validation purposes only) — may be less
    /// than `1.0` for a genuinely partially-occupied site. Not itself a
    /// signal that the neighbor is disordered: a *multi*-species (disorder)
    /// position is excluded from this list entirely (see
    /// `diagnostics::coordination`'s module doc comment), independent of
    /// whether the single species present is fully or partially occupied.
    pub occupancy: f64,
    /// Whether this candidate survived the largest-relative-gap step and is
    /// counted toward `coordination_number`. `false` entries are what
    /// `shell_gap_ratio` measures separation against.
    pub included_in_first_shell: bool,
}

/// Descriptive coordination/local-environment data for one site — not itself
/// an anomaly. `shell_gap_ratio` is the ambiguity signal (how cleanly
/// separated the resolved shell is from the next candidate) — v0.1 reports
/// it but does not yet turn a low value into a finding; see
/// `diagnostics::coordination`'s module doc comment and `tasks/todo.md`.
/// One entry exists per site whenever [`ComponentName::Coordination`] ran at
/// all; a site whose own coordination number couldn't be computed still gets
/// an entry, with `coordination_number: None` and `not_computed_reason`
/// explaining why (AGENTS.md §7.4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SiteLocalEnvironment {
    /// Index into the structure's site list.
    pub site_index: usize,
    /// Number of neighbors in the resolved first coordination shell, if one
    /// could be resolved.
    pub coordination_number: Option<usize>,
    /// Breakdown of `coordination_number` by neighbor element. Empty when
    /// `coordination_number` is `None`.
    pub neighbor_species: Vec<NeighborSpeciesCount>,
    /// Every candidate neighbor considered (within the radius-sum+epsilon
    /// search bound), not just the ones counted toward
    /// `coordination_number` — see [`NeighborRecord::included_in_first_shell`].
    /// Empty exactly when `coordination_number` is `None`. Sorted
    /// deterministically by distance, then `neighbor_site_index`, then
    /// `image`, then `element` — never left to depend on internal iteration
    /// order. `#[serde(default)]` so a schema-version-2 report (which never
    /// had this field) still deserializes into this type, as an empty list
    /// rather than an error.
    #[serde(default)]
    pub neighbors: Vec<NeighborRecord>,
    /// Ratio between the first excluded candidate's distance and the last
    /// included neighbor's distance: how clearly separated the resolved
    /// shell is from the next one (larger is less ambiguous). `None` when
    /// `coordination_number` is `None`, or when every candidate within the
    /// search radius was included (no next shell to compare against).
    pub shell_gap_ratio: Option<f64>,
    /// Why `coordination_number` is `None`, if it is.
    pub not_computed_reason: Option<String>,
    /// Caveats about this entry even when `coordination_number` is `Some`
    /// (e.g. a nearby candidate neighbor was excluded from consideration
    /// because its own position is disordered or its element unresolvable —
    /// see `diagnostics::coordination`'s module doc comment).
    pub limitations: Vec<String>,
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
    /// Descriptive per-site coordination/local-environment data — not
    /// findings, since this is present for a clean structure too (AGENTS.md
    /// §7.4). Empty if [`ComponentName::Coordination`] didn't run.
    pub local_environment: Vec<SiteLocalEnvironment>,
    /// Non-binding suggestions derived from `findings`. Always empty in v0.1:
    /// no suggestion-generation logic has been implemented yet.
    pub suggestions: Vec<Suggestion>,
    /// What produced this report and with what reference data.
    pub provenance: Provenance,
}
