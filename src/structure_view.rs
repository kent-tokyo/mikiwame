//! The read-only structure boundary mikiwame's diagnostics program against.
//!
//! chematic's default branch has no periodic/occupancy-aware structure type to
//! depend on (see `docs/chematic-prerequisites.md`), so mikiwame defines its
//! own minimal trait instead of a concrete owned type. Callers with their own
//! structure representation (including a future chematic type) implement
//! [`PeriodicStructureView`] directly; [`OwnedStructure`] exists only for
//! tests, fixtures, and direct programmatic construction.

/// One atomic site: an element at a fractional position, with an occupancy.
///
/// Disorder (multiple species sharing a position) is represented as multiple
/// `Site`s with the same `fractional` coordinates and different `element`s —
/// there is no dedicated multi-species site type in v0.1.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Site {
    /// Element symbol, e.g. `"Na"`. Not validated at construction — an
    /// unrecognized symbol is reported as `INPUT_UNKNOWN_ELEMENT` by
    /// `analyze`, not rejected here, so the diagnosis can explain what's
    /// wrong.
    pub element: String,
    /// Fractional coordinates within the unit cell.
    pub fractional: [f64; 3],
    /// Site occupancy. Expected to be finite and within `[0.0, 1.0]`;
    /// violations are reported as `INPUT_INVALID_OCCUPANCY`, not rejected at
    /// construction, so that `analyze` can explain what is wrong.
    pub occupancy: f64,
}

/// Read-only access to a periodic structure's lattice and sites.
///
/// This is the entire surface mikiwame's diagnostics consume. See the module
/// doc comment for why this is a trait rather than a concrete type.
pub trait PeriodicStructureView {
    /// The lattice matrix as row vectors `[a, b, c]`, in Angstrom.
    fn lattice(&self) -> &[[f64; 3]; 3];
    /// All sites in the structure.
    fn sites(&self) -> &[Site];
}

/// An owned, directly-constructed structure implementing
/// [`PeriodicStructureView`]. Used by tests, fixtures, and any caller without
/// its own structure type to hand to.
#[derive(Debug, Clone, PartialEq)]
pub struct OwnedStructure {
    lattice: [[f64; 3]; 3],
    sites: Vec<Site>,
}

impl OwnedStructure {
    /// Builds an `OwnedStructure` from a lattice matrix and a site list.
    pub fn new(lattice: [[f64; 3]; 3], sites: Vec<Site>) -> Self {
        Self { lattice, sites }
    }
}

impl PeriodicStructureView for OwnedStructure {
    fn lattice(&self) -> &[[f64; 3]; 3] {
        &self.lattice
    }

    fn sites(&self) -> &[Site] {
        &self.sites
    }
}

/// Cell volume as the scalar triple product of the lattice row vectors.
///
/// Can be negative (left-handed axes) or (numerically) zero; callers decide
/// what that means. Does not itself guard against non-finite input — callers
/// must check finiteness first (see `diagnostics::input_quality`).
pub(crate) fn cell_volume(lattice: &[[f64; 3]; 3]) -> f64 {
    let [a, b, c] = *lattice;
    let cross = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    cross[0] * c[0] + cross[1] * c[1] + cross[2] * c[2]
}

/// Converts fractional coordinates to Cartesian, given lattice row vectors.
pub(crate) fn frac_to_cart(lattice: &[[f64; 3]; 3], frac: [f64; 3]) -> [f64; 3] {
    let [a, b, c] = *lattice;
    [
        frac[0] * a[0] + frac[1] * b[0] + frac[2] * c[0],
        frac[0] * a[1] + frac[1] * b[1] + frac[2] * c[1],
        frac[0] * a[2] + frac[1] * b[2] + frac[2] * c[2],
    ]
}

