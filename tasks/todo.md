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
- [ ] Three finding codes are structurally unreachable on the CIF input path (see "Done
      this round" below for what shipped) — `INPUT_UNKNOWN_ELEMENT`,
      `INPUT_INVALID_OCCUPANCY`, `DISORDER_OCCUPANCY_SUM_EXCEEDS_ONE` — because
      `chematic-mol` rejects the CIF before mikiwame ever sees it. Not a bug (it's the
      documented reject-vs-diagnose tradeoff, same as `docs/chematic-prerequisites.md`
      already names for `chematic_crystal` generally), but a real, user-visible divergence
      between the CIF and JSON input paths worth someone eventually deciding whether to
      close (would need a raw, un-validated CIF intermediate representation upstream, same
      scope-expansion concern noted before CIF was even implemented).
- [ ] `chematic_crystal::PeriodicSite::label` (e.g. `"Na1"`) is dropped during CIF
      conversion — mikiwame's `Site` has no label field, so CIF-sourced findings say
      "site 3", not "site Na1". Not CIF-specific (JSON input never had labels either);
      adding a `label` field to `Site` is a breaking change (public fields, no constructor)
      that needs its own decision, not bundled into the CIF round.
- [ ] Non-P1 CIF symmetry expansion: mikiwame 0.3.1 rejects any CIF declaring symmetry
      beyond P1 outright (see "Done this round" below) rather than expanding it — expansion
      itself needs chematic to expose typed symmetry operations (not just an operation
      count), which doesn't exist yet on either the CIF side or as a proposal (no open
      chematic issue/PR, unlike the CIF adapter's own history). Requested shape recorded in
      `docs/chematic-prerequisites.md`'s 2026-08-15 addendum
      (`SymmetryOperation`/`expand_asymmetric_unit`). Deliberately not something mikiwame
      builds itself — real expansion needs exact affine-expression parsing, `[0,1)`
      wrapping, special-position dedup, and disorder-aware species merging, which is a
      crystal-symmetry engine, not a small CIF-adapter addition, and would duplicate
      chematic-mol's own (private) CIF symop-loop parsing to boot.
- [ ] Broader CIF differential validation: mikiwame's 5 known-good fixtures are idealized,
      not real experimental data. The Crystallography Open Database (COD,
      crystallography.net) is CC0/public-domain with an open REST API and bulk CIF
      download — no licensing concern — and is the natural source for a real corpus. Scope
      is necessarily P1-only (or pre-expanded-to-P1) until the symmetry-expansion item
      above is resolved, since most COD entries declare non-P1 symmetry and mikiwame now
      rejects those outright. Not started: needs a corpus-fetching/curation script (new,
      distinct from `scripts/differential_validation.py`, which builds its 5 fixtures
      programmatically rather than from real CIF files) and a decision on corpus size.
- [ ] `Provenance` has no field recording "this report's input was read from a CIF and
      analyzed as P1" — would help a report consumer distinguish a JSON-constructed
      structure from a CIF-derived one after the fact. `Provenance` is `#[non_exhaustive]`
      with a constructor (`Provenance::current`), so this is a non-breaking addition
      whenever it's prioritized; not bundled into the 0.3.1 correctness-fix round.
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
- [ ] Phase 6: broader differential comparison against pymatgen/spglib — coordination
      number is done (`scripts/differential_validation.py`, `docs/validation.md`); bond
      distances, symmetry info, oxidation states, and distortion metrics (AGENTS.md §15.4)
      are not. A real (non-idealized) structure corpus is no longer blocked on CIF input
      itself (that shipped in 0.3.0/0.3.1) — see the "Broader CIF differential validation"
      item above for the current, more specific blocker (P1-only scope until non-P1
      symmetry expansion lands). Benchmark report also not started.
- [ ] Fixture definitions are duplicated by hand in two places —
      `tests/known_good_fixtures.rs` and `scripts/differential_validation.py`'s
      `structure_fixture()` — with a comment asserting they're identical but nothing
      that enforces it; either could drift without the other noticing. Not fixed this
      round (not required for the differential-validation fix that prompted noticing
      it). Candidate direction: shared JSON fixture files (e.g.
      `fixtures/structures/{nacl,cscl,diamond,zinc_blende,perovskite}.json`) that both
      the Rust tests and the Python script load, and that a future CIF round-trip test
      could also reuse.
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
- [x] Differential validation (AGENTS.md §15.4), coordination-number slice:
      `scripts/differential_validation.py` compares mikiwame's coordination numbers against
      pymatgen's `CrystalNN` (both its chemically-weighted default and its documented
      geometric-only mode) on the same 5 known-good fixtures, run in an isolated venv
      (`.venv-differential-validation/`, gitignored). Result: exact agreement on all 10 site
      cases, including perovskite's O (mikiwame: 2, tightest-shell method) — going in, this
      was flagged as a likely disagreement against the "2 Ti + 4 Sr = 6" convention some
      sources use, but pymatgen's CrystalNN independently lands on 2 as well. See
      `docs/validation.md` for the full writeup, table, and scope caveats (coordination
      number only, 5 idealized structures — not bond distances, symmetry, oxidation states,
      or a broader corpus).
