# todo

## Needs a cited source before implementation (do not guess thresholds)

- [ ] `SITE_SEVERE_OVERLAP` / `SITE_UNUSUALLY_SHORT_DISTANCE` (AGENTS.md §7.3). The radius
      table itself is resolved (Cordero et al. 2008, `src/radii.rs`), but embedding it
      surfaced a narrower, separate problem: `observed_distance < covalent_radius_sum`
      false-positives on ionic bonding, demonstrated against the already-shipped perovskite
      fixture (Ti–O in SrTiO3, 1.9525 Å observed vs. 2.26 Å covalent-radii sum — a normal
      bond flagged as "unusually short"). See `docs/validation.md` for the full table
      (diamond/zinc-blende are fine; perovskite isn't) and
      `radii::tests::expected_distance_from_covalent_radii_is_unsafe_for_ionic_bonds`.
      Two paths forward, either needing its own owner decision:
      (a) oxidation-state-aware ionic radii — depends on the oxidation-state table below
      plus composition analysis (Phase 4), or
      (b) a species-independent absolute-distance floor ("no two nuclei can be closer than
      X regardless of element") — a different, narrower claim than "shorter than expected
      for these two elements," needing its own citable basis rather than an arbitrary
      fraction of the covalent-radii sum.
- [ ] Documented criterion for "extreme" lattice aspect ratio / poor conditioning
      (`LATTICE_EXTREME_ASPECT_RATIO`, `LATTICE_POORLY_CONDITIONED`) — currently only
      `LATTICE_SINGULAR` (volume ≤ 0 or numerically singular) is implemented, since that
      needs no tuned constant. Candidate now available: `chematic_crystal::Lattice`
      exposes `condition_indicator()` (volume / bounding-box-volume) with its own
      `MIN_CONDITION_INDICATOR = 1e-3` threshold and test suite — `Lattice::from_matrix`
      already rejects near-singular/very-short-axis lattices mikiwame's own
      `LATTICE_SINGULAR` currently lets through (see `structure_view.rs`'s
      `minimum_image_distance` fallback and `docs/validation.md`). Adopting this as
      mikiwame's own threshold is still an owner decision (changes what's fatal), not
      done in this round.
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
- [x] `README_ja.md` added (translation, not a design decision — was only deferred for
      that reason, per `docs/architecture.md`).
- [x] Elemental radius table decided and embedded: Cordero et al. 2008, `src/radii.rs`
      (Z=1–96, values cross-checked against MolSSI QCElemental's transcription,
      hybridization/spin-state disambiguation documented in the module doc comment).
      Not wired into any diagnostic yet — see the new blocker above this table replaced
      ("Needs a cited source" section) and `docs/validation.md`.
- [x] `chematic-crystal` 0.15.0 (published on crates.io the same day as this round)
      integrated: added as a dependency, `structure_view::minimum_image_distance` now
      delegates to `chematic_crystal::minimum_image` (exact, reciprocal-lattice-bounded
      search) instead of the old per-axis-rounding approximation, with a fallback to the
      old approximation only for lattices `Lattice::from_matrix` rejects that
      `LATTICE_SINGULAR` doesn't catch. `SITE_DUPLICATE` and disorder's coincidence
      detection both ride on this for free (same shared function). `PeriodicStructureView`
      kept as mikiwame's own input boundary rather than switched to
      `chematic_crystal::PeriodicStructure` — see `docs/chematic-prerequisites.md`'s
      2026-08-14 update for why (construction-time validation vs. mikiwame's
      diagnose-don't-refuse model). `Site`/`OwnedStructure`, the CLI JSON schema, and
      `disorder.rs`'s multi-site coincidence-group representation were deliberately left
      alone this round — chematic_crystal's native multi-species `PeriodicSite` is a
      better model than mikiwame's "two sites at the same position" convention, but
      adopting it is a separate, larger change with its own occupancy-validation
      implications, not bundled into a geometry-only swap.
