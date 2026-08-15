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

/// How a [`PeriodicDistance`] was computed.
///
/// Exposed to callers (rather than folded silently into the distance number)
/// so a finding built from an approximate distance can say so — mikiwame's
/// evidence-first stance (AGENTS.md §6) applies to *how* a number was
/// computed, not just what it is.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PeriodicDistanceMethod {
    /// `chematic_crystal::minimum_image`'s reciprocal-lattice-bounded
    /// search: exact for any lattice `Lattice::from_matrix` accepts.
    Exact,
    /// The old per-axis-rounding approximation, used only because
    /// `chematic_crystal::Lattice::from_matrix` rejected the lattice; can
    /// miss the true minimum image on skewed cells (see
    /// `docs/validation.md`).
    ApproximateFallback {
        /// Why `Lattice::from_matrix` rejected the lattice, in
        /// `chematic_crystal`'s own words (its `CrystalError` messages are
        /// written to be self-contained — see that crate's error module).
        reason: String,
    },
}

impl PeriodicDistanceMethod {
    /// A `Finding::limitations` entry for this method, or `None` for the
    /// exact path (nothing to caveat).
    pub(crate) fn limitation(&self) -> Option<String> {
        match self {
            PeriodicDistanceMethod::Exact => None,
            PeriodicDistanceMethod::ApproximateFallback { reason } => Some(format!(
                "periodic distance approximated by independent per-axis rounding, not the \
                 exact minimum-image search, because the lattice was rejected by \
                 chematic_crystal::Lattice::from_matrix ({reason}); may not be the true \
                 minimum image on a sufficiently skewed cell"
            )),
        }
    }
}

/// A minimum-image distance together with how it was computed.
pub(crate) struct PeriodicDistance {
    pub(crate) distance_angstrom: f64,
    pub(crate) method: PeriodicDistanceMethod,
}

/// A lattice matrix resolved once into either an exact
/// `chematic_crystal::Lattice` or the fallback's raw-matrix-plus-reason, so
/// repeated pairwise [`ResolvedLattice::minimum_image`] calls against the
/// *same* lattice (as [`coincidence_groups`] and `separation::check` each
/// do, inside an O(n^2) scan) don't redundantly reconstruct and re-validate
/// `chematic_crystal::Lattice` — matrix inversion and all — on every single
/// pair.
pub(crate) enum ResolvedLattice {
    Exact(chematic_crystal::Lattice),
    ApproximateFallback { raw: [[f64; 3]; 3], reason: String },
}

impl ResolvedLattice {
    /// Resolves a raw lattice matrix once. See
    /// [`ResolvedLattice::minimum_image`]'s doc comment for why this can
    /// fall back rather than always succeeding.
    pub(crate) fn resolve(lattice: &[[f64; 3]; 3]) -> Self {
        match chematic_crystal::Lattice::from_matrix(*lattice) {
            Ok(crystal_lattice) => Self::Exact(crystal_lattice),
            Err(err) => Self::ApproximateFallback {
                raw: *lattice,
                reason: err.to_string(),
            },
        }
    }

    /// Minimum-image distance between two fractional positions under PBC.
    ///
    /// Delegates to `chematic_crystal::minimum_image`, which is exact for
    /// any lattice it accepts (a reciprocal-lattice-derived search box,
    /// brute-force checked inside it — see that crate's `periodic` module
    /// doc for the derivation and its own oracle tests). This fixed a real
    /// gap: the previous per-axis-rounding approximation could miss the
    /// true minimum image on skewed cells (see `docs/validation.md`).
    ///
    /// `input_quality`'s `LATTICE_SINGULAR` check is fatal only for
    /// non-positive cell volume, not for near-singularity or a very short
    /// axis — both of which `chematic_crystal::Lattice::from_matrix`
    /// rejects. So a lattice that passed `input_quality` and reached
    /// [`ResolvedLattice::resolve`] can still fail construction; in that
    /// case this falls back to the old per-axis approximation rather than
    /// panicking, so `separation`/`disorder` keep running on such lattices
    /// exactly as they did before `chematic_crystal` was adopted — but
    /// callers get told which method ran (see [`PeriodicDistanceMethod`])
    /// so a finding built from a fallback-computed distance can say so,
    /// rather than reporting the same unqualified confidence either way.
    /// Adopting `chematic_crystal::Lattice`'s stricter condition-number
    /// criterion as `LATTICE_SINGULAR`'s own threshold (it would also
    /// resolve the `LATTICE_EXTREME_ASPECT_RATIO` backlog item — see
    /// `tasks/todo.md`) is a separate, future decision, not made here.
    pub(crate) fn minimum_image(&self, a: [f64; 3], b: [f64; 3]) -> PeriodicDistance {
        match self {
            Self::Exact(crystal_lattice) => {
                let distance = chematic_crystal::minimum_image(
                    crystal_lattice,
                    chematic_crystal::FractionalCoord::new(a),
                    chematic_crystal::FractionalCoord::new(b),
                )
                .distance;
                PeriodicDistance {
                    distance_angstrom: distance,
                    method: PeriodicDistanceMethod::Exact,
                }
            }
            Self::ApproximateFallback { raw, reason } => PeriodicDistance {
                distance_angstrom: naive_minimum_image_distance(raw, a, b),
                method: PeriodicDistanceMethod::ApproximateFallback {
                    reason: reason.clone(),
                },
            },
        }
    }
}

