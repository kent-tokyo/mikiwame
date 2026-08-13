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
#[derive(Debug, Clone, PartialEq)]
pub struct Site {
    /// Element symbol, e.g. `"Na"`. Not validated against a periodic table in
    /// v0.1 (see `tasks/todo.md`: `INPUT_UNKNOWN_ELEMENT` is deferred).
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