- [x] Fixed a real doc bug found while reviewing the coordination-number round's output:
      `SiteLocalEnvironment`'s doc comment (`src/report.rs`) referenced
      `FindingCode::CoordinationAmbiguous`, which was drafted and then removed before
      shipping (see the ambiguity-criterion item above) — the comment was never updated to
      match. Reworded to describe what actually ships (`shell_gap_ratio` as the ambiguity
      signal, no finding from it yet). Also reworded README.md/README_ja.md's "Status:
      v0.1" line, which read as a version-number claim conflicting with the crate shipping
      as 0.2.0 on crates.io — "v0.1" elsewhere in the repo (AGENTS.md, code comments) is a
      project milestone/scope label, not a version claim, and was left alone; only the
      two ambiguous user-facing "Status:" lines were reworded.
- [x] Made the differential-validation script above genuinely end-to-end. It originally
      compared pymatgen against a hand-maintained Python dict of mikiwame's *expected*
      coordination numbers — real agreement between that table and pymatgen, but no
      guarantee the table still matched what `diagnostics::coordination` actually computes,
      and no mechanism to notice if it drifted. `scripts/differential_validation.py` now
      builds the `mikiwame` CLI, runs `analyze --format json` on each fixture as a real
      subprocess, and reads `coordination_number` out of the actual returned
      `local_environment` — a real regression would show up here, not just in `cargo test`.
      Also now checks all 31 individual sites (not one representative per element) and
      exits non-zero on any mismatch, and records both the mikiwame and pymatgen versions
      actually used in its output. Result unchanged: 0 mismatches. See
      `docs/validation.md`.
- [x] Closed a real gap in the "end-to-end" claim above: the script checked that
      `target/debug/mikiwame` *existed* but never actually rebuilt it, so a stale binary
      from an earlier `cargo build` could report false agreement after a real regression
      in `diagnostics::coordination` — the commit message claimed the CLI was built; the
      code didn't do that yet. `scripts/differential_validation.py` now runs
      `cargo build --bin mikiwame` (`cwd=REPO_ROOT, check=True`) at the start of every
      invocation. Also switched the per-fixture input JSON from
      `NamedTemporaryFile(delete=False)` (leaked a file per run) to
      `tempfile.TemporaryDirectory()` (cleaned up automatically). Re-ran after both fixes:
      still 0 mismatches out of 31 sites, mikiwame 0.2.0 / pymatgen 2026.5.4.