/// The per-axis-rounding approximation [`ResolvedLattice::minimum_image`]
/// used exclusively before this module started delegating to
/// `chematic_crystal::minimum_image`. Kept only as a fallback for lattices
/// `chematic_crystal::Lattice::from_matrix` rejects as near-singular — see
/// that method's doc comment.
fn naive_minimum_image_distance(lattice: &[[f64; 3]; 3], a: [f64; 3], b: [f64; 3]) -> f64 {
    let mut delta = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    for x in delta.iter_mut() {
        *x -= x.round();
    }
    let cart = frac_to_cart(lattice, delta);
    (cart[0] * cart[0] + cart[1] * cart[1] + cart[2] * cart[2]).sqrt()
}

fn union_find(parent: &mut [usize], x: usize) -> usize {
    if parent[x] != x {
        parent[x] = union_find(parent, parent[x]);
    }
    parent[x]
}

/// Every site index grouped with every other site index it exactly coincides
/// with under PBC (numerical-identity tolerance, transitive closure), plus a
/// `Finding::limitations`-ready note if any pairwise distance behind the
/// grouping used [`ResolvedLattice::minimum_image`]'s approximate fallback. Includes
/// singleton groups (an ordinary site with no coincidence partner is its own
/// one-element group) — callers that only care about actual coincidences
/// (e.g. disorder detection) filter those out themselves; callers that need
/// every site accounted for (e.g. building a whole-structure geometry
/// object) do not have to special-case them.
///
/// Shared by `diagnostics::disorder` (same-tolerance duplicate/disorder
/// detection) and `diagnostics::coordination` (grouping coincident sites
/// into one multi-species position before neighbor search) rather than each
/// re-implementing the same union-find scan.
///
/// ponytail: O(n^2) pairwise scan plus union-find; fine at v0.1's scale,
/// revisit if mikiwame is used on structures with many thousands of sites.
/// The lattice itself is resolved once (see [`ResolvedLattice`]), not once
/// per pair, so this is O(n^2) distance computations, not O(n^2) lattice
/// constructions on top of that.
pub(crate) fn coincidence_groups<S: PeriodicStructureView>(
    structure: &S,
    tolerance_angstrom: f64,
) -> (Vec<Vec<usize>>, Option<String>) {
    let resolved_lattice = ResolvedLattice::resolve(structure.lattice());
    let sites = structure.sites();
    let n = sites.len();
    let mut parent: Vec<usize> = (0..n).collect();
    let mut fallback_limitation: Option<String> = None;

    for i in 0..n {
        for j in (i + 1)..n {
            let distance = resolved_lattice.minimum_image(sites[i].fractional, sites[j].fractional);
            if fallback_limitation.is_none() {
                fallback_limitation = distance.method.limitation();
            }
            if distance.distance_angstrom < tolerance_angstrom {
                let (root_i, root_j) = (union_find(&mut parent, i), union_find(&mut parent, j));
                if root_i != root_j {
                    parent[root_i] = root_j;
                }
            }
        }
    }

    let mut groups: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for i in 0..n {
        let root = union_find(&mut parent, i);
        groups.entry(root).or_default().push(i);
    }
    (groups.into_values().collect(), fallback_limitation)
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
        let d =
            ResolvedLattice::resolve(&lattice).minimum_image([0.05, 0.0, 0.0], [0.95, 0.0, 0.0]);
        // True separation is 0.1 cell widths through the periodic boundary,
        // not 0.9 straight across the cell.
        approx(d.distance_angstrom, 0.1 * a);
        assert_eq!(d.method, PeriodicDistanceMethod::Exact);
    }

    /// Same skewed lattice/points as the naive-fallback regression below,
    /// but exercised through the public `ResolvedLattice::minimum_image`
    /// dispatcher — proves the `chematic_crystal` swap actually fixed the
    /// gap, not just that the crate's own tests claim to. Also pins that an
    /// ordinary (if strongly skewed) triclinic-ish cell like this one takes
    /// the exact path, not the fallback — the fallback is for lattices
    /// `chematic_crystal::Lattice::from_matrix` outright rejects, a
    /// stricter bar than "skewed".
    #[test]
    fn minimum_image_distance_finds_the_true_minimum_on_a_skewed_cell() {
        let lattice = [[1.0, 0.0, 0.0], [0.9, 0.1, 0.0], [0.0, 0.0, 10.0]];
        let p = [0.75, 0.75, 0.0];
        let q = [0.25, 0.25, 0.0];

        let found = ResolvedLattice::resolve(&lattice).minimum_image(p, q);
        let true_min = true_minimum_via_known_shorter_image(&lattice, p, q);

        approx(found.distance_angstrom, true_min);
        assert_eq!(found.method, PeriodicDistanceMethod::Exact);
    }

    /// Documents that the fallback path (used only when
    /// `chematic_crystal::Lattice::from_matrix` rejects the lattice — see
    /// `ResolvedLattice::minimum_image`'s doc comment) still has the
    /// historical per-axis-rounding gap. With nearly-parallel lattice
    /// vectors a and b, a legitimate periodic image (p vs. q shifted by one
    /// whole b vector) is dramatically shorter than what independent
    /// per-axis rounding finds — because that image requires a *different*
    /// integer shift on the a axis than on the b axis, which per-axis
    /// rounding of a single delta cannot produce when both components start
    /// out equal.
    ///
    /// Exercises `naive_minimum_image_distance` directly (not through
    /// `ResolvedLattice::minimum_image`) since this same lattice takes the
    /// *exact* path there (see the test above) — the fallback is reachable
    /// only via a lattice `Lattice::from_matrix` itself rejects, covered
    /// separately below.
    #[test]
    fn naive_fallback_can_still_miss_the_true_minimum_on_a_skewed_cell() {
        let lattice = [[1.0, 0.0, 0.0], [0.9, 0.1, 0.0], [0.0, 0.0, 10.0]];
        let p = [0.75, 0.75, 0.0];
        let q = [0.25, 0.25, 0.0];

        let naive = naive_minimum_image_distance(&lattice, p, q);
        let true_min = true_minimum_via_known_shorter_image(&lattice, p, q);

        assert!(
            true_min < naive * 0.5,
            "expected a much shorter periodic image ({true_min}) than the naive result ({naive})"
        );
    }

    /// A lattice `chematic_crystal::Lattice::from_matrix` rejects as
    /// near-singular (see its `MIN_CONDITION_INDICATOR`) but that
    /// mikiwame's own `LATTICE_SINGULAR` (positive-volume-only) check lets
    /// through — `ResolvedLattice::minimum_image` must fall back, not
    /// panic, and must say so via `PeriodicDistanceMethod::ApproximateFallback`.
    #[test]
    fn minimum_image_falls_back_on_a_near_singular_lattice() {
        // b is 1e-4 away from lying exactly in the a/c plane: tiny but
        // strictly positive volume, well below chematic_crystal's condition
        // threshold.
        let lattice = [[1.0, 0.0, 0.0], [0.5, 1e-4, 0.0], [0.0, 0.0, 1.0]];
        assert!(
            chematic_crystal::Lattice::from_matrix(lattice).is_err(),
            "test lattice must actually exercise the rejection path"
        );

        let d = ResolvedLattice::resolve(&lattice).minimum_image([0.0, 0.0, 0.0], [0.5, 0.0, 0.0]);
        match d.method {
            PeriodicDistanceMethod::ApproximateFallback { .. } => {}
            PeriodicDistanceMethod::Exact => panic!("expected the fallback path"),
        }
    }

    /// A lattice `chematic_crystal::Lattice::from_matrix` rejects for a
    /// short axis (below `MIN_LENGTH`) rather than near-singularity —
    /// distinct rejection reason, same fallback behavior.
    #[test]
    fn minimum_image_falls_back_on_an_extremely_short_lattice_vector() {
        let lattice = [[1.0, 0.0, 0.0], [0.0, 1e-9, 0.0], [0.0, 0.0, 1.0]];
        assert!(chematic_crystal::Lattice::from_matrix(lattice).is_err());

        let d = ResolvedLattice::resolve(&lattice).minimum_image([0.0, 0.0, 0.0], [0.0, 0.5, 0.0]);
        assert!(matches!(
            d.method,
            PeriodicDistanceMethod::ApproximateFallback { .. }
        ));
    }

    /// The true minimum-image distance between `p` and `q` on the
    /// deliberately-skewed test lattice shared by the two tests above,
    /// found by hand via the specific known-shorter image (`q` shifted by
    /// one whole `b` lattice vector) rather than a general search.
    fn true_minimum_via_known_shorter_image(
        lattice: &[[f64; 3]; 3],
        p: [f64; 3],
        q: [f64; 3],
    ) -> f64 {
        let q_shifted_by_b = [q[0], q[1] + 1.0, q[2]];
        let true_delta = [
            p[0] - q_shifted_by_b[0],
            p[1] - q_shifted_by_b[1],
            p[2] - q_shifted_by_b[2],
        ];
        let cart = frac_to_cart(lattice, true_delta);
        (cart[0] * cart[0] + cart[1] * cart[1] + cart[2] * cart[2]).sqrt()
    }
}
