# todo

## Blocked on owner decision (AGENTS.md §22)

- [ ] Review `docs/chematic-prerequisites.md` and decide whether/when to open the
      `chematic` PR for `Lattice`/`PeriodicStructure`/periodic-neighbor-search/occupancy
      CIF reading. Not started; separate repo/PR per AGENTS.md §4.

## Needs a cited source before implementation (do not guess thresholds)

- [ ] Elemental radius table (source + version) for `SITE_SEVERE_OVERLAP` /
      `SITE_UNUSUALLY_SHORT_DISTANCE` (AGENTS.md §7.3).
- [ ] Documented criterion for "extreme" lattice aspect ratio / poor conditioning
      (`LATTICE_EXTREME_ASPECT_RATIO`, `LATTICE_POORLY_CONDITIONED`) — currently only
      `LATTICE_SINGULAR` (volume ≤ 0 or numerically singular) is implemented, since that
      needs no tuned constant.
- [ ] Formal oxidation-state table (source + version) for §7.6 composition/charge checks.
- [ ] Neighbor-definition method (name + cutoff rule + provenance) for §7.4 coordination.
- [ ] Ideal-polyhedron reference set for §7.5 distortion metrics.

## No threshold needed, just not built yet this round

- [ ] `INPUT_UNKNOWN_ELEMENT`: validate `Site::element` against a static periodic-table
      symbol list (AGENTS.md §7.1 "不明元素"). No threshold involved, just not in the
      advisor-scoped cut for this pass; `Site::element` is currently an unvalidated
      `String`.
- [ ] Out-of-domain applicability detection (surfaces/interfaces/amorphous/polymers,
      AGENTS.md §5): `analyze` currently always reports `ApplicabilityLevel::FullyApplicable`
      for any input that passes input-quality checks — it does not yet try to recognize
      these structure classes at all.

## Phase backlog (not started)

- [ ] Phase 3: coordination number, local environment summary, `COORDINATION_*` codes.
- [ ] Phase 3: polyhedral distortion (`POLYHEDRON_*` codes), ambiguous-environment handling.
- [ ] Phase 4: composition/oxidation-state plausibility.
- [ ] Phase 4: occupancy/disorder diagnostics beyond the input-quality sum/range checks
      already in `diagnostics/input_quality.rs`.
- [ ] Phase 5: CLI (`analyze`, `batch`, `explain`, `doctor`), `src/bin/mikiwame.rs` is
      not yet created.
- [ ] Phase 6: known-good fixture set beyond the single NaCl-style fixture in `tests/`,
      metamorphic tests (rotation/translation/permutation/supercell invariance),
      differential comparison against pymatgen/spglib, benchmark report.
- [ ] Phase 7: release checklist, docs.rs/crates.io checks, semver audit.

## Done this round

- [x] Phase 0: chematic investigation, architecture/scientific_scope/competitors/
      chematic-prerequisites docs.
- [x] Phase 1: crate skeleton, error type, model types (`Score01`, `ClosedRange`,
      `Verdict`, `Severity`, `ApplicabilityLevel`), `PeriodicStructureView` +
      `OwnedStructure`, report/finding/provenance types.
- [x] Phase 2 (no-threshold subset only): `INPUT_EMPTY_STRUCTURE`,
      `INPUT_NONFINITE_COORDINATE`, `INPUT_INVALID_OCCUPANCY`, `LATTICE_SINGULAR`,
      `SITE_DUPLICATE`.
