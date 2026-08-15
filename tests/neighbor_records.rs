//! Phase 1 (v0.4): `SiteLocalEnvironment::neighbors` / `NeighborRecord`
//! (schema v3, see `docs/v04_plan.md`). Covers the two corrections found by
//! reading `diagnostics::coordination`'s actual implementation before this
//! landed: `chematic_crystal::PeriodicNeighbor::neighbor_index` is a
//! coincidence-*group* index, not an original input site index, and a
//! resolved neighbor's occupancy must come from the original input site,
//! not the internally-constructed (artificially-occupied) geometry object.

use mikiwame::{AnalysisConfig, OwnedStructure, Site, Verdict, analyze};

fn site(element: &str, fractional: [f64; 3], occupancy: f64) -> Site {
    Site {
        element: element.to_string(),
        fractional,
        occupancy,
    }
}

fn cubic(a: f64) -> [[f64; 3]; 3] {
    [[a, 0.0, 0.0], [0.0, a, 0.0], [0.0, 0.0, a]]
}

/// A center site can be its own neighbor, reached via a non-zero periodic
/// image -- correct, not a self-reference bug (see `docs/v04_plan.md`'s
/// Phase 1 refinement). Simple cubic, BCC, and FCC one-site primitive cells
/// are the minimal fixtures that exercise this: every one of NaCl/CsCl/
/// diamond/zinc-blende/perovskite has >= 2 distinct sites, so none of them
/// would catch an implementation that (incorrectly) excludes a candidate
/// because `neighbor_site_index == center_index`.
mod self_via_periodic_image {
    use super::*;

    // Fe (Cordero radius 1.32 A) throughout: pairwise cutoff = 2*1.32 + 0.4
    // (BOND_TOLERANCE_ANGSTROM) = 3.04 A. Each fixture's lattice constant is
    // chosen so the correct nearest shell falls inside 3.04 A and the next
    // shell falls outside it, so the coordination number is unambiguous
    // without needing the gap step to do any work.

    #[test]
    fn simple_cubic_primitive_cell_has_six_self_neighbors() {
        // Nearest shell at a = 2.6 A (within 3.04 A cutoff); next shell
        // (face diagonal) at a*sqrt(2) = 3.677 A (outside it).
        let structure = OwnedStructure::new(cubic(2.6), vec![site("Fe", [0.0, 0.0, 0.0], 1.0)]);
        let report = analyze(&structure, &AnalysisConfig::default());
        assert_eq!(report.overall.verdict, Verdict::StructurallyConsistent);
        let entry = &report.local_environment[0];
        assert_eq!(entry.coordination_number, Some(6));
        assert_eq!(entry.neighbors.len(), 6);
        for n in &entry.neighbors {
            assert_eq!(
                n.neighbor_site_index, 0,
                "the only site is its own neighbor"
            );
            assert_ne!(
                n.image,
                [0, 0, 0],
                "a real neighbor is never the zero image"
            );
            assert!(n.included_in_first_shell);
        }
        // Deterministic order: all six tie on distance and element ("Fe"),
        // so the reported order is decided entirely by `image` (lexical
        // [i32; 3] order) -- hand-derived, not merely asserted-nonempty.
        let images: Vec<[i32; 3]> = entry.neighbors.iter().map(|n| n.image).collect();
        assert_eq!(
            images,
            vec![
                [-1, 0, 0],
                [0, -1, 0],
                [0, 0, -1],
                [0, 0, 1],
                [0, 1, 0],
                [1, 0, 0],
            ]
        );
    }

    #[test]
    fn bcc_primitive_cell_has_eight_self_neighbors() {
        // Rhombohedral primitive cell, one atom at the origin. Conventional
        // cubic constant a_conv = 3.2 A: nearest-neighbor distance
        // (sqrt(3)/2)*a_conv = 2.771 A (within cutoff), next shell at
        // a_conv = 3.2 A (outside it).
        let a_conv = 3.2;
        let h = a_conv / 2.0;
        let lattice = [[-h, h, h], [h, -h, h], [h, h, -h]];
        let structure = OwnedStructure::new(lattice, vec![site("Fe", [0.0, 0.0, 0.0], 1.0)]);
        let report = analyze(&structure, &AnalysisConfig::default());
        assert_eq!(report.overall.verdict, Verdict::StructurallyConsistent);
        let entry = &report.local_environment[0];
        assert_eq!(entry.coordination_number, Some(8));
        assert_eq!(entry.neighbors.len(), 8);
        for n in &entry.neighbors {
            assert_eq!(n.neighbor_site_index, 0);
            assert_ne!(n.image, [0, 0, 0]);
            assert!(n.included_in_first_shell);
        }
    }

    #[test]
    fn fcc_primitive_cell_has_twelve_self_neighbors() {
        // Rhombohedral primitive cell, one atom at the origin. Conventional
        // cubic constant a_conv = 3.5 A: nearest-neighbor distance
        // a_conv/sqrt(2) = 2.475 A (within cutoff), next shell at
        // a_conv = 3.5 A (outside it).
        let a_conv = 3.5;
        let h = a_conv / 2.0;
        let lattice = [[0.0, h, h], [h, 0.0, h], [h, h, 0.0]];
        let structure = OwnedStructure::new(lattice, vec![site("Fe", [0.0, 0.0, 0.0], 1.0)]);
        let report = analyze(&structure, &AnalysisConfig::default());
        assert_eq!(report.overall.verdict, Verdict::StructurallyConsistent);
        let entry = &report.local_environment[0];
        assert_eq!(entry.coordination_number, Some(12));
        assert_eq!(entry.neighbors.len(), 12);
        for n in &entry.neighbors {
            assert_eq!(n.neighbor_site_index, 0);
            assert_ne!(n.image, [0, 0, 0]);
            assert!(n.included_in_first_shell);
        }
    }
}

