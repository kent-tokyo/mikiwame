# Validation

## Geometry conventions (closed-form, `src/structure_view.rs::tests`)

The internal `cell_volume`/`frac_to_cart`/`minimum_image` helpers are pinned against
hand-derived closed-form values, not just exercised indirectly through the NaCl fixture
(which is cubic — a row/column transposition bug would be invisible there, since both
conventions agree on a diagonal lattice matrix):

* `cell_volume` on a cubic cell equals `a³`; on a hexagonal cell (α=β=90°, γ=120°)
  equals `a²c·sin(γ)`, derived directly from the scalar triple product.
* `frac_to_cart([1,0,0])` / `frac_to_cart([0,1,0])` land exactly on the lattice's first
  and second rows on the same non-orthogonal (hexagonal) cell — pins the row-vector
  convention.
* `minimum_image` correctly wraps a pair of sites across the cell boundary (`0.05` vs.
  `0.95` fractional is `0.1` cells apart, not `0.9`).

## Known limitation, fixed by delegating to `chematic_crystal` (2026-08-14)

`minimum_image_distance` used to wrap each fractional axis independently
(`d -= round(d)` per component) rather than searching the full periodic-image
neighborhood. `naive_minimum_image_can_miss_the_true_minimum_on_a_skewed_cell` pinned
this: a lattice with nearly-parallel `a`/`b` vectors (`a=(1,0,0)`, `b=(0.9,0.1,0)`) where
a legitimate periodic image was under half the naive result.

Once `chematic-crystal` 0.15.0 shipped an exact minimum-image search (a
reciprocal-lattice-derived search box, provably sufficient, brute-force checked inside
it — see that crate's `periodic` module and `docs/rfcs/chematic_crystal_foundation.md` in
the `chematic` repo), the function (renamed `minimum_image` once it started returning a
`PeriodicDistance` — see below) was rewired to delegate to it.
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

## Making the exact/fallback split observable (2026-08-14)

The exact/fallback split above was initially invisible to report readers: whichever
method computed a distance, `SITE_DUPLICATE`'s `limitations` was unconditionally empty.
That's a real gap for an evidence-first tool — a caller can't tell whether a "no
duplicate found" (or a found one) rested on the exact search or the weaker
approximation.

`structure_view::minimum_image` now returns a [`PeriodicDistance`], pairing the distance
with a `PeriodicDistanceMethod` (`Exact` or `ApproximateFallback { reason }`, `reason`
being `chematic_crystal`'s own `CrystalError` message — self-contained by that crate's
own design, reused rather than re-derived). `separation::check` and `disorder::check`
attach `PeriodicDistanceMethod::limitation()` to `SITE_DUPLICATE`, `DISORDER_PRESENT`,
and `DISORDER_OCCUPANCY_SUM_EXCEEDS_ONE` findings when the fallback was used, and leave
`limitations` empty on the exact path — pinned both ways by report-level tests in
`tests/diagnostics.rs` (`coincident_same_element_sites_are_duplicates` for the empty
case, `coincident_same_element_sites_on_a_near_singular_lattice_note_the_fallback` for
the caveat). `structure_view::tests` separately pins that `Lattice::from_matrix`
actually rejects a near-singular lattice and a too-short lattice vector (two distinct
`CrystalError` variants), so the fallback path is exercised for a documented reason, not
assumed reachable.

**Confidence was deliberately left unlowered** on findings computed via the fallback,
rather than picking an arbitrary discount (AGENTS.md §21 forbids exactly that: a
threshold/penalty invented because it "sounds reasonable"). The reasoning: the naive
per-axis-rounded distance is always a *real, achievable* periodic separation (rounding
each fractional axis to its nearest integer is still a legitimate image, just not
necessarily the shortest one) — so it can only be `>=` the true minimum, never smaller.
For both `SITE_DUPLICATE` (fires when distance < a tight numerical-identity tolerance)
and disorder's coincidence grouping (same tolerance, same direction of error), that means
the fallback's only failure mode is a **false negative** — missing a real coincidence
whose true minimum image is below tolerance but whose naive image isn't — never a false
positive. A finding that *did* fire under the fallback is exactly as certain as one found
via the exact search; there is no finding to attach a lowered confidence to for the ones
that were missed. If a future check's error direction differs (over- rather than
under-counting), this reasoning needs to be re-derived for that check, not copied.

