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

## Known limitation, demonstrated not just claimed

`minimum_image_distance` wraps each fractional axis independently
(`d -= round(d)` per component) rather than searching the full 3×3×3 neighboring-cell
shell. `naive_minimum_image_can_miss_the_true_minimum_on_a_skewed_cell` constructs a
lattice with nearly-parallel `a`/`b` vectors (`a=(1,0,0)`, `b=(0.9,0.1,0)`) where a
legitimate periodic image is found to be under half the naive result — the test asserts
this gap exists, so the limitation named in the function's `ponytail:` comment is
verified, not just asserted in prose. Affects `SITE_DUPLICATE` (and any future
distance-based check) only for structures with strongly skewed/acute cells; the shipped
NaCl fixture and any orthogonal-ish cell are unaffected.

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
