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

## Not yet done

* Differential comparison against pymatgen/spglib (AGENTS.md §15.4) — no Python
  materials-science stack is installed in the dev environment used so far. Not attempted
  via a scratch `pip install`, since that would modify the environment beyond this repo
  without being asked to.
* Known-good fixture set beyond the single NaCl structure (AGENTS.md §15.1).
* Metamorphic tests (rotation/translation/permutation/supercell invariance, AGENTS.md
  §15.3).
* Benchmark report (AGENTS.md §16) — no throughput/memory numbers have been measured.