- [x] General bug-check and refactoring pass across all of `src/`. Found and fixed:
      `src/provenance.rs`'s `radius_table_version`/`coordination_method` field docs still
      said "None in v0.1: not implemented yet" though both are populated by the
      coordination-number round; `src/radii.rs`'s module doc comment still said "not yet
      consumed by any diagnostic" directly contradicting `RADIUS_TABLE_VERSION`'s own doc
      comment 30 lines below in the same file; `src/lib.rs`'s crate-level doc overclaimed
      that coordination-number anomalies are reported "as machine-readable Findings" when
      currently none are (only descriptive `local_environment` data, since
      `FindingCode::CoordinationAmbiguous` was removed before shipping); `doctor`'s
      "enabled features: none" line was wrong — `cli` is necessarily enabled for the
      binary to run at all. No logic bugs found (checked: index bounds, `.unwrap()`/
      `.expect()` justification, the exact/fallback dispatch, `resolve_shell`'s edge
      cases) — the bugs found this pass were all doc/output-text staleness, not behavior.
      Refactored: `structure_view::minimum_image` reconstructed (and re-validated —
      matrix inversion included) `chematic_crystal::Lattice` from the raw matrix on
      *every* pairwise call, including inside the O(n^2) scans in `coincidence_groups`
      and `separation::check` — now `ResolvedLattice::resolve` builds it once per `check()`
      call and `.minimum_image()` reuses it; the old free function had no remaining
      caller so it was removed rather than kept as unexercised API surface, and its tests
      now call `ResolvedLattice` directly. Also extracted `coordination::not_computed()` to
      remove three copies of the same `SiteLocalEnvironment` skip-with-reason construction.
      Full quality gate and `scripts/differential_validation.py` re-run after: unchanged
      results (41 Rust tests, 0/31 differential mismatches).

- [x] CIF input (mikiwame 0.3.0): `src/cif.rs::read_cif`, optional `cif` feature
      (`cif = ["dep:chematic-mol", "chematic-mol/crystal"]`, not in `default`),
      against `chematic-mol` 0.16.0's published `parse_cif_periodic_structure` (the
      release PR #323 had been waiting on). Converts to `OwnedStructure`, wired into
      the CLI's `analyze` via `.cif` extension detection (`batch` stays JSONL-only,
      not line-oriented data). A CIF `chematic-mol` rejects (occupancy sum exceeded,
      missing cell tags, etc.) is a CLI error, not a diagnosed `InvalidInput` report —
      exactly the first-cut resolution decided ahead of time in this file. Multi-species
      disorder flattens into mikiwame's existing multi-site convention with no changes
      needed to `diagnostics/disorder.rs`. See `docs/chematic-prerequisites.md`'s
      2026-08-15 update and `tests/cif.rs` (including a non-cubic fixture specifically
      chosen to discriminate `chematic_crystal::Lattice::matrix()`'s row-vector
      convention from a column-vector one, since all of mikiwame's other fixtures are
      cubic and can't tell the two apart). New backlog items this surfaced (not fixed,
      see the two new entries in the "Needs a cited source" section above): three
      finding codes structurally unreachable on the CIF path, and CIF site labels
      (e.g. "Na1") dropped since `Site` has no label field.
- [x] Fixed a real correctness bug in 0.3.0, released as 0.3.1 (0.3.0 yanked):
      `read_cif_structure` (`src/bin/mikiwame.rs`) printed a stderr warning on
      `CifSymmetryStatus::UnexpandedSymmetry` and then analyzed the asymmetric-unit-only
      sites as if they were the complete cell — since `analyze`'s default output is JSON
      on stdout, an automated caller reading only stdout never saw the warning and got a
      confidently wrong report (misreported coordination numbers / near-neighbor
      distances). Now a non-P1 CIF is a CLI error, same as a CIF `chematic-mol` can't
      parse or validate — no report generated. `mikiwame::cif::read_cif` (library level)
      is unchanged: it still returns `CifSymmetryStatus` either way, so a caller doing its
      own symmetry handling isn't blocked; only the CLI's policy changed. See
      `docs/chematic-prerequisites.md`'s 2026-08-15 addendum for the typed
      `SymmetryOperation`/`expand_asymmetric_unit` API this would need from chematic to
      implement real expansion (not proposed as an actual chematic PR yet — no open
      issue exists for it).
