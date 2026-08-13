# Changelog

All notable changes to this project are documented here.

## [Unreleased]

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