/// `NeighborRecord::occupancy` must be the *original input* occupancy, not
/// the artificial value `diagnostics::coordination::build_periodic_site`
/// assigns internally (always 1.0 for a singleton coincidence group,
/// regardless of what the real input said) to satisfy
/// `chematic_crystal::PeriodicSite`'s own occupancy-sum validation.
#[test]
fn partial_occupancy_of_a_single_species_site_is_preserved_in_the_neighbor_record() {
    let structure = OwnedStructure::new(
        cubic(10.0),
        vec![
            site("Fe", [0.0, 0.0, 0.0], 1.0),
            // Fe-Ni cutoff = 1.32 + 1.24 + 0.4 = 2.96 A; placed at 2.0 A.
            site("Ni", [0.2, 0.0, 0.0], 0.5),
        ],
    );
    let report = analyze(&structure, &AnalysisConfig::default());
    assert_eq!(report.overall.verdict, Verdict::StructurallyConsistent);
    let fe = &report.local_environment[0];
    assert_eq!(fe.coordination_number, Some(1));
    let ni_neighbor = &fe.neighbors[0];
    assert_eq!(ni_neighbor.neighbor_site_index, 1);
    assert_eq!(ni_neighbor.element, "Ni");
    assert_eq!(
        ni_neighbor.occupancy, 0.5,
        "must be the real input occupancy, not the internal geometry object's artificial 1.0"
    );
}

/// A disorder group (two species at the same position) ahead of an ordinary
/// site in the input shifts every later coincidence-group's index away from
/// its members' real input site index. `chematic_crystal::PeriodicNeighbor`
/// is built from the *group*-indexed geometry object
/// (`crystal_structure`), so `n.neighbor_index` is a group index -- using it
/// directly as `NeighborRecord::neighbor_site_index` would silently report
/// the wrong site for any structure shaped like this one.
#[test]
fn disorder_group_ahead_of_a_neighbor_does_not_shift_its_reported_site_index() {
    let structure = OwnedStructure::new(
        cubic(6.0),
        vec![
            // Group 0 (unresolved, 2 members): a disorder pair at the origin.
            site("Fe", [0.0, 0.0, 0.0], 0.5),
            site("Ni", [0.0, 0.0, 0.0], 0.5),
            // Site 2 (its own singleton group): O.
            site("O", [0.3, 0.0, 0.0], 1.0),
            // Site 3 (its own singleton group): Ti, O's neighbor.
            // O-Ti cutoff = 0.66 + 1.60 + 0.4 = 2.66 A; placed at 1.2 A.
            site("Ti", [0.5, 0.0, 0.0], 1.0),
        ],
    );
    let report = analyze(&structure, &AnalysisConfig::default());
    // Only 3 coincidence groups exist for these 4 sites (indices 0..=2), so
    // a `neighbor_site_index` of 3 can only come from mapping back through
    // `groups[n.neighbor_index].site_indices[0]` -- a bug that reported the
    // raw group index directly could never produce `3` here, regardless of
    // which arbitrary order the groups end up in internally.
    let oxygen = &report.local_environment[2];
    assert_eq!(oxygen.coordination_number, Some(1));
    assert_eq!(oxygen.neighbors.len(), 1);
    let ti_neighbor = &oxygen.neighbors[0];
    assert_eq!(
        ti_neighbor.neighbor_site_index, 3,
        "must be Ti's real input site index"
    );
    assert_eq!(ti_neighbor.element, "Ti");
    assert_eq!(ti_neighbor.occupancy, 1.0);
}

/// Schema-version-2 reports (no `neighbors` field at all) must still
/// deserialize into today's `SiteLocalEnvironment`, as an empty list rather
/// than an error -- the whole point of `#[serde(default)]` on the field.
///
/// This is the *actual* `local_environment[0]` JSON `mikiwame` v0.3.1
/// (tag `v0.3.1`, commit `7993590`) produced for the same rock-salt NaCl
/// fixture used elsewhere in this file, captured by building that tag in a
/// separate `git worktree` and running its real CLI -- not hand-typed from
/// memory of the old struct's shape, and not generated by serializing
/// today's type back down to a v2-looking subset.
#[test]
fn schema_v2_json_without_a_neighbors_field_deserializes_as_empty() {
    let v2_json = r#"{"site_index": 0, "coordination_number": 6, "neighbor_species": [{"element": "Cl", "count": 6}], "shell_gap_ratio": null, "not_computed_reason": null, "limitations": []}"#;
    let entry: mikiwame::report::SiteLocalEnvironment =
        serde_json::from_str(v2_json).expect("a schema-v2 SiteLocalEnvironment must still parse");
    assert_eq!(entry.coordination_number, Some(6));
    assert_eq!(entry.neighbors, Vec::new());
}

/// A `NeighborRecord` round-trips through JSON exactly.
#[test]
fn neighbor_record_json_round_trips() {
    let structure = OwnedStructure::new(cubic(2.6), vec![site("Fe", [0.0, 0.0, 0.0], 1.0)]);
    let report = analyze(&structure, &AnalysisConfig::default());
    let json = serde_json::to_string(&report).expect("report must serialize");
    let round_tripped: mikiwame::MaterialDiagnosticReport =
        serde_json::from_str(&json).expect("report must deserialize");
    assert_eq!(round_tripped, report);
    assert_eq!(round_tripped.local_environment[0].neighbors.len(), 6);
}
