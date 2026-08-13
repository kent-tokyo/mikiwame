//! Occupancy and disorder checks (AGENTS.md §7.7) — the no-threshold subset.
//!
//! Detects sites of different elements coinciding under PBC (using the same
//! numerical-identity tolerance as `separation::check`'s duplicate
//! detection) as a disorder group — informational, not itself an anomaly
//! (AGENTS.md §7.7: "disorderをエラー扱いせず"). A group whose occupancies
//! sum to more than 1.0 *is* flagged: a site cannot be more than fully
//! occupied, which is a logical fact needing no external threshold, unlike
//! `SITE_SEVERE_OVERLAP` (deferred; see `tasks/todo.md`).

use std::collections::{HashMap, HashSet};

use crate::finding::{Evidence, Finding, FindingCode, FindingScope, NumericEvidence};
use crate::model::{ClosedRange, MetricCode, Score01, Severity, Unit};
use crate::structure_view::{PeriodicStructureView, minimum_image_distance};

// Numerical-identity tolerance (float round-trip noise), same value and
// justification as separation::DUPLICATE_TOLERANCE_ANGSTROM. Kept as its own
// constant since the two components are independent.
const COINCIDENCE_TOLERANCE_ANGSTROM: f64 = 1e-6;

fn certain() -> Score01 {
    Score01::new(1.0).expect("1.0 is a valid Score01")
}

fn find(parent: &mut [usize], x: usize) -> usize {
    if parent[x] != x {
        parent[x] = find(parent, parent[x]);
    }
    parent[x]
}

/// Groups of site indices that mutually coincide under PBC, by transitive
/// closure over pairwise coincidence.
///
/// ponytail: O(n^2) pairwise scan plus union-find; fine at v0.1's scale
/// (matches separation::check's own O(n^2) scan), revisit if mikiwame is
/// used on structures with many thousands of sites.
fn coincidence_groups<S: PeriodicStructureView>(structure: &S) -> Vec<Vec<usize>> {
    let lattice = structure.lattice();
    let sites = structure.sites();
    let n = sites.len();
    let mut parent: Vec<usize> = (0..n).collect();

    for i in 0..n {
        for j in (i + 1)..n {
            let distance =
                minimum_image_distance(lattice, sites[i].fractional, sites[j].fractional);
            if distance < COINCIDENCE_TOLERANCE_ANGSTROM {
                let (root_i, root_j) = (find(&mut parent, i), find(&mut parent, j));
                if root_i != root_j {
                    parent[root_i] = root_j;
                }
            }
        }
    }

    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..n {
        let root = find(&mut parent, i);
        groups.entry(root).or_default().push(i);
    }
    groups.into_values().filter(|g| g.len() > 1).collect()
}

pub(crate) fn check<S: PeriodicStructureView>(structure: &S) -> Vec<Finding> {
    let sites = structure.sites();
    let mut findings = Vec::new();

    for group in coincidence_groups(structure) {
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
            limitations: Vec::new(),
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
                    limitations: Vec::new(),
                });
            }
        }
        // Non-finite occupancy in the group is already reported as
        // INPUT_INVALID_OCCUPANCY by input_quality::check; a NaN sum here
        // would add nothing (and must not leak into a NumericEvidence).
    }

    findings
}
