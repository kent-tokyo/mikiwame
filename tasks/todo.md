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
- [ ] Phase 4: remaining §7.7 disorder items beyond the no-threshold subset now shipped
      (`DISORDER_PRESENT`, `DISORDER_OCCUPANCY_SUM_EXCEEDS_ONE` in `diagnostics/disorder.rs`)
      — specifically "disorderによって距離診断が不確かになる場合" (lowering confidence of
      *other* diagnostics near a disorder group). Nothing to discount yet: no
      distance-based diagnostic beyond exact-coincidence detection exists to lower
      confidence on. Revisit once one does, rather than inventing a discount factor now.
- [ ] Phase 6: wurtzite, rutile, spinel, graphite fixtures — deferred from
      `tests/known_good_fixtures.rs` (which now covers CsCl, diamond, zinc blende,
      perovskite) because each needs at least one free internal positional parameter
      (or, for graphite, was deferred alongside them for consistency) sourced from a
      citation rather than memory. See `fixtures/README.md`.
- [ ] Phase 6: differential comparison against pymatgen/spglib (no Python materials
      stack is installed in the dev environment used so far — see `docs/validation.md`),
      benchmark report.
- [ ] CLI exit codes are currently just 0 (ran) / 1 (usage or I/O error) and don't reflect
      `Verdict` (e.g. for CI gating on `StrongAnomalyDetected`). Left alone rather than
      guessed at: that's a genuine product decision (different callers would want
      different behavior) AGENTS.md doesn't specify, unlike `batch`'s per-line parse
      handling (fixed — see "Done this round").
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
- [x] Phase 6 (partial): metamorphic/invariance tests in `tests/metamorphic.rs` — site
      order, origin shift, out-of-range fractional coordinates, lattice rotation, and
      supercell invariance. See `docs/validation.md`.
- [x] Phase 6 (partial): known-good fixtures for CsCl, diamond, zinc blende, and ideal
      perovskite in `tests/known_good_fixtures.rs`. See `fixtures/README.md` for why
      wurtzite/rutile/spinel/graphite are deferred instead.
- [x] Phase 4 (partial, pulled forward): disorder no-threshold subset —
      `DISORDER_PRESENT` (informational) and `DISORDER_OCCUPANCY_SUM_EXCEEDS_ONE` in
      `diagnostics/disorder.rs`, reusing the PBC coincidence detection from
      `separation.rs`. `decide_verdict` updated so `Info`-severity findings alone don't
      move the verdict off `StructurallyConsistent`.
- [x] `mikiwame batch` no longer aborts the whole run on one malformed JSONL line — skips
      it with a warning to stderr and a final skipped-count summary, matching
      `analyze_batch`'s own per-structure guarantee extended to file parsing. Fails only
      if every line in the file is unparseable.
