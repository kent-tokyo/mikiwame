//! CIF input integration tests — only compiled with `--features cif` (see
//! `[[test]]` in `Cargo.toml`).

use mikiwame::PeriodicStructureView;
use mikiwame::cif::{CifPeriodicError, CifSymmetryStatus, read_cif};

const TOL: f64 = 1e-9;

fn approx(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() < TOL,
        "{label}: expected {expected}, got {actual}"
    );
}

/// A cubic NaCl-like CIF with literal fractional coordinates. Fractional
/// coordinates come straight from CIF tokens (no lattice math), so those
/// compare exactly; the lattice matrix goes through
/// `chematic_crystal::Lattice::from_parameters`, which produces ~1e-16 trig
/// noise on off-diagonal entries (`cos(90°)` rather than a literal `0.0`) —
/// compared elementwise with tolerance, not `assert_eq!`.
const NACL_CIF: &str = "data_NaCl\n\
    _cell_length_a 5.6402\n_cell_length_b 5.6402\n_cell_length_c 5.6402\n\
    _cell_angle_alpha 90\n_cell_angle_beta 90\n_cell_angle_gamma 90\n\
    loop_\n\
    _atom_site_label\n_atom_site_type_symbol\n\
    _atom_site_fract_x\n_atom_site_fract_y\n_atom_site_fract_z\n\
    Na1 Na 0.0 0.0 0.0\n\
    Cl1 Cl 0.5 0.5 0.5\n";

#[test]
fn cubic_round_trip_matches_expected_sites_and_lattice() {
    let cif = read_cif(NACL_CIF).expect("valid CIF must parse");
    assert_eq!(cif.symmetry, CifSymmetryStatus::P1);

    let sites = cif.structure.sites();
    assert_eq!(sites.len(), 2);
    assert_eq!(sites[0].element, "Na");
    assert_eq!(sites[0].fractional, [0.0, 0.0, 0.0]);
    approx(sites[0].occupancy, 1.0, "Na occupancy");
    assert_eq!(sites[1].element, "Cl");
    assert_eq!(sites[1].fractional, [0.5, 0.5, 0.5]);
    approx(sites[1].occupancy, 1.0, "Cl occupancy");

    let lattice = cif.structure.lattice();
    let a = 5.6402;
    let expected = [[a, 0.0, 0.0], [0.0, a, 0.0], [0.0, 0.0, a]];
    for row in 0..3 {
        for col in 0..3 {
            approx(
                lattice[row][col],
                expected[row][col],
                &format!("lattice[{row}][{col}]"),
            );
        }
    }
}

/// Hexagonal-parameter cell (a=b=3, c=5, gamma=120°) — deliberately
/// non-cubic, since row vs. column lattice convention is indistinguishable
/// on a cubic cell (mikiwame's other fixtures are all cubic). Checks the
/// raw returned lattice matrix directly against the IUCr placement
/// (`Lattice::from_parameters`'s own documented convention: a along x, b in
/// the xy-plane) — if `PeriodicStructure::lattice().matrix()` were
/// transposed relative to mikiwame's row-vector convention
/// (`structure_view::frac_to_cart`), row 1 here would come out as
/// `[3*cos(120°), 3, 0]` instead of `[3*cos(120°), 3*sin(120°), 0]`, so this
/// test fails where a cubic-only fixture could not.
const HEX_CIF: &str = "data_hex_test\n\
    _cell_length_a 3.0\n_cell_length_b 3.0\n_cell_length_c 5.0\n\
    _cell_angle_alpha 90\n_cell_angle_beta 90\n_cell_angle_gamma 120\n\
    loop_\n\
    _atom_site_label\n_atom_site_type_symbol\n\
    _atom_site_fract_x\n_atom_site_fract_y\n_atom_site_fract_z\n\
    C1 C 0.0 0.0 0.0\n\
    C2 C 0.333333 0.666667 0.5\n";

#[test]
fn non_cubic_lattice_uses_row_vector_convention() {
    let cif = read_cif(HEX_CIF).expect("valid CIF must parse");
    let lattice = cif.structure.lattice();

    let gamma = 120f64.to_radians();
    approx(lattice[0][0], 3.0, "a.x");
    approx(lattice[0][1], 0.0, "a.y");
    approx(lattice[0][2], 0.0, "a.z");
    approx(lattice[1][0], 3.0 * gamma.cos(), "b.x");
    approx(lattice[1][1], 3.0 * gamma.sin(), "b.y");
    approx(lattice[1][2], 0.0, "b.z");
    approx(lattice[2][0], 0.0, "c.x");
    approx(lattice[2][1], 0.0, "c.y");
    approx(lattice[2][2], 5.0, "c.z");
}

