//! Site separation checks (AGENTS.md §7.3) — the no-threshold subset only.
//!
//! Only exact (same-element, same-position-under-PBC) duplicate detection
//! ships in v0.1. `SITE_SEVERE_OVERLAP` / `SITE_UNUSUALLY_SHORT_DISTANCE` need
//! an elemental-radius table with a recorded source before they can carry a
//! threshold honestly (AGENTS.md §21); see `tasks/todo.md`.

use crate::finding::{Evidence, Finding, FindingCode, FindingScope, NumericEvidence};
use crate::model::{MetricCode, Score01, Severity, Unit};
use crate::structure_view::{PeriodicStructureView, ResolvedLattice};

// ponytail: numerical-identity tolerance (float round-trip noise), not a
// chemistry judgment about how close atoms may physically sit — that is
// SITE_SEVERE_OVERLAP, deferred until a radius table exists.
const DUPLICATE_TOLERANCE_ANGSTROM: f64 = 1e-6;

fn certain() -> Score01 {
    Score01::new(1.0).expect("1.0 is a valid Score01")
}

pub(crate) fn check<S: PeriodicStructureView>(structure: &S) -> Vec<Finding> {
    // Resolved once, not once per pair -- see ResolvedLattice's doc comment.
    let resolved_lattice = ResolvedLattice::resolve(structure.lattice());
    let sites = structure.sites();
    let mut findings = Vec::new();

    for i in 0..sites.len() {
        for j in (i + 1)..sites.len() {
            if sites[i].element != sites[j].element {
                continue;
            }
            let distance = resolved_lattice.minimum_image(sites[i].fractional, sites[j].fractional);
            if distance.distance_angstrom < DUPLICATE_TOLERANCE_ANGSTROM {
                findings.push(Finding {
                    code: FindingCode::SiteDuplicate,
                    severity: Severity::Critical,
                    // Not lowered on the fallback path: a distance this
                    // method reports as below tolerance is a genuine
                    // periodic image at that separation either way (see
                    // docs/validation.md's "fallback confidence" note) —
                    // the fallback's risk is a missed finding (false
                    // negative, nothing to attach confidence to), not a
                    // wrong one.
                    confidence: certain(),
                    scope: FindingScope::SitePair { a: i, b: j },
                    evidence: vec![Evidence::Numeric(NumericEvidence {
                        metric: MetricCode::PeriodicDistance,
                        observed: distance.distance_angstrom,
                        expected_range: None,
                        threshold: Some(DUPLICATE_TOLERANCE_ANGSTROM),
                        unit: Some(Unit::Angstrom),
                        site_indices: vec![i, j],
                    })],
                    explanation: format!(
                        "sites {i} and {j} (both {}) coincide under periodic boundary conditions (separation {:.3e} \u{c5})",
                        sites[i].element, distance.distance_angstrom
                    ),
                    limitations: distance.method.limitation().into_iter().collect(),
                });
            }
        }
    }

    findings
}
