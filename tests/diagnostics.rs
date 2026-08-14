//! End-to-end checks: a known-good fixture stays clean, and each intentionally
//! broken variant fires exactly its expected finding code and nothing else
//! (AGENTS.md §15.1/§15.2). See `fixtures/README.md` for the fixture's source.

use std::collections::HashSet;

use mikiwame::{
    AnalysisConfig, ComponentStatus, FindingCode, OwnedStructure, Site, Verdict, analyze,
};

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

fn nacl() -> OwnedStructure {
    OwnedStructure::new(nacl_lattice(), nacl_sites())
}

fn codes(findings: &[mikiwame::Finding]) -> HashSet<FindingCode> {
    findings.iter().map(|f| f.code).collect()
}

#[test]
fn clean_nacl_is_structurally_consistent() {
    let report = analyze(&nacl(), &AnalysisConfig::default());
    assert_eq!(report.overall.verdict, Verdict::StructurallyConsistent);
    assert!(report.findings.is_empty());
    assert_eq!(report.input.site_count, 8);
    assert_eq!(report.input.distinct_element_count, 2);
    assert!(
        report
            .components
            .iter()
            .all(|c| c.status == ComponentStatus::Ran)
    );
}

#[test]
fn empty_structure_is_invalid_input() {
    let structure = OwnedStructure::new(nacl_lattice(), Vec::new());
    let report = analyze(&structure, &AnalysisConfig::default());
    assert_eq!(report.overall.verdict, Verdict::InvalidInput);
    assert_eq!(
        codes(&report.findings),
        HashSet::from([FindingCode::InputEmptyStructure])
    );
}

#[test]
fn nonfinite_lattice_is_invalid_input() {
    let mut lattice = nacl_lattice();
    lattice[0][0] = f64::NAN;
    let report = analyze(
        &OwnedStructure::new(lattice, nacl_sites()),
        &AnalysisConfig::default(),
    );
    assert_eq!(report.overall.verdict, Verdict::InvalidInput);
    assert_eq!(
        codes(&report.findings),
        HashSet::from([FindingCode::InputNonfiniteLattice])
    );
}

#[test]
fn collapsed_lattice_is_singular() {
    // Second lattice vector made identical to the first: zero volume, no
    // floating-point noise involved (exact duplicate values).
    let lattice = [[A, 0.0, 0.0], [A, 0.0, 0.0], [0.0, 0.0, A]];
    let report = analyze(
        &OwnedStructure::new(lattice, nacl_sites()),
        &AnalysisConfig::default(),
    );
    assert_eq!(report.overall.verdict, Verdict::InvalidInput);
    assert_eq!(
        codes(&report.findings),
        HashSet::from([FindingCode::LatticeSingular])
    );
}

#[test]
fn nonfinite_coordinate_is_invalid_input() {
    let mut sites = nacl_sites();
    sites[0].fractional[1] = f64::INFINITY;
    let report = analyze(
        &OwnedStructure::new(nacl_lattice(), sites),
        &AnalysisConfig::default(),
    );
    assert_eq!(report.overall.verdict, Verdict::InvalidInput);
    assert_eq!(
        codes(&report.findings),
        HashSet::from([FindingCode::InputNonfiniteCoordinate])
    );
}

#[test]
fn negative_occupancy_is_review_recommended_not_fatal() {
    let mut sites = nacl_sites();
    sites[0].occupancy = -0.1;
    let report = analyze(
        &OwnedStructure::new(nacl_lattice(), sites),
        &AnalysisConfig::default(),
    );
    // Occupancy does not affect geometry, so this is not fatal: separation
    // still ran, and the structure has no duplicates, so nothing else fires.
    assert_eq!(report.overall.verdict, Verdict::ReviewRecommended);
    assert_eq!(
        codes(&report.findings),
        HashSet::from([FindingCode::InputInvalidOccupancy])
    );
    assert!(
        report
            .components
            .iter()
            .all(|c| c.status == ComponentStatus::Ran)
    );
}

#[test]
fn occupancy_above_one_is_invalid() {
    let mut sites = nacl_sites();
    sites[1].occupancy = 1.2;
    let report = analyze(
        &OwnedStructure::new(nacl_lattice(), sites),
        &AnalysisConfig::default(),
    );
    assert_eq!(
        codes(&report.findings),
        HashSet::from([FindingCode::InputInvalidOccupancy])
    );
}

#[test]
fn unknown_element_symbol_is_flagged_but_not_fatal() {
    let mut sites = nacl_sites();
    sites[0].element = "Xx".to_string();
    let report = analyze(
        &OwnedStructure::new(nacl_lattice(), sites),
        &AnalysisConfig::default(),
    );
    // Element identity doesn't affect geometry, so this is not fatal, same as
    // invalid occupancy.
    assert_eq!(report.overall.verdict, Verdict::ReviewRecommended);
    assert_eq!(
        codes(&report.findings),
        HashSet::from([FindingCode::InputUnknownElement])
    );
    assert!(
        report
            .components
            .iter()
            .all(|c| c.status == ComponentStatus::Ran)
    );
}

