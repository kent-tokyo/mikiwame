# Validation

## Geometry conventions (closed-form, `src/structure_view.rs::tests`)

The internal `cell_volume`/`frac_to_cart`/`minimum_image_distance` helpers are pinned
against hand-derived closed-form values, not just exercised indirectly through the NaCl
fixture (which is cubic — a row/column transposition bug would be invisible there, since
both conventions agree on a diagonal lattice matrix):

* `cell_volume` on a cubic cell equals `a³`; on a hexagonal cell (α=β=90°, γ=120°)
  equals `a²c·sin(γ)`, derived directly from the scalar triple product.
* `frac_to_cart([1,0,0])` / `frac_to_cart([0,1,0])` land exactly on the lattice's first
  and second rows on the same non-orthogonal (hexagonal) cell — pins the row-vector
  convention.
* `minimum_image_distance` correctly wraps a pair of sites across the cell boundary
  (`0.05` vs. `0.95` fractional is `0.1` cells apart, not `0.9`).

## Known limitation, fixed by delegating to `chematic_crystal` (2026-08-14)

`minimum_image_distance` used to wrap each fractional axis independently
(`d -= round(d)` per component) rather than searching the full periodic-image
neighborhood. `naive_minimum_image_can_miss_the_true_minimum_on_a_skewed_cell` pinned
this: a lattice with nearly-parallel `a`/`b` vectors (`a=(1,0,0)`, `b=(0.9,0.1,0)`) where
a legitimate periodic image was under half the naive result.

Once `chematic-crystal` 0.15.0 shipped an exact minimum-image search (a
reciprocal-lattice-derived search box, provably sufficient, brute-force checked inside
it — see that crate's `periodic` module and `docs/rfcs/chematic_crystal_foundation.md` in
the `chematic` repo), `minimum_image_distance` was rewired to delegate to it.
`minimum_image_distance_finds_the_true_minimum_on_a_skewed_cell` runs the *same* skewed
lattice through the new code path and asserts it now finds the true minimum — the fix is
demonstrated, not just the old gap. The naive approximation is kept as
`naive_minimum_image_distance`, used only as a fallback for lattices
`chematic_crystal::Lattice::from_matrix` rejects (near-singular, or an axis shorter than
its `MIN_LENGTH`) that `input_quality`'s own `LATTICE_SINGULAR` check — fatal only for
non-positive volume — doesn't catch;
`naive_fallback_can_still_miss_the_true_minimum_on_a_skewed_cell` keeps pinning that this
fallback path still has the historical gap, since it's still reachable code.

This is one instance of a broader design point worth recording: `chematic_crystal`
validates and rejects at construction (`Lattice::from_matrix`, `PeriodicStructure::new`
return `Result`), while mikiwame's whole premise is diagnosing malformed input rather
than refusing it (`docs/architecture.md`'s fatal/non-fatal split, `INPUT_INVALID_OCCUPANCY`
being non-fatal, etc.). `PeriodicStructureView`/`Site` stay mikiwame's own types for this
reason — `chematic_crystal` is used internally, on the geometry path only, after
`input_quality`'s own checks have run. See `docs/chematic-prerequisites.md`.

## Metamorphic / invariance (`tests/metamorphic.rs`, AGENTS.md §15.3)

`analyze`'s verdict and finding-code counts (site indices inside findings are allowed to
differ, and do — documented in the test file's module comment, not silently) are checked
invariant under:

* site order (reversing the site list),
* choice of origin (shifting every fractional coordinate by a fixed vector, mod 1),
* fractional coordinates left outside `[0,1)` instead of pre-wrapped (`+3.0`/`-2.0`/`+1.0`
  offsets on one site) — this one is a direct consequence of `minimum_image_distance`'s
  wrap being translation-equivariant (`round(f+n) = round(f)+n` for integer `n`), so it
  doubles as a second, independent check on that function beyond the closed-form tests
  above,
* rigid rotation of the lattice (a 90°, floating-point-exact rotation about `z`, fractional
  coordinates unchanged),
* describing the same physical structure as a 2×1×1 supercell (16 sites instead of 8) —
  stays `StructurallyConsistent` with zero findings, i.e. supercelling a clean structure
  does not manufacture spurious `SITE_DUPLICATE`s.

Each check runs against a deliberately-broken variant (one duplicate pair, one invalid
occupancy) where that makes the invariance non-trivial, not just the clean fixture.

## Covalent radii are not a safe "expected distance" for ionic bonding

The elemental radius table decision (`tasks/todo.md`, resolved: Cordero et al. 2008,
embedded in `src/radii.rs`) was originally motivated by `SITE_SEVERE_OVERLAP` /
`SITE_UNUSUALLY_SHORT_DISTANCE` (AGENTS.md §7.3). Before implementing either, checking the
table against the already-shipped perovskite fixture
(`tests/known_good_fixtures.rs::perovskite_is_structurally_consistent`) turned up a false
positive:

| pair (fixture)        | observed distance | sum of Cordero covalent radii | naive verdict |
|------------------------|-------------------:|-------------------------------:|----------------|
| C–C (diamond)          | 1.544 Å             | 0.76 + 0.76 = 1.52 Å            | not flagged (correct) |
| Zn–S (zinc blende)     | 2.343 Å             | 1.22 + 1.05 = 2.27 Å            | not flagged (correct) |
| Ti–O (ideal perovskite)| 1.9525 Å            | 1.60 + 0.66 = 2.26 Å            | **flagged as "unusually short"** — wrong |

Ti–O at 1.9525 Å in cubic SrTiO3 is a textbook-normal bond, not an anomaly. Covalent radii
are additive estimates of *covalent* single-bond length and track the covalently-bonded
fixtures well; they systematically overestimate expected separation for ionic bonding,
where the relevant length scale is closer to (charge-state-dependent) ionic radii, not
neutral-atom covalent radii. Most inorganic crystals mikiwame targets are at least partly
ionic, so `observed_distance < covalent_radius_sum` is not a safe general-purpose
"shorter than expected" test. This is pinned as
`radii::tests::expected_distance_from_covalent_radii_is_unsafe_for_ionic_bonds` so it
can't silently regress into being re-implemented.

Consequence: the radius table itself is embedded (source, coverage, and disambiguation
choices documented in `src/radii.rs`), but `SITE_SEVERE_OVERLAP` /
`SITE_UNUSUALLY_SHORT_DISTANCE` remain unimplemented. Doing this correctly needs either
oxidation-state-aware ionic radii (which needs the still-parked oxidation-state table plus
composition analysis — Phase 4) or a species-independent absolute-floor check with its own
citable basis (a different, narrower claim than "shorter than expected for these two
elements"). See `tasks/todo.md`.

## Not yet done

* Differential comparison against pymatgen/spglib (AGENTS.md §15.4) — no Python
  materials-science stack is installed in the dev environment used so far. Not attempted
  via a scratch `pip install`, since that would modify the environment beyond this repo
  without being asked to.
* Known-good fixture set beyond the single NaCl structure (AGENTS.md §15.1) — lower
  priority than it looks: none of the currently-implemented diagnostics inspect real bond
  distances/coordination (only exact same-position-same-element coincidence), so
  additional "known-good" structures wouldn't exercise any new code path yet. Mainly
  valuable once Phase 3 (coordination/distortion) lands.
* Benchmark report (AGENTS.md §16) — no throughput/memory numbers have been measured.
