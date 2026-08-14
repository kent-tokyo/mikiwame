//! Occupancy and disorder checks (AGENTS.md §7.7) — the no-threshold subset.
//!
//! Detects sites of different elements coinciding under PBC (using the same
//! numerical-identity tolerance as `separation::check`'s duplicate
//! detection) as a disorder group — informational, not itself an anomaly
//! (AGENTS.md §7.7: "disorderをエラー扱いせず"). A group whose occupancies
//! sum to more than 1.0 *is* flagged: a site cannot be more than fully
//! occupied, which is a logical fact needing no external threshold, unlike
//! `SITE_SEVERE_OVERLAP` (deferred; see `tasks/todo.md`).
//!
//! Findings here keep full confidence even when `coincidence_groups` used
//! `structure_view`'s approximate fallback distance method (recorded in
//! `limitations` instead, same reasoning as `separation::check`): the
//! fallback's naive per-axis-rounded distance is always a real, achievable
//! periodic separation, never smaller than the true minimum — so any group
//! it *does* merge is a genuine coincidence. Its risk is a group that
//! *should* have merged but didn't (a missed `DISORDER_PRESENT`, or an
//! occupancy sum computed over too few sites to trip
//! `DISORDER_OCCUPANCY_SUM_EXCEEDS_ONE`), which is a false negative with no
//! finding to attach a lowered confidence to.

use std::collections::HashSet;

use crate::finding::{Evidence, Finding, FindingCode, FindingScope, NumericEvidence};
use crate::model::{ClosedRange, MetricCode, Score01, Severity, Unit};
use crate::structure_view::{PeriodicStructureView, coincidence_groups};

// Numerical-identity tolerance (float round-trip noise), same value and
// justification as separation::DUPLICATE_TOLERANCE_ANGSTROM. Kept as its own
// constant since the two components are independent.
const COINCIDENCE_TOLERANCE_ANGSTROM: f64 = 1e-6;

fn certain() -> Score01 {
    Score01::new(1.0).expect("1.0 is a valid Score01")
}

pub(crate) fn check<S: PeriodicStructureView>(structure: &S) -> Vec<Finding> {
    let sites = structure.sites();
    let mut findings = Vec::new();
    let (groups, fallback_limitation) =
        coincidence_groups(structure, COINCIDENCE_TOLERANCE_ANGSTROM);

    // Singleton groups (an ordinary site with no coincidence partner) and
    // same-element-only groups (separation::check's SITE_DUPLICATE, not
    // disorder) aren't this component's concern.
    for group in groups.into_iter().filter(|g| g.len() > 1) {
        let distinct_elements: HashSet<&str> =
            group.iter().map(|&i| sites[i].element.as_str()).collect();
        if distinct_elements.len() < 2 {
            // Same-element coincidence is separation::check's SITE_DUPLICATE,
            // not disorder.
            continue;
        }

        findings.push(Finding {
            code: FindingCode::DisorderPresent,
            severity: Severity::Info,
            confidence: certain(),
            scope: FindingScope::SiteGroup {
                indices: group.clone(),
            },
            evidence: Vec::new(),
            explanation: format!(
                "sites {group:?} ({} distinct elements) coincide under periodic boundary conditions: modeled as positional disorder, not an anomaly",
                distinct_elements.len()
            ),
            limitations: fallback_limitation.clone().into_iter().collect(),
        });

        let occupancies: Vec<f64> = group.iter().map(|&i| sites[i].occupancy).collect();
        if occupancies.iter().all(|o| o.is_finite()) {
            let occupancy_sum: f64 = occupancies.iter().sum();
            if occupancy_sum > 1.0 + 1e-9 {
                findings.push(Finding {
                    code: FindingCode::DisorderOccupancySumExceedsOne,
                    severity: Severity::High,
                    confidence: certain(),
                    scope: FindingScope::SiteGroup {
                        indices: group.clone(),
                    },
                    evidence: vec![Evidence::Numeric(NumericEvidence {
                        metric: MetricCode::Occupancy,
                        observed: occupancy_sum,
                        expected_range: Some(
                            ClosedRange::new(0.0, 1.0).expect("0.0..=1.0 is a valid ClosedRange"),
                        ),
                        threshold: None,
                        unit: Some(Unit::Dimensionless),
                        site_indices: group,
                    })],
                    explanation: format!(
                        "disordered site group's occupancies sum to {occupancy_sum}, which exceeds 1.0"
                    ),
                    limitations: fallback_limitation.clone().into_iter().collect(),
                });
            }
        }
        // Non-finite occupancy in the group is already reported as
        // INPUT_INVALID_OCCUPANCY by input_quality::check; a NaN sum here
        // would add nothing (and must not leak into a NumericEvidence).
    }

    findings
}
