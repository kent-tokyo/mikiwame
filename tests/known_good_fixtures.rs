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

use mikiwame::{AnalysisConfig, MaterialDiagnosticReport, OwnedStructure, Site, Verdict, analyze};

fn assert_clean(
    structure: &OwnedStructure,
    expected_sites: usize,
    expected_elements: usize,
) -> MaterialDiagnosticReport {
    let report = analyze(structure, &AnalysisConfig::default());
    assert_eq!(report.overall.verdict, Verdict::StructurallyConsistent);
    assert!(report.findings.is_empty());
    assert_eq!(report.input.site_count, expected_sites);
    assert_eq!(report.input.distinct_element_count, expected_elements);
    report
}

/// The resolved coordination number for `site_index`, panicking if
/// `diagnostics::coordination` produced no entry at all for it (as opposed
/// to an entry with `coordination_number: None`, which callers check for
/// directly — a missing entry means the component didn't run).
fn coordination_number(report: &MaterialDiagnosticReport, site_index: usize) -> Option<usize> {
    report
        .local_environment
        .iter()
        .find(|e| e.site_index == site_index)
        .unwrap_or_else(|| panic!("no local_environment entry for site {site_index}"))
        .coordination_number
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
    let report = assert_clean(&OwnedStructure::new(lattice, sites), 2, 2);
    // The textbook 8-fold cubic coordination — the case this project's own
    // design work found a naive radius-sum+epsilon cutoff gets wrong (14:
    // 8 Cl at 3.571 A *and* 6 Cs at 4.123 A, since Cs+Cs+epsilon = 5.28 A).
    // See diagnostics::coordination's module doc comment and
    // docs/scientific_scope.md.
    assert_eq!(coordination_number(&report, 0), Some(8), "Cs");
    assert_eq!(coordination_number(&report, 1), Some(8), "Cl");
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
    let report = assert_clean(&OwnedStructure::new(lattice, sites), 8, 1);
    // Tetrahedral, 4-fold: nearest-neighbor C-C at a*sqrt(3)/4 = 1.544 A,
    // well inside C+C+epsilon = 1.92 A; the next (same-sublattice) shell at
    // a/sqrt(2) = 2.522 A is correctly excluded by the pairwise cutoff
    // itself here (no ambiguity to resolve via the gap step).
    for site_index in 0..8 {
        assert_eq!(coordination_number(&report, site_index), Some(4));
    }
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
    let report = assert_clean(&OwnedStructure::new(lattice, sites), 8, 2);
    // Tetrahedral, 4-fold, both sublattices: Zn-S at a*sqrt(3)/4 = 2.343 A
    // inside Zn+S+epsilon = 2.67 A; same-element next shell at a/sqrt(2) =
    // 3.825 A excluded by the pairwise cutoff (Zn+Zn+epsilon = 2.84 A,
    // S+S+epsilon = 2.50 A) before the gap step is even needed.
    for site_index in 0..8 {
        assert_eq!(coordination_number(&report, site_index), Some(4));
    }
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
    let report = assert_clean(&OwnedStructure::new(lattice, sites), 5, 3);
    // The sharpest case this project's design work checked by hand: three
    // candidate shells survive the pairwise radius-sum+epsilon filter
    // around Sr (12 O at a/sqrt(2)=2.761 A, 8 Ti at a*sqrt(3)/2=3.382 A, 6
    // Sr at a=3.905 A -- Sr+Ti+epsilon=3.95 A and Sr+Sr+epsilon=4.3 A both
    // exceed the O shell's distance too). The largest-relative-gap step
    // must pick the *first* of the two gaps (O->Ti ratio ~=1.225 vs.
    // Ti->Sr ratio ~=1.155), not just any gap, to land on the textbook
    // 12-fold A-site coordination.
    assert_eq!(coordination_number(&report, 0), Some(12), "Sr");
    // Textbook 6-fold octahedral B-site coordination -- the other case this
    // project's design work found a naive radius-sum+epsilon cutoff gets
    // wrong (14: 6 O at 1.9525 A *and* 8 Sr at 3.382 A, since
    // Ti+Sr+epsilon = 3.95 A).
    assert_eq!(coordination_number(&report, 1), Some(6), "Ti");
    // O's tightest, most clearly separated shell is its 2 collinear Ti
    // neighbors at a/2=1.9525 A (Ti+O+epsilon=2.66 A); the next shell (4 Sr
    // + up to 8 O, all at a/sqrt(2)=2.761 A) is materially farther
    // (gap ratio sqrt(2)~=1.414) and is correctly excluded. This differs
    // from some textbook conventions that describe O as "6-coordinate"
    // (2 Ti + 4 Sr combined) -- this method reports the geometrically
    // tightest shell, not a convention-specific combined count; see
    // diagnostics::coordination's module doc comment.
    for site_index in 2..5 {
        assert_eq!(coordination_number(&report, site_index), Some(2), "O");
    }
}
