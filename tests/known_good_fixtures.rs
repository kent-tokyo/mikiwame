//! Known-good structure fixtures beyond rock salt (AGENTS.md §15.1), scoped
//! to the ones buildable from memorized textbook Wyckoff positions with high
//! confidence — no free internal structural parameter to get subtly wrong.
//! Wurtzite, rutile, and spinel (also on AGENTS.md's candidate list) all have
//! at least one free positional parameter (e.g. wurtzite's `u`, rutile's `x`)
//! that would need a specific cited source to reproduce accurately rather
//! than transcribe from memory; graphite is deferred with them for the same
//! reason applied consistently rather than cherry-picked. See
//! `fixtures/README.md` and `docs/validation.md`.
//!
//! Each fixture only asserts `StructurallyConsistent` with zero findings and
//! the expected site/element counts: no currently-shipped diagnostic
//! inspects real bond distances or coordination, so that's the full extent
//! of what these fixtures can currently exercise (see `docs/validation.md`).

use mikiwame::{AnalysisConfig, OwnedStructure, Site, Verdict, analyze};

fn assert_clean(structure: &OwnedStructure, expected_sites: usize, expected_elements: usize) {
    let report = analyze(structure, &AnalysisConfig::default());
    assert_eq!(report.overall.verdict, Verdict::StructurallyConsistent);
    assert!(report.findings.is_empty());
    assert_eq!(report.input.site_count, expected_sites);
    assert_eq!(report.input.distinct_element_count, expected_elements);
}

fn site(element: &str, fractional: [f64; 3]) -> Site {
    Site {
        element: element.to_string(),
        fractional,
        occupancy: 1.0,
    }
}

/// CsCl (space group Pm-3m, No. 221): two interpenetrating simple-cubic
/// sublattices, one Cs and one Cl — not body-centered cubic, since the two
/// sites are different elements. a = 4.123 Å.
#[test]
fn cscl_is_structurally_consistent() {
    let a = 4.123;
    let lattice = [[a, 0.0, 0.0], [0.0, a, 0.0], [0.0, 0.0, a]];
    let sites = vec![site("Cs", [0.0, 0.0, 0.0]), site("Cl", [0.5, 0.5, 0.5])];
    assert_clean(&OwnedStructure::new(lattice, sites), 2, 2);
}

/// Diamond cubic (space group Fd-3m, No. 227): two face-centered-cubic
/// sublattices offset by (1/4, 1/4, 1/4). a = 3.567 Å.
#[test]
fn diamond_is_structurally_consistent() {
    let a = 3.567;
    let lattice = [[a, 0.0, 0.0], [0.0, a, 0.0], [0.0, 0.0, a]];
    let sites = ["C"]
        .iter()
        .flat_map(|element| {
            [
                [0.0, 0.0, 0.0],
                [0.5, 0.5, 0.0],
                [0.5, 0.0, 0.5],
                [0.0, 0.5, 0.5],
                [0.25, 0.25, 0.25],
                [0.75, 0.75, 0.25],
                [0.75, 0.25, 0.75],
                [0.25, 0.75, 0.75],
            ]
            .into_iter()
            .map(move |f| site(element, f))
        })
        .collect::<Vec<_>>();
    assert_clean(&OwnedStructure::new(lattice, sites), 8, 1);
}

/// Zinc blende (sphalerite ZnS, space group F-43m, No. 216): the diamond
/// topology with Zn and S on the two FCC sublattices instead of one element
/// on both. a = 5.41 Å.
#[test]
fn zinc_blende_is_structurally_consistent() {
    let a = 5.41;
    let lattice = [[a, 0.0, 0.0], [0.0, a, 0.0], [0.0, 0.0, a]];
    let zn_positions = [
        [0.0, 0.0, 0.0],
        [0.5, 0.5, 0.0],
        [0.5, 0.0, 0.5],
        [0.0, 0.5, 0.5],
    ];
    let s_positions = [
        [0.25, 0.25, 0.25],
        [0.75, 0.75, 0.25],
        [0.75, 0.25, 0.75],
        [0.25, 0.75, 0.75],
    ];
    let sites = zn_positions
        .into_iter()
        .map(|f| site("Zn", f))
        .chain(s_positions.into_iter().map(|f| site("S", f)))
        .collect::<Vec<_>>();
    assert_clean(&OwnedStructure::new(lattice, sites), 8, 2);
}

/// Ideal cubic perovskite (SrTiO3, space group Pm-3m, No. 221): A-site at the
/// corner, B-site at the body center, O at the three face centers. a = 3.905 Å.
#[test]
fn perovskite_is_structurally_consistent() {
    let a = 3.905;
    let lattice = [[a, 0.0, 0.0], [0.0, a, 0.0], [0.0, 0.0, a]];
    let sites = vec![
        site("Sr", [0.0, 0.0, 0.0]),
        site("Ti", [0.5, 0.5, 0.5]),
        site("O", [0.5, 0.5, 0.0]),
        site("O", [0.5, 0.0, 0.5]),
        site("O", [0.0, 0.5, 0.5]),
    ];
    assert_clean(&OwnedStructure::new(lattice, sites), 5, 3);
}
