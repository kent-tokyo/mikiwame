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

## Update (2026-08-15): CIF input shipped

`chematic-mol` 0.16.0 published the CIF adapter this document's Phase 0 "requested types"
section sketched as `read_cif_periodic` (below) — `chematic_mol::cif::parse_cif_periodic_structure`,
occupancy/disorder-preserving, plus an explicit `CifSymmetryStatus` distinguishing a
genuinely-P1 file from one whose declared symmetry it did not expand. `mikiwame` now
depends on it behind an optional `cif` feature (`src/cif.rs`, `mikiwame::cif::read_cif`).

Two differences from the original sketch, both deliberate:

* `read_cif` returns mikiwame's own `OwnedStructure`, not a raw
  `chematic_crystal::PeriodicStructure` — the reject-vs-diagnose tension the update above
  documents applies here too: a CIF `chematic-mol` cannot parse or validate is a CLI error,
  not a diagnosed `InvalidInput` report, so the adapter's job ends at handing back a
  structure through mikiwame's own boundary, same as every other input path.
* No new mikiwame-specific error type — `chematic_mol::cif::CifPeriodicError` is reused
  directly (re-exported from `mikiwame::cif`) rather than wrapped, since its `Display` is
  already self-contained.

Concrete, user-visible consequence of the reject-at-construction model: three finding codes
are structurally unreachable on the CIF input path — `INPUT_UNKNOWN_ELEMENT`,
`INPUT_INVALID_OCCUPANCY`, `DISORDER_OCCUPANCY_SUM_EXCEEDS_ONE` — because a CIF that would
trigger any of them fails to parse at all. See `src/cif.rs`'s module doc comment and
`tasks/todo.md`.

## Update (2026-08-15): non-P1 CIF rejected, not analyzed (0.3.0 → 0.3.1)

`chematic-crystal`'s own crate doc comment states it is out of scope for "symmetry (space
groups, Wyckoff positions, Niggli reduction)", and `chematic-mol`'s CIF adapter confirms
this at the CIF level: `CifSymmetryStatus::UnexpandedSymmetry` exists specifically because
symmetry operations are never expanded, but the adapter only exposes an `operation_count`
for a declared symop loop — not the operator strings themselves (e.g. `-x, y, -z+1/2`).

0.3.0 shipped treating this as a warning: the CLI printed a note to stderr and proceeded to
analyze the asymmetric-unit-only sites as if they were the complete cell. This was wrong —
`analyze`'s default output is JSON on stdout, so an automated caller reading only stdout
(the normal case for a machine-readable diagnostic tool) never saw the warning and got a
confidently wrong report (coordination numbers and near-neighbor distances computed from an
incomplete structure). 0.3.1 rejects non-P1 CIF input outright instead (`src/cif.rs`'s
`read_cif` is unchanged and still returns `CifSymmetryStatus` either way — only the CLI's
policy changed, in `src/bin/mikiwame.rs::read_cif_structure`). See `CHANGELOG.md`.

**Requested chematic types (for a future PR — not implemented here, same status as every
other item in this document)**, to make real symmetry expansion possible without mikiwame
re-implementing CIF symmetry-loop parsing or a space-group table itself:

```rust
// chematic-crystal: a typed symmetry operation and its application —
// no space-group database needed, just the operator as literally given.
pub struct SymmetryOperation {
    pub rotation: [[i32; 3]; 3],
    pub translation: [Rational; 3], // exact fractions, e.g. 1/2, 1/4
}

impl SymmetryOperation {
    pub fn apply(&self, coord: FractionalCoord) -> FractionalCoord;
}

// Applies every operation to every asymmetric-unit site, wraps into [0,1),
// deduplicates special positions, and merges species/occupancy at sites the
// operations map onto each other -- the parts that make this more than a
// "small symop-string parser" (exact affine-expression parsing, special-
// position dedup, disorder-aware merging, fail-closed on a malformed
// operator) and therefore not something to build inside mikiwame itself.
pub fn expand_asymmetric_unit(
    structure: &PeriodicStructure,
    operations: &[SymmetryOperation],
    tolerance: f64,
) -> Result<PeriodicStructure, CrystalError>;
```

```rust
// chematic-mol: parse a CIF's symop loop (modern or legacy tag) into
// SymmetryOperation instead of only counting rows.
pub struct CifPeriodicResult {
    pub structure: PeriodicStructure,
    pub symmetry: CifSymmetryStatus,
    pub symmetry_operations: Vec<SymmetryOperation>, // empty when P1
}
```

With this, mikiwame's own symmetry handling would be exactly
`chematic_crystal::expand_asymmetric_unit(&result.structure, &result.symmetry_operations, tol)`
— no CIF or symmetry-operator syntax of its own. Not proposed as an actual PR to
`kent-tokyo/chematic` yet (no open issue exists for it, unlike the CIF adapter itself,
which had an already-merged PR waiting only on a release) — recorded here so the shape is
decided ahead of time, same as this document's other "requested types" sections.

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
