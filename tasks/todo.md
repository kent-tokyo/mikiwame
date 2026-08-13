# todo

## Blocked on owner decision (AGENTS.md §22)

- [ ] Review `docs/chematic-prerequisites.md` and decide whether/when to open the
      `chematic` PR for `Lattice`/`PeriodicStructure`/periodic-neighbor-search/occupancy
      CIF reading. Not started; separate repo/PR per AGENTS.md §4.
- [ ] Decide the source for an elemental radius table, needed for `SITE_SEVERE_OVERLAP` /
      `SITE_UNUSUALLY_SHORT_DISTANCE` (AGENTS.md §7.3). Candidate: Cordero et al.,
      *Covalent radii revisited*, Dalton Trans. 2008, 2832–2838 — the table used (cited or
      re-derived) by ASE (`ase.data.covalent_radii`), pymatgen, and most other open-source
      chem/materials tools under permissive licenses. Not implemented here: AGENTS.md §22
      lists "利用候補データのライセンス/再配布条件が不明" as a stop-and-report condition,
      and embedding ~90 numeric values from memory or an unverified fetch is exactly the
      kind of "plausible-sounding" data §21 warns against without an owner-reviewed source.

## Needs a cited source before implementation (do not guess thresholds)

- [ ] Documented criterion for "extreme" lattice aspect ratio / poor conditioning
      (`LATTICE_EXTREME_ASPECT_RATIO`, `LATTICE_POORLY_CONDITIONED`) — currently only
      `LATTICE_SINGULAR` (volume ≤ 0 or numerically singular) is implemented, since that
      needs no tuned constant.
- [ ] Formal oxidation-state table (source + version) for §7.6 composition/charge checks.
- [ ] Neighbor-definition method (name + cutoff rule + provenance) for §7.4 coordination.
- [ ] Ideal-polyhedron reference set for §7.5 distortion metrics.
- [ ] Out-of-domain applicability detection (surfaces/interfaces/amorphous/polymers,
      AGENTS.md §5): `analyze` currently always reports `ApplicabilityLevel::FullyApplicable`
      for any input that passes input-quality checks. Parked here rather than implemented:
      every structural signal for "this is a slab, not a bulk crystal" (a large vacuum gap,
      near-2D periodicity) needs a size/ratio cutoff to decide "how much vacuum counts as a
      surface", which is exactly the kind of "常識的だから" threshold AGENTS.md §21 forbids
      inventing. Needs a documented criterion (or an explicit decision to accept a
      conservative one), same as the aspect-ratio item above.

## Phase backlog (not started)

- [ ] Phase 3: coordination number, local environment summary, `COORDINATION_*` codes.
- [ ] Phase 3: polyhedral distortion (`POLYHEDRON_*` codes), ambiguous-environment handling.
- [ ] Phase 4: composition/oxidation-state plausibility.
- [ ] Phase 4: occupancy/disorder diagnostics beyond the input-quality sum/range checks
      already in `diagnostics/input_quality.rs`.
- [ ] Phase 6: known-good fixture set beyond the single NaCl-style fixture in `tests/`.
- [ ] Phase 6: metamorphic tests (rotation/translation/permutation/supercell invariance).
- [ ] Phase 6: differential comparison against pymatgen/spglib (no Python materials
      stack is installed in the dev environment used so far — see `docs/validation.md`),
      benchmark report.
- [ ] CLI follow-ups (not blocking, all Green when picked up): `mikiwame batch` fails the
      whole run on the first malformed JSONL line instead of reporting per-line and
      continuing; exit codes are currently just 0 (ran) / 1 (usage or I/O error) and don't
      reflect `Verdict` (e.g. for CI gating on `StrongAnomalyDetected`) — that's a product
      decision AGENTS.md doesn't specify, left alone rather than guessed at.
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
- [x] Closed-form geometry unit tests (`cell_volume`, `frac_to_cart`,
      `minimum_image_distance`) on a non-orthogonal lattice, plus a test demonstrating
      (not just claiming) the naive minimum-image ceiling. `docs/validation.md` created.
- [x] `INPUT_UNKNOWN_ELEMENT`: `Site::element` checked against the 118 IUPAC element
      symbols (plain enumerable fact, not a measured constant — no citation needed unlike
      the radius/oxidation tables above). Non-fatal, same reasoning as invalid occupancy.
- [x] Phase 5: CLI (`analyze`/`batch`/`explain`/`doctor`) in `src/bin/mikiwame.rs`, behind
      a `cli` Cargo feature (on by default) so the library itself never requires
      `serde_json`. JSON structure-file schema documented in that file's module doc
      comment and in `README.md`.
