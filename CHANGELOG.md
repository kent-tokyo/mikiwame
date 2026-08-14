# Changelog

All notable changes to this project are documented here.

## [Unreleased]

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
