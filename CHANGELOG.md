# Changelog

All notable changes to this project are documented here.

## [Unreleased]

## [0.3.1] - 2026-08-15

### Fixed

- 0.3.0 was yanked: the CLI warned but continued analyzing non-P1 CIF files whose symmetry
  operations had not been expanded, potentially producing diagnostics from the asymmetric
  unit rather than the complete unit cell. 0.3.1 now rejects such inputs. This was
  invisible to any caller that only read `analyze`'s default stdout JSON output, since the
  warning was stderr-only. The library-level `mikiwame::cif::read_cif` is unchanged — it
  still reports `CifSymmetryStatus` either way; only the CLI's policy changed.

## [0.3.0] - 2026-08-15

### Added

- CIF file input, behind a new optional `cif` Cargo feature (not in `default` — unlike
  `cli`, it pulls in `chematic-mol`'s full dependency tree). `mikiwame::cif::read_cif`
  parses a CIF via `chematic-mol` 0.16.0's occupancy/disorder-preserving
  `parse_cif_periodic_structure` adapter and converts it into mikiwame's own
  `OwnedStructure`. The CLI's `analyze` accepts a `.cif` path directly (extension-based
  detection); `batch` stays JSONL-only. A CIF `chematic-mol` cannot parse or validate
  (bad occupancy, missing cell parameters, etc.) is a CLI error, not a diagnosed
  `InvalidInput` report — a deliberate first-cut scope decision, see `src/cif.rs`'s
  module doc comment. A CIF declaring symmetry beyond P1 is accepted (symmetry
  operations are not expanded) with a warning, since the returned sites are then only
  the asymmetric unit. `doctor` reports whether CIF support is compiled in.

### Changed

- `chematic-core`/`chematic-crystal` bumped to 0.16.0 (required by `chematic-mol`
  0.16.0's `crystal` feature).

### Known limitations

- Three finding codes are structurally unreachable on the CIF input path
  (`INPUT_UNKNOWN_ELEMENT`, `INPUT_INVALID_OCCUPANCY`,
  `DISORDER_OCCUPANCY_SUM_EXCEEDS_ONE`) — `chematic-mol` rejects the CIF before mikiwame
  ever constructs a structure from it. Use the JSON input path for diagnosis of a
  malformed structure.
- CIF site labels (e.g. `"Na1"`) are dropped — mikiwame's `Site` has no label field, so
  CIF-sourced findings reference sites by index only, same as JSON input.

## [0.2.1] - 2026-08-15

### Added

- `Cargo.toml` `categories`/`keywords` for crates.io discoverability:
  `science::materials`, `science::computational-chemistry`, `command-line-utilities`;
  `crystallography`, `materials-science`, `crystal-structure`, `diagnostics`,
  `cheminformatics`.
- `scripts/differential_validation.py`: an end-to-end differential check of
  coordination number against pymatgen's `CrystalNN`. Rebuilds and runs the actual
  `mikiwame` CLI as a subprocess (not a hand-maintained expected-value table) on the 5
  known-good fixtures; exact agreement on all 31 sites as of writing. Not wired into
  `cargo test`/CI — a manually-reproduced Python check, see the script's own header and
  `docs/validation.md` for setup and results.

### Fixed

- `SiteLocalEnvironment`'s doc comment (`src/report.rs`) referenced
  `FindingCode::CoordinationAmbiguous`, which had been drafted and then removed before
  0.2.0 shipped — the comment was never updated to match.
- README.md/README_ja.md's "Status: v0.1" line reworded — it read as a version-number
  claim conflicting with the crate shipping as 0.2.0.

## [0.2.0] - 2026-08-14

### Added

- Coordination number / local environment (AGENTS.md §7.4):
  `MaterialDiagnosticReport::local_environment`, one `SiteLocalEnvironment` per resolvable
  site (coordination number, neighbor-species breakdown, shell gap ratio). Reported as
  descriptive data, not findings — a coordination number isn't itself an anomaly. Method:
  candidate neighbors bounded by covalent-radius-sum (Cordero et al. 2008) plus a
  0.4 Å tolerance (Šidlauskaitė et al. 2026, arXiv:2601.02017; PackFlow 2025), then the
  actual coordination shell resolved as the largest relative gap in the sorted
  candidate-distance list — the radius-sum bound alone was hand-verified, before
  implementation, to over-count CsCl (14 instead of 8) and perovskite's Ti (14 instead of
  6); see `docs/validation.md`. Disordered (multi-species) positions and elements outside
  Cordero's Z=1-96 coverage are skipped with a recorded reason.
- `chematic-core` added as a direct dependency (`chematic_core::Element`, used to resolve
  covalent radii by symbol for the coordination check above).

### Changed

- **Breaking**: `MaterialDiagnosticReport` gained a required `local_environment` field;
  `SCHEMA_VERSION` bumped to `2`. A report serialized under schema `1` cannot be
  deserialized as-is.
- `structure_view`'s PBC-coincidence grouping (previously private to `diagnostics::disorder`)
  is now `structure_view::coincidence_groups`, shared with `diagnostics::coordination`
  (which uses it to merge disordered positions into one multi-species
  `chematic_crystal::PeriodicSite` before neighbor search).

## [0.1.1] - 2026-08-14

### Changed

- `structure_view::minimum_image` now delegates to `chematic-crystal` 0.15.0's exact,
  reciprocal-lattice-bounded minimum-image search instead of the previous per-axis-
  rounding approximation, which could miss the true minimum image on skewed cells.
  `SITE_DUPLICATE` and disorder's coincidence detection both benefit, since both call
  this shared function. The old approximation is kept as a fallback for lattices
  `chematic_crystal::Lattice::from_matrix` rejects (near-singular, or an axis shorter
  than its minimum length) that `LATTICE_SINGULAR` doesn't catch.
- `minimum_image` returns which method computed the result (`PeriodicDistanceMethod`);
  `SITE_DUPLICATE`, `DISORDER_PRESENT`, and `DISORDER_OCCUPANCY_SUM_EXCEEDS_ONE`
  findings now note in `limitations` when the fallback method was used, and stay empty
  on the exact path. See `docs/validation.md` for why finding confidence is left
  unchanged either way.
- Added `chematic-crystal` as a required dependency.

## [0.1.0]

### Added

- Phase 0: `chematic` investigation, `docs/architecture.md`,
  `docs/scientific_scope.md`, `docs/competitors.md`, `docs/chematic-prerequisites.md`.
- Phase 1: crate foundation — `PeriodicStructureView` trait and `OwnedStructure` DTO,
  `MaterialDiagnosticReport` and its component types, typed `MikiwameError`,
  `Score01`/`ClosedRange` bounded numeric types, JSON schema (`schema_version = 1`).
- Phase 2 (no-threshold subset): `analyze`/`analyze_batch`, with findings
  `INPUT_EMPTY_STRUCTURE`, `INPUT_NONFINITE_LATTICE`, `INPUT_NONFINITE_COORDINATE`,
  `INPUT_INVALID_OCCUPANCY`, `LATTICE_SINGULAR`, `SITE_DUPLICATE`.
- Known-good NaCl fixture, synthetic-anomaly tests, JSON round-trip test.