/// Two rows sharing one fractional position (CIF's disorder convention)
/// must flatten into two mikiwame `Site`s at that position, matching
/// mikiwame's own disorder representation (multiple same-position `Site`s,
/// not a dedicated multi-species type).
const DISORDER_CIF: &str = "data_disorder_test\n\
    _cell_length_a 4.0\n_cell_length_b 4.0\n_cell_length_c 4.0\n\
    _cell_angle_alpha 90\n_cell_angle_beta 90\n_cell_angle_gamma 90\n\
    loop_\n\
    _atom_site_label\n_atom_site_type_symbol\n\
    _atom_site_fract_x\n_atom_site_fract_y\n_atom_site_fract_z\n\
    _atom_site_occupancy\n\
    Fe1 Fe 0.0 0.0 0.0 0.6\n\
    Ni1 Ni 0.0 0.0 0.0 0.4\n";

#[test]
fn disorder_rows_become_two_sites_at_the_same_position() {
    let cif = read_cif(DISORDER_CIF).expect("valid CIF must parse");
    let sites = cif.structure.sites();
    assert_eq!(sites.len(), 2);
    assert_eq!(sites[0].fractional, [0.0, 0.0, 0.0]);
    assert_eq!(sites[1].fractional, [0.0, 0.0, 0.0]);
    assert_eq!(sites[0].element, "Fe");
    approx(sites[0].occupancy, 0.6, "Fe occupancy");
    assert_eq!(sites[1].element, "Ni");
    approx(sites[1].occupancy, 0.4, "Ni occupancy");
}

/// An occupancy sum over `1.0` (+ tolerance) is rejected by
/// `chematic_crystal::PeriodicSite`'s own construction-time validation —
/// this is the CLI-error-not-diagnosed-finding tradeoff this module's doc
/// comment documents, confirmed at the library boundary.
const OVER_OCCUPANCY_CIF: &str = "data_bad_occupancy\n\
    _cell_length_a 4.0\n_cell_length_b 4.0\n_cell_length_c 4.0\n\
    _cell_angle_alpha 90\n_cell_angle_beta 90\n_cell_angle_gamma 90\n\
    loop_\n\
    _atom_site_label\n_atom_site_type_symbol\n\
    _atom_site_fract_x\n_atom_site_fract_y\n_atom_site_fract_z\n\
    _atom_site_occupancy\n\
    Fe1 Fe 0.0 0.0 0.0 0.7\n\
    Ni1 Ni 0.0 0.0 0.0 0.5\n";

#[test]
fn occupancy_sum_exceeded_is_rejected_not_diagnosed() {
    let err = read_cif(OVER_OCCUPANCY_CIF).expect_err("occupancy sum of 1.2 must be rejected");
    match err {
        CifPeriodicError::Crystal(chematic_crystal::CrystalError::OccupancySumExceeded {
            ..
        }) => {}
        other => panic!("expected OccupancySumExceeded, got {other:?}"),
    }
}

/// A CIF declaring symmetry beyond P1 must surface
/// `CifSymmetryStatus::UnexpandedSymmetry`, not be silently treated as a
/// complete cell — the returned sites are only the asymmetric unit.
const SYMMETRY_CIF: &str = "data_symmetry_test\n\
    _cell_length_a 4.0\n_cell_length_b 4.0\n_cell_length_c 4.0\n\
    _cell_angle_alpha 90\n_cell_angle_beta 90\n_cell_angle_gamma 90\n\
    _symmetry_Int_Tables_number 15\n\
    loop_\n\
    _atom_site_label\n_atom_site_type_symbol\n\
    _atom_site_fract_x\n_atom_site_fract_y\n_atom_site_fract_z\n\
    Ti1 Ti 0.0 0.25 0.25\n";

#[test]
fn declared_non_p1_symmetry_is_surfaced_not_expanded() {
    let cif = read_cif(SYMMETRY_CIF).expect("valid CIF must parse");
    match cif.symmetry {
        CifSymmetryStatus::UnexpandedSymmetry {
            space_group_name,
            operation_count,
        } => {
            assert_eq!(space_group_name, None);
            assert_eq!(operation_count, 0);
        }
        CifSymmetryStatus::P1 => panic!("expected UnexpandedSymmetry"),
    }
    assert_eq!(cif.structure.sites().len(), 1);
}