/// Minimum-image distance between two fractional positions under PBC.
///
/// ponytail: wraps each fractional axis independently to its nearest image
/// (`d -= round(d)`) rather than searching the full 3×3×3 neighboring-cell
/// shell. Exact for reasonably orthogonal cells; can miss the true minimum
/// image for highly skewed/acute lattices. Upgrade to a 3×3×3 image search if
/// mikiwame is used on strongly non-orthogonal cells.
pub(crate) fn minimum_image_distance(lattice: &[[f64; 3]; 3], a: [f64; 3], b: [f64; 3]) -> f64 {
    let mut delta = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    for x in delta.iter_mut() {
        *x -= x.round();
    }
    let cart = frac_to_cart(lattice, delta);
    (cart[0] * cart[0] + cart[1] * cart[1] + cart[2] * cart[2]).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-9;

    fn approx(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < TOL,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn cell_volume_of_cubic_cell_is_a_cubed() {
        let a = 5.6402;
        let lattice = [[a, 0.0, 0.0], [0.0, a, 0.0], [0.0, 0.0, a]];
        approx(cell_volume(&lattice), a * a * a);
    }

    #[test]
    fn cell_volume_of_hexagonal_cell_matches_closed_form() {
        // a = b = 3, c = 5, gamma = 120 degrees, alpha = beta = 90 degrees.
        // Non-orthogonal: every other fixture in this crate is cubic, where a
        // row/column transposition bug in the volume or frac_to_cart formula
        // would be invisible. This one catches it.
        let a = 3.0_f64;
        let c = 5.0_f64;
        let gamma = 120f64.to_radians();
        let lattice = [
            [a, 0.0, 0.0],
            [a * gamma.cos(), a * gamma.sin(), 0.0],
            [0.0, 0.0, c],
        ];
        approx(cell_volume(&lattice), a * a * c * gamma.sin());
    }

    #[test]
    fn frac_to_cart_uses_row_vector_convention() {
        let lattice = [
            [3.0, 0.0, 0.0],
            [
                3.0 * 120f64.to_radians().cos(),
                3.0 * 120f64.to_radians().sin(),
                0.0,
            ],
            [0.0, 0.0, 5.0],
        ];
        // A unit step along one fractional axis must land exactly on that
        // lattice row: pins the row-vector convention so a future refactor to
        // column-vectors fails loudly instead of silently (both conventions
        // agree on a diagonal/cubic lattice, which is why no cubic fixture
        // elsewhere in this crate would catch a transposition).
        assert_eq!(frac_to_cart(&lattice, [1.0, 0.0, 0.0]), lattice[0]);
        assert_eq!(frac_to_cart(&lattice, [0.0, 1.0, 0.0]), lattice[1]);
    }

    #[test]
    fn minimum_image_distance_wraps_across_the_cell_boundary() {
        let a = 5.6402;
        let lattice = [[a, 0.0, 0.0], [0.0, a, 0.0], [0.0, 0.0, a]];
        let d = minimum_image_distance(&lattice, [0.05, 0.0, 0.0], [0.95, 0.0, 0.0]);
        // True separation is 0.1 cell widths through the periodic boundary,
        // not 0.9 straight across the cell.
        approx(d, 0.1 * a);
    }

    #[test]
    fn naive_minimum_image_can_miss_the_true_minimum_on_a_skewed_cell() {
        // Documents the ceiling named in minimum_image_distance's `ponytail`
        // comment. With nearly-parallel lattice vectors a and b, a legitimate
        // periodic image (p vs. q shifted by one whole b vector) is
        // dramatically shorter than what independent per-axis rounding finds
        // — because that image requires a *different* integer shift on the a
        // axis than on the b axis, which per-axis rounding of a single delta
        // cannot produce when both components start out equal.
        let lattice = [[1.0, 0.0, 0.0], [0.9, 0.1, 0.0], [0.0, 0.0, 10.0]];
        let p = [0.75, 0.75, 0.0];
        let q = [0.25, 0.25, 0.0];

        let naive = minimum_image_distance(&lattice, p, q);

        let q_shifted_by_b = [q[0], q[1] + 1.0, q[2]];
        let true_delta = [
            p[0] - q_shifted_by_b[0],
            p[1] - q_shifted_by_b[1],
            p[2] - q_shifted_by_b[2],
        ];
        let cart = frac_to_cart(&lattice, true_delta);
        let true_min = (cart[0] * cart[0] + cart[1] * cart[1] + cart[2] * cart[2]).sqrt();

        assert!(
            true_min < naive * 0.5,
            "expected a much shorter periodic image ({true_min}) than the naive result ({naive})"
        );
    }
}