Not built this round (explicitly out of scope, per AGENTS.md §21's "don't reuse a
neighboring threshold without its own basis" — see `tasks/todo.md`): turning
`chematic_crystal`'s rejection reason into a scored `LATTICE_POORLY_CONDITIONED` finding,
or lowering component/report-level confidence when the fallback is used. `condition_indicator()`
being a *construction safety* threshold for `chematic_crystal`'s own algorithms doesn't
by itself make it a *materials-science* anomaly threshold — that would need its own
justification even though the number is right there.

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

## Differential validation: coordination number vs. pymatgen (2026-08-14)

AGENTS.md §15.4 asks for differential comparison against pymatgen/spglib where possible.
This covers the coordination-number slice: `scripts/differential_validation.py` builds
the `mikiwame` CLI, runs `analyze --format json` on the same five structures
`tests/known_good_fixtures.rs` uses (identical lattice constants and fractional
coordinates) — a real subprocess call against the real built binary, not a re-derivation —
reads `coordination_number` out of each report's actual `local_environment`, and compares
it against pymatgen's `CrystalNN`, computed on the identical structure, in two
configurations: `CrystalNN`'s chemically-weighted default, and its documented
geometric-only mode (`distance_cutoffs=None, x_diff_weight=0, porous_adjustment=False`,
closer in spirit to mikiwame's own no-chemical-weighting method). Run inside an isolated
virtualenv (`.venv-differential-validation/`, gitignored; see the script's own header for
setup) — does not touch system Python, not wired into `cargo test` or CI.

**This is deliberately end-to-end, not a comparison against a hand-maintained expected-value
table.** An earlier version of this script hardcoded mikiwame's expected coordination
numbers in a Python dict and only checked that *those expectations* agreed with pymatgen —
which would have gone silently stale if `diagnostics::coordination` ever regressed, since
nothing would re-derive the expectations. Running the actual binary and parsing its actual
JSON output means a real regression shows up here as a mismatch, not just in `cargo test`.

Result: **exact agreement on all 31 individual sites across all 5 fixtures** (mikiwame
0.2.0 vs. pymatgen 2026.5.4), both `CrystalNN` configurations — every site checked
individually, not deduplicated by symmetry:

```text
structure    site element  mikiwame  CrystalNN(default)  CrystalNN(geometric)  agree?
-------------------------------------------------------------------------------------
NaCl         0   Na            6                   6                     6  yes
NaCl         1   Na            6                   6                     6  yes
NaCl         2   Na            6                   6                     6  yes
NaCl         3   Na            6                   6                     6  yes
NaCl         4   Cl            6                   6                     6  yes
NaCl         5   Cl            6                   6                     6  yes
NaCl         6   Cl            6                   6                     6  yes
NaCl         7   Cl            6                   6                     6  yes
CsCl         0   Cs            8                   8                     8  yes
CsCl         1   Cl            8                   8                     8  yes
diamond      0   C             4                   4                     4  yes
diamond      1   C             4                   4                     4  yes
diamond      2   C             4                   4                     4  yes
diamond      3   C             4                   4                     4  yes
diamond      4   C             4                   4                     4  yes
diamond      5   C             4                   4                     4  yes
diamond      6   C             4                   4                     4  yes
diamond      7   C             4                   4                     4  yes
zinc_blende  0   Zn            4                   4                     4  yes
zinc_blende  1   Zn            4                   4                     4  yes
zinc_blende  2   Zn            4                   4                     4  yes
zinc_blende  3   Zn            4                   4                     4  yes
zinc_blende  4   S             4                   4                     4  yes
zinc_blende  5   S             4                   4                     4  yes
zinc_blende  6   S             4                   4                     4  yes
zinc_blende  7   S             4                   4                     4  yes
perovskite   0   Sr           12                  12                    12  yes
perovskite   1   Ti            6                   6                     6  yes
perovskite   2   O             2                   2                     2  yes
perovskite   3   O             2                   2                     2  yes
perovskite   4   O             2                   2                     2  yes

mikiwame version: 0.2.0
pymatgen version: 2026.5.4
0 mismatch(es) out of 31 sites
```

The perovskite-O case is the interesting one: mikiwame reports O as 2-coordinate (its
tightest, most clearly separated shell — 2 collinear Ti neighbors at 1.9525 Å; see
`diagnostics::coordination`'s module doc comment), which differs from the "2 Ti + 4 Sr =
6" combined count some crystallography references use for O in perovskites. Going in,
this was flagged as an expected, methodology-driven disagreement worth documenting either
way — but pymatgen's `CrystalNN` (in both configurations) independently lands on the same
2, not 6. That doesn't make the alternate "6" convention wrong (it's answering a related
but different question — total near-neighbor count vs. tightest coordination shell — see
`tests/known_good_fixtures.rs::perovskite_is_structurally_consistent`'s comment for that
distinction) — it does mean two independent geometric methods agree on what the *tightest
shell* boundary is, which is the specific claim mikiwame's coordination number makes.

pymatgen logs `UserWarning`s when no oxidation states are set ("cannot locate an
appropriate radius, covalent or atomic radii will be used") — expected and consistent:
with no oxidation states given, `CrystalNN` falls back to covalent radii for its distance
checks, the same category of reference data (Cordero et al. 2008 specifically, on
mikiwame's side) both tools are working from.

Scope of this comparison: coordination number only, on 5 idealized high-symmetry
structures. Not covered: bond distances, symmetry/space-group information, oxidation
states, distortion metrics (AGENTS.md §15.4 lists these too), skewed/low-symmetry
structures, real experimental CIFs, or a benchmark corpus. A single clean agreement across
5 textbook structures is meaningful evidence the method isn't obviously wrong, not a
substitute for broader validation against a real, non-idealized corpus.

## Not yet done

* Broader differential validation: bond distances, symmetry information, oxidation
  states, distortion metrics (AGENTS.md §15.4); a larger/less idealized structure corpus.
  CIF input itself shipped (0.3.0/0.3.1, `src/cif.rs`) and is no longer the blocker, but
  mikiwame only accepts P1 (or already-expanded) CIFs — it rejects any CIF declaring
  symmetry beyond P1 outright rather than analyzing an incomplete asymmetric unit (see
  `docs/chematic-prerequisites.md`'s 2026-08-15 addendum) — so a broader corpus is itself
  scoped to P1-only structures until real symmetry expansion lands upstream. See
  `tasks/todo.md`.
* Known-good fixture set beyond the five current structures (CsCl, NaCl, diamond, zinc
  blende, perovskite; AGENTS.md §15.1) — wurtzite/rutile/spinel/graphite are deferred
  pending a cited free-positional-parameter source (see `fixtures/README.md`).
* Benchmark report (AGENTS.md §16) — no throughput/memory numbers have been measured.
