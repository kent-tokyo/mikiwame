//! Provenance: what produced a report, and with what reference data.

use serde::{Deserialize, Serialize};

/// Records what produced a [`crate::MaterialDiagnosticReport`] and which
/// versioned reference data (if any) it used, so results are reproducible and
/// auditable (AGENTS.md §16, §19).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Provenance {
    /// The `mikiwame` crate version that produced this report.
    pub mikiwame_version: String,
    /// Whether the analysis was fully deterministic (v0.1 always is: no
    /// randomness is used).
    pub deterministic: bool,
    /// Version of the elemental-radius table used for distance checks, if
    /// any were run. `None` in v0.1: no radius-based check ships yet (see
    /// `tasks/todo.md`).
    pub radius_table_version: Option<String>,
    /// Version of the formal oxidation-state table used, if composition
    /// checks were run. `None` in v0.1: not implemented yet.
    pub oxidation_table_version: Option<String>,
    /// Name of the coordination-environment method used, if coordination
    /// checks were run. `None` in v0.1: not implemented yet.
    pub coordination_method: Option<String>,
}

impl Provenance {
    /// Builds provenance for the current crate version. `radius_table_version`
    /// and `coordination_method` should be `Some` exactly when a check that
    /// consumed that reference data actually ran (`None` otherwise, rather
    /// than a value nothing used) — see `lib.rs::build_report`'s caller.
    pub fn current(
        radius_table_version: Option<String>,
        coordination_method: Option<String>,
    ) -> Self {
        Self {
            mikiwame_version: env!("CARGO_PKG_VERSION").to_string(),
            deterministic: true,
            radius_table_version,
            oxidation_table_version: None,
            coordination_method,
        }
    }
}
