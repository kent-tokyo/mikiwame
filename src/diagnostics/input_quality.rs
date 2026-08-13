//! Input quality checks (AGENTS.md §7.1) — the no-threshold subset only.
//!
//! `LATTICE_SINGULAR` lives here rather than in a separate `lattice` module
//! because AGENTS.md §7.1 itself lists "singular/near-singular lattice" and
//! "non-positive cell volume" as input-quality checks, distinct from the
//! aspect-ratio/conditioning checks of §7.2 (not implemented: every one of
//! those needs an invented "extreme" cutoff).

use crate::finding::{Evidence, Finding, FindingCode, FindingScope, NumericEvidence};
use crate::model::{MetricCode, Score01, Severity, Unit};
use crate::structure_view::{PeriodicStructureView, cell_volume};

/// Result of running input-quality checks.
pub(crate) struct Outcome {
    pub(crate) findings: Vec<Finding>,
    /// Whether later components must be skipped: geometry (lattice matrix or
    /// any site's fractional coordinates) could not be trusted enough to
    /// compute distances from. Invalid occupancy does *not* set this: it
    /// does not affect geometry.
    pub(crate) fatal: bool,
}

fn certain() -> Score01 {
    Score01::new(1.0).expect("1.0 is a valid Score01")
}

fn finite3(v: [f64; 3]) -> bool {
    v.iter().all(|x| x.is_finite())
}

pub(crate) fn check<S: PeriodicStructureView>(structure: &S) -> Outcome {
    let mut findings = Vec::new();
    let mut fatal = false;

    if structure.sites().is_empty() {
        findings.push(Finding {
            code: FindingCode::InputEmptyStructure,
            severity: Severity::Critical,
            confidence: certain(),
            scope: FindingScope::WholeStructure,
            evidence: Vec::new(),
            explanation: "the structure has no sites".to_string(),
            limitations: Vec::new(),
        });
        return Outcome {
            findings,
            fatal: true,
        };
    }

    let lattice = structure.lattice();
    let lattice_finite = lattice.iter().all(|row| finite3(*row));
    if !lattice_finite {
        findings.push(Finding {
            code: FindingCode::InputNonfiniteLattice,
            severity: Severity::Critical,
            confidence: certain(),
            scope: FindingScope::Lattice,
            evidence: Vec::new(),
            explanation: "the lattice matrix contains a non-finite value".to_string(),
            limitations: Vec::new(),
        });
        fatal = true;
    } else {
        let volume = cell_volume(lattice);
        if volume <= 0.0 {
            findings.push(Finding {
                code: FindingCode::LatticeSingular,
                severity: Severity::Critical,
                confidence: certain(),
                scope: FindingScope::Lattice,
                evidence: vec![Evidence::Numeric(NumericEvidence {
                    metric: MetricCode::CellVolume,
                    observed: volume,
                    expected_range: None,
                    threshold: Some(0.0),
                    unit: Some(Unit::CubicAngstrom),
                    site_indices: Vec::new(),
                })],
                explanation: format!(
                    "unit cell volume is non-positive ({volume:.6} \u{c5}\u{b3}); the lattice is singular or has inverted handedness"
                ),
                limitations: Vec::new(),
            });
            fatal = true;
        }
    }

    for (index, site) in structure.sites().iter().enumerate() {
        if !finite3(site.fractional) {
            findings.push(Finding {
                code: FindingCode::InputNonfiniteCoordinate,
                severity: Severity::Critical,
                confidence: certain(),
                scope: FindingScope::Site { index },
                evidence: Vec::new(),
                explanation: format!("site {index} has a non-finite fractional coordinate"),
                limitations: Vec::new(),
            });
            fatal = true;
        }

        let occ = site.occupancy;
        if !occ.is_finite() {
            findings.push(Finding {
                code: FindingCode::InputInvalidOccupancy,
                severity: Severity::High,
                confidence: certain(),
                scope: FindingScope::Site { index },
                evidence: Vec::new(),
                explanation: format!("site {index} has a non-finite occupancy"),
                limitations: Vec::new(),
            });
        } else if !(0.0..=1.0).contains(&occ) {
            findings.push(Finding {
                code: FindingCode::InputInvalidOccupancy,
                severity: Severity::High,
                confidence: certain(),
                scope: FindingScope::Site { index },
                evidence: vec![Evidence::Numeric(NumericEvidence {
                    metric: MetricCode::Occupancy,
                    observed: occ,
                    expected_range: Some(
                        crate::model::ClosedRange::new(0.0, 1.0)
                            .expect("0.0..=1.0 is a valid ClosedRange"),
                    ),
                    threshold: None,
                    unit: Some(Unit::Dimensionless),
                    site_indices: vec![index],
                })],
                explanation: format!(
                    "site {index} occupancy {occ} is outside the valid range 0.0..=1.0"
                ),
                limitations: Vec::new(),
            });
        }
    }

    Outcome { findings, fatal }
}
