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
- [ ] Ideal-polyhedron reference set for §7.5 distortion metrics.
- [ ] Ambiguity criterion for coordination number: `SiteLocalEnvironment::shell_gap_ratio` is
      reported (a gap of exactly `1.0` is impossible by construction — see
      `diagnostics::coordination::resolve_shell`'s doc comment — so `None` means "one clean
      shell, nothing else nearby" and `Some(r)` means `r > 1.0` always), but nothing yet turns
      a *small* `r` into a `FindingCode::CoordinationAmbiguous` finding or a lowered
      confidence. An earlier attempt at this during the round that shipped coordination number
      was found to be simply backwards (a bug caught by testing NaCl's fully-tied 6-neighbor
      shell before trusting it — see `docs/validation.md`), and its replacement — a specific
      "how close to 1.0 counts as ambiguous" cutoff — would itself be exactly the kind of
      invented threshold AGENTS.md §21 forbids without a citable basis. Needs either a citation
      for that cutoff or a different, threshold-free ambiguity signal.
- [ ] CIF input: still blocked on chematic-mol's occupancy-aware CIF reader (referred to
      earlier as "PR2" in the chematic-crystal integration sequence), which has not landed —
      confirmed via `gh pr list --repo kent-tokyo/chematic` (only chematic-crystal's PR #318
      and the v0.15.0 release PR exist as of the coordination-number round). AGENTS.md forbids
      reimplementing CIF infrastructure inside mikiwame ("一般的なCIF基盤の重複実装" is listed
      under "mikiwameに含めない"), so this stays parked rather than worked around.
- [ ] Out-of-domain applicability detection (surfaces/interfaces/amorphous/polymers,
      AGENTS.md §5): `analyze` currently always reports `ApplicabilityLevel::FullyApplicable`
      for any input that passes input-quality checks. Parked here rather than implemented:
      every structural signal for "this is a slab, not a bulk crystal" (a large vacuum gap,
      near-2D periodicity) needs a size/ratio cutoff to decide "how much vacuum counts as a
      surface", which is exactly the kind of "常識的だから" threshold AGENTS.md §21 forbids
      inventing. Needs a documented criterion (or an explicit decision to accept a
      conservative one), same as the aspect-ratio item above.

## Phase backlog (not started)

- [ ] Phase 3: polyhedral distortion (`POLYHEDRON_*` codes) — needs the ideal-polyhedron
      reference set above. Coordination number / local environment shipped this round (see
      "Done this round"), so "ambiguous-environment handling" here now means §7.5's
      `AmbiguousCoordinationEnvironment` (shape recognition for distortion), a different
      concern from the coordination-number ambiguity item above.
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
      integrated: added as a dependency, `structure_view::minimum_image` now
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
- [x] Made the exact/fallback split from the item above observable: `minimum_image` now
      returns a `PeriodicDistance { distance_angstrom, method }` instead of a bare `f64`;
      `SITE_DUPLICATE`/`DISORDER_PRESENT`/`DISORDER_OCCUPANCY_SUM_EXCEEDS_ONE` findings
      carry a `limitations` entry when the fallback was used and stay empty on the exact
      path (previously unconditionally empty — a real evidence-first gap). Confidence
      deliberately left unlowered either way, reasoning documented in
      `docs/validation.md` (fallback can only produce false negatives for these
      tolerance checks, never a false positive on a finding that did fire). Added
      `Lattice::from_matrix`-rejection tests (near-singular, too-short-axis — two
      distinct `CrystalError` variants) and report-level tests pinning both the
      empty-on-exact and present-on-fallback cases. Did not implement
      `LATTICE_POORLY_CONDITIONED`/`LATTICE_EXTREME_ASPECT_RATIO` or lower
      component/report-level confidence — `chematic_crystal`'s construction-safety
      threshold isn't automatically a materials-anomaly threshold; see
      `docs/validation.md`.
- [x] Phase 3 (core): coordination number / local environment (AGENTS.md §7.4) —
      `diagnostics/coordination.rs`, new `MaterialDiagnosticReport::local_environment`
      (`SCHEMA_VERSION` bumped to `2`), reported as descriptive per-site data, not findings
      (coordination number for a clean structure isn't an anomaly). Method: candidate
      neighbors bounded by covalent-radius-sum + tolerance epsilon=0.4 Å (Cordero et al.
      2008 + Šidlauskaitė et al. 2026/arXiv:2601.02017 + PackFlow 2025 — the radius table's
      first real consumer, closing the loop from the radius-table round), then the actual
      shell boundary resolved as the largest relative gap in the sorted candidate-distance
      list — a pure radius-sum+epsilon cutoff alone was hand-verified *before implementation*
      to over-count CsCl (14 instead of 8) and perovskite's Ti (14 instead of 6); see
      `docs/validation.md` and `diagnostics::coordination`'s module doc comment for the full
      derivation, and `tests/known_good_fixtures.rs` / `coordination::tests` for the
      regression tests proving the gap step is load-bearing on exactly those two cases plus
      perovskite's three-shell Sr (12-fold, correctly not 12+8+6). Disordered
      (multi-species) positions and unresolvable elements are skipped with a recorded
      reason, never defaulted. `COORDINATION_UNDERCOORDINATED`/`OVERCOORDINATED` and
      polyhedral distortion not implemented (need Phase 4's oxidation states and the
      ideal-polyhedron reference set above, respectively). `FindingCode::CoordinationAmbiguous`
      was drafted and then removed before shipping — see the ambiguity-criterion item above.
