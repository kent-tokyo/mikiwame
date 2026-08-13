//! Metamorphic / invariance checks (AGENTS.md §15.3): diagnostic results
//! should not depend on site order, choice of origin, whether fractional
//! coordinates are pre-wrapped into `[0,1)`, the lattice's orientation in
//! space, or whether the same structure is described as a larger supercell.
//! Where full equality isn't the right invariant, this documents exactly
//! what's allowed to differ (site indices inside findings; construction here
//! sticks to verdict + finding-code counts for that reason).

use std::collections::HashMap;

use mikiwame::{AnalysisConfig, Finding, FindingCode, OwnedStructure, Site, Verdict, analyze};

const A: f64 = 5.6402;

fn nacl_lattice() -> [[f64; 3]; 3] {
    [[A, 0.0, 0.0], [0.0, A, 0.0], [0.0, 0.0, A]]
}

fn nacl_sites() -> Vec<Site> {
    let na = |f: [f64; 3]| Site {
        element: "Na".to_string(),
        fractional: f,
        occupancy: 1.0,
    };
    let cl = |f: [f64; 3]| Site {
        element: "Cl".to_string(),
        fractional: f,
        occupancy: 1.0,
    };
    vec![
        na([0.0, 0.0, 0.0]),
        na([0.5, 0.5, 0.0]),
        na([0.5, 0.0, 0.5]),
        na([0.0, 0.5, 0.5]),
        cl([0.5, 0.5, 0.5]),
        cl([0.0, 0.0, 0.5]),
        cl([0.0, 0.5, 0.0]),
        cl([0.5, 0.0, 0.0]),
    ]
}

fn code_counts(findings: &[Finding]) -> HashMap<FindingCode, usize> {
    let mut counts = HashMap::new();
    for finding in findings {
        *counts.entry(finding.code).or_insert(0) += 1;
    }
    counts
}

/// A deliberately-broken variant of the NaCl fixture (one duplicate pair, one
/// invalid occupancy), used where the clean fixture would make an invariance
/// check trivially true regardless of whether the invariance actually holds.
fn broken_sites() -> Vec<Site> {
    let mut sites = nacl_sites();
    sites[1].fractional = sites[0].fractional; // Na onto Na
    sites[2].occupancy = 1.4;
    sites
}

#[test]
fn site_order_does_not_change_verdict_or_finding_codes() {
    let sites = broken_sites();
    let original = analyze(
        &OwnedStructure::new(nacl_lattice(), sites.clone()),
        &AnalysisConfig::default(),
    );

    let mut reversed = sites;
    reversed.reverse();
    let permuted = analyze(
        &OwnedStructure::new(nacl_lattice(), reversed),
        &AnalysisConfig::default(),
    );

    assert_eq!(original.overall.verdict, permuted.overall.verdict);
    assert_eq!(
        code_counts(&original.findings),
        code_counts(&permuted.findings)
    );
}

#[test]
fn origin_shift_does_not_change_verdict_or_finding_codes() {
    let shift = [0.137, -0.482, 0.913];
    let shifted: Vec<Site> = nacl_sites()
        .into_iter()
        .map(|mut site| {
            for (coord, delta) in site.fractional.iter_mut().zip(shift) {
                *coord = (*coord + delta).rem_euclid(1.0);
            }
            site
        })
        .collect();

    let original = analyze(
        &OwnedStructure::new(nacl_lattice(), nacl_sites()),
        &AnalysisConfig::default(),
    );
    let translated = analyze(
        &OwnedStructure::new(nacl_lattice(), shifted),
        &AnalysisConfig::default(),
    );

    assert_eq!(original.overall.verdict, Verdict::StructurallyConsistent);
    assert_eq!(translated.overall.verdict, Verdict::StructurallyConsistent);
    assert_eq!(
        code_counts(&original.findings),
        code_counts(&translated.findings)
    );
}

#[test]
fn fractional_coordinate_outside_unit_range_behaves_like_its_wrapped_form() {
    let in_range = nacl_sites();
    let mut out_of_range = nacl_sites();
    out_of_range[3].fractional = [
        in_range[3].fractional[0] + 3.0,
        in_range[3].fractional[1] - 2.0,
        in_range[3].fractional[2] + 1.0,
    ];

    let a = analyze(
        &OwnedStructure::new(nacl_lattice(), in_range),
        &AnalysisConfig::default(),
    );
    let b = analyze(
        &OwnedStructure::new(nacl_lattice(), out_of_range),
        &AnalysisConfig::default(),
    );

    assert_eq!(a.overall.verdict, Verdict::StructurallyConsistent);
    assert_eq!(a.overall.verdict, b.overall.verdict);
    assert_eq!(code_counts(&a.findings), code_counts(&b.findings));
}

#[test]
fn rigid_rotation_of_the_lattice_does_not_change_verdict_or_finding_codes() {
    let sites = broken_sites();
    let original = analyze(
        &OwnedStructure::new(nacl_lattice(), sites.clone()),
        &AnalysisConfig::default(),
    );

    // 90-degree rotation about z: (x, y, z) -> (-y, x, z). Exact in floating
    // point (only negates/permutes components), so this isolates orientation
    // dependence from unrelated floating-point noise.
    let rotate = |v: [f64; 3]| [-v[1], v[0], v[2]];
    let lattice = nacl_lattice();
    let rotated_lattice = [rotate(lattice[0]), rotate(lattice[1]), rotate(lattice[2])];
    let rotated = analyze(
        &OwnedStructure::new(rotated_lattice, sites),
        &AnalysisConfig::default(),
    );

    assert_eq!(original.overall.verdict, rotated.overall.verdict);
    assert_eq!(
        code_counts(&original.findings),
        code_counts(&rotated.findings)
    );
}

#[test]
fn doubling_the_cell_along_one_axis_stays_structurally_consistent() {
    let lattice = nacl_lattice();
    let sites = nacl_sites();

    let mut supercell_lattice = lattice;
    supercell_lattice[0] = [
        lattice[0][0] * 2.0,
        lattice[0][1] * 2.0,
        lattice[0][2] * 2.0,
    ];

    let mut supercell_sites = Vec::with_capacity(sites.len() * 2);
    for site in &sites {
        for offset in [0.0, 0.5] {
            supercell_sites.push(Site {
                element: site.element.clone(),
                fractional: [
                    site.fractional[0] / 2.0 + offset,
                    site.fractional[1],
                    site.fractional[2],
                ],
                occupancy: site.occupancy,
            });
        }
    }

    let expected_site_count = sites.len() * 2;
    let supercell = analyze(
        &OwnedStructure::new(supercell_lattice, supercell_sites),
        &AnalysisConfig::default(),
    );

    assert_eq!(supercell.overall.verdict, Verdict::StructurallyConsistent);
    assert!(supercell.findings.is_empty());
    assert_eq!(supercell.input.site_count, expected_site_count);
    assert_eq!(supercell.input.distinct_element_count, 2);
}
