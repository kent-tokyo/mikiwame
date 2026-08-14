# chematic prerequisites for mikiwame

## Update (2026-08-14): chematic-crystal shipped, trait kept anyway

`chematic-crystal` 0.15.0 now provides everything requested below (`Lattice`,
`PeriodicStructure`, `PeriodicSite` with multi-species occupancy for disorder, exact
minimum-image PBC distance, periodic neighbor search, diagonal supercells). mikiwame now
depends on it (`Cargo.toml`) and `structure_view::minimum_image_distance` delegates to
`chematic_crystal::minimum_image` — see `docs/validation.md`.

`PeriodicStructureView`/`Site` were **not** deleted in favor of `chematic_crystal`'s own
types, and the original reason below ("chematic has nothing to depend on") is no longer
why. The reason that survives: `chematic_crystal::PeriodicStructure::new` and
`PeriodicSite::new` *validate and reject* malformed input (`Result`, `Err` on negative or
over-summed occupancy, empty species, non-finite coordinates) — but mikiwame's entire
premise is to accept a structure exactly as given and *diagnose* what's wrong with it,
not refuse it (`INPUT_INVALID_OCCUPANCY` and `DISORDER_OCCUPANCY_SUM_EXCEEDS_ONE` are
both non-fatal findings today; a caller handing mikiwame a structure with occupancy 1.4
expects a report back, not a constructor error). A `chematic_crystal::PeriodicStructure`
literally cannot represent that input. So `PeriodicStructureView` stays mikiwame's public
input boundary — the only type that can hold a structure mikiwame is supposed to
diagnose — and `chematic_crystal` is used internally, for geometry only, after
`input_quality`'s own checks (see `diagnostics/mod.rs`'s pipeline order).

The rest of this document is the original Phase 0 finding, left as-is for history.

## Finding (Phase 0 investigation, 2026-08-13)

Investigated `chematic` at its GitHub default branch (`main`, checked out locally as
`chematic-canon-diag`, commit `5c51bbb`, workspace version `0.8.0`, edition 2024,
MSRV 1.85). Also checked `chematic-release-v0.8.1` (release branch) for completeness.

`chematic` is a **molecular** cheminformatics library (SMILES/InChI canonicalization,
2D/3D conformers, force fields, fingerprints, reactions). It has **no periodic-materials
foundation**:

* No `PeriodicStructure` or equivalent type.
* `crates/chematic-mol/src/cif.rs` has a `UnitCell` (cell lengths/angles, volume,
  frac↔cart conversion) but it exists only to place atoms of a single bonded
  `Molecule` read from a small-molecule CIF. Its own doc comment states symmetry
  operations are **not** performed. No occupancy, no multi-species disordered
  sites, no general `Lattice` usable outside CIF parsing.
* No periodic neighbor search / minimum-image / PBC distance API.
* No site representation with fractional occupancy.
* No space-group / symmetry API beyond writing `P 1` on CIF export.

This matches AGENTS.md §4 "必要な基盤が存在しない場合". Per §4 and §21, mikiwame does
**not** implement a large crystal-structure foundation, and this repository does not
modify any `chematic` checkout.

## Stop-and-report (AGENTS.md §22, item 1)

1. **What was found**: chematic main has no periodic/occupancy-aware structure API.
2. **Why it matters**: mikiwame's core value (site-level, occupancy-aware, PBC-aware
   diagnostics) cannot sit on top of chematic's molecular `Molecule`/`UnitCell` without
   distorting their meaning (no occupancy field, no periodic image search, symmetry
   deliberately unhandled).
3. **Minimal fix**: mikiwame defines its own small read-only trait
   (`PeriodicStructureView`, below) plus owned DTOs for tests/CLI input. No new type
   is added to chematic in this round.
4. **Alternative**: implement full periodic-structure infra inside mikiwame now. Rejected:
   AGENTS.md §4 and §3 explicitly forbid re-implementing structure basics inside
   mikiwame ("大規模な結晶構造基盤を実装してはいけません", "pymatgen全体のRust再実装" is
   out of scope).
5. **Recommendation**: adopt the trait/DTO boundary below now; propose the matching
   types to chematic as a separate repository/PR once an owner reviews this document.
6. **Additional work if chematic adds this later**: an adapter crate/module in mikiwame
   mapping chematic's future periodic type to `PeriodicStructureView`, replacing the
   local DTO as the primary construction path. Public report schema is unaffected.
7. **Safe to continue now**: all of Phase 1 (foundation types) and the no-threshold
   subset of Phase 2 (input quality, lattice singularity, exact-duplicate site
   detection) — none of it depends on chematic.

## Requested chematic types (for a future, separate PR — not implemented here)

```rust
// crystallographic lattice, independent of any bonded Molecule
pub struct Lattice {
    pub matrix: [[f64; 3]; 3], // row vectors, Angstrom
}

impl Lattice {
    pub fn volume(&self) -> f64;
    pub fn frac_to_cart(&self, frac: [f64; 3]) -> [f64; 3];
    pub fn cart_to_frac(&self, cart: [f64; 3]) -> [f64; 3];
}

// a periodic structure: lattice + sites, sites may be partially occupied
pub struct PeriodicStructure {
    pub lattice: Lattice,
    pub sites: Vec<PeriodicSite>,
}

pub struct PeriodicSite {
    pub element: Element,
    pub fractional: [f64; 3],
    pub occupancy: f64, // may be < 1.0 (disorder), multiple species per site possible
}

// periodic (minimum-image or full-shell) neighbor search under PBC
pub fn periodic_neighbors(
    structure: &PeriodicStructure,
    site: usize,
    cutoff: f64,
) -> Vec<PeriodicNeighbor>;

// general CIF reader that preserves occupancy and does not require bonding
pub fn read_cif_periodic(input: &str) -> Result<PeriodicStructure, CifError>;
```

These are conceptual signatures, not a finished API design — ownership, allocation,
WASM-compatibility, and API stability need review by chematic's owner before
implementation, per AGENTS.md §4.

## What mikiwame uses instead, today

`src/structure_view.rs` defines a minimal read-only trait mikiwame's diagnostics
program against, plus an owned `OwnedStructure` DTO used by tests, fixtures, and the
CLI's JSON input. See that module for the exact shape. This boundary is intentionally
small and will be superseded by a chematic adapter if/when the above lands.