#[test]
fn coincident_same_element_sites_are_duplicates() {
    let mut sites = nacl_sites();
    sites[1].fractional = sites[0].fractional; // Na onto Na
    let report = analyze(
        &OwnedStructure::new(nacl_lattice(), sites),
        &AnalysisConfig::default(),
    );
    assert_eq!(report.overall.verdict, Verdict::StrongAnomalyDetected);
    assert_eq!(
        codes(&report.findings),
        HashSet::from([FindingCode::SiteDuplicate])
    );
    // NaCl's lattice is cubic -- well inside chematic_crystal::Lattice's
    // acceptance range -- so this took the exact minimum-image path, which
    // has nothing to caveat.
    assert!(
        report.findings[0].limitations.is_empty(),
        "exact-path SITE_DUPLICATE should have no limitations, got {:?}",
        report.findings[0].limitations
    );
}

#[test]
fn coincident_same_element_sites_on_a_near_singular_lattice_note_the_fallback() {
    // Same "b" vector chematic_crystal::Lattice::from_matrix rejects as
    // near-singular in structure_view::tests -- positive volume, so
    // input_quality's LATTICE_SINGULAR (fatal only for volume <= 0) lets it
    // through to separation, which must fall back rather than panic.
    let lattice = [[1.0, 0.0, 0.0], [0.5, 1e-4, 0.0], [0.0, 0.0, 1.0]];
    let sites = vec![
        Site {
            element: "Na".to_string(),
            fractional: [0.2, 0.2, 0.2],
            occupancy: 1.0,
        },
        Site {
            element: "Na".to_string(),
            fractional: [0.2, 0.2, 0.2],
            occupancy: 1.0,
        },
    ];
    let report = analyze(
        &OwnedStructure::new(lattice, sites),
        &AnalysisConfig::default(),
    );
    assert_eq!(
        codes(&report.findings),
        HashSet::from([FindingCode::SiteDuplicate])
    );
    let limitations = &report.findings[0].limitations;
    assert_eq!(limitations.len(), 1, "got {limitations:?}");
    assert!(
        limitations[0].contains("approximated"),
        "expected the fallback caveat, got {limitations:?}"
    );
}

#[test]
fn coincident_different_element_sites_with_valid_occupancy_sum_are_disorder_not_duplicates() {
    // A Cl placed exactly where a Na sits, each at half occupancy, is a
    // textbook-valid disordered site (occupancies sum to 1.0) — must not
    // fire SITE_DUPLICATE, and the occupancy sum being valid must not fire
    // DISORDER_OCCUPANCY_SUM_EXCEEDS_ONE either. DISORDER_PRESENT is
    // informational and must not move the verdict off StructurallyConsistent
    // (AGENTS.md §7.7: disorder is not itself an anomaly).
    let mut sites = nacl_sites();
    sites[4].fractional = sites[0].fractional; // Cl onto Na
    sites[0].occupancy = 0.5;
    sites[4].occupancy = 0.5;
    let report = analyze(
        &OwnedStructure::new(nacl_lattice(), sites),
        &AnalysisConfig::default(),
    );
    assert_eq!(report.overall.verdict, Verdict::StructurallyConsistent);
    assert_eq!(
        codes(&report.findings),
        HashSet::from([FindingCode::DisorderPresent])
    );
}

#[test]
fn coincident_different_element_sites_with_full_occupancy_each_exceed_the_site() {
    // Same coincident pair as above, but both left at full occupancy (as in
    // the unmodified NaCl fixture): physically over-full, since occupancy
    // sum needs no external threshold to know it can't exceed 1.0.
    let mut sites = nacl_sites();
    sites[4].fractional = sites[0].fractional; // Cl onto Na, both occupancy 1.0
    let report = analyze(
        &OwnedStructure::new(nacl_lattice(), sites),
        &AnalysisConfig::default(),
    );
    assert_eq!(report.overall.verdict, Verdict::ReviewRecommended);
    assert_eq!(
        codes(&report.findings),
        HashSet::from([
            FindingCode::DisorderPresent,
            FindingCode::DisorderOccupancySumExceedsOne,
        ])
    );
}

#[test]
fn report_with_evidence_round_trips_through_json() {
    // Exercises the custom Score01/ClosedRange (de)serializers and the
    // NumericEvidence path, not just the empty-findings case (AGENTS.md §15.5).
    let mut sites = nacl_sites();
    sites[0].occupancy = -0.1;
    let report = analyze(
        &OwnedStructure::new(nacl_lattice(), sites),
        &AnalysisConfig::default(),
    );
    assert!(!report.findings.is_empty());

    let json = serde_json::to_string(&report).expect("report serializes");
    let restored: mikiwame::MaterialDiagnosticReport =
        serde_json::from_str(&json).expect("report deserializes");
    assert_eq!(report, restored);
}
