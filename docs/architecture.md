# mikiwame architecture (v0.1)

## Scope of this document

Records the Phase 0/1 design decisions. Not a general design-docs process; update in
place as the crate evolves.

## Module layout

```text
src/
├── lib.rs              analyze() / analyze_batch() entry points, crate docs
├── config.rs            AnalysisConfig
├── error.rs              MikiwameError (typed, no panics on bad input)
├── model.rs              Score01, ClosedRange, Verdict, Severity, ApplicabilityLevel, MetricCode, Unit
├── finding.rs             Finding, FindingCode, FindingScope, Evidence
├── report.rs              MaterialDiagnosticReport and its component types
├── provenance.rs          Provenance (versions, config digest, timestamps caller-supplied)
├── structure_view.rs      PeriodicStructureView trait + OwnedStructure DTO (see chematic-prerequisites.md)
└── diagnostics/
    ├── mod.rs             component pipeline: input_quality, then (if not fatal) separation, disorder
    ├── input_quality.rs   Phase 2, no-threshold checks — this is where LATTICE_SINGULAR lives too:
    │                      AGENTS.md §7.1 lists "singular/near-singular lattice" and "non-positive
    │                      cell volume" as *input quality* checks, not §7.2 geometry checks
    ├── separation.rs      Phase 2, no-threshold checks (exact-duplicate sites under PBC)
    └── disorder.rs        Phase 4 (pulled forward), no-threshold subset of §7.7: coincident
                           different-element sites (DISORDER_PRESENT, informational) and an
                           occupancy-sum-exceeds-one check reusing the same PBC coincidence
                           detection as separation.rs
```

`lattice.rs` (§7.2: aspect ratio, angles, conditioning) is not created yet — every check
listed there needs an "extreme"/"poorly conditioned" cutoff, i.e. an invented threshold,
which AGENTS.md §21 forbids without a citable basis. It will be added once such a basis
exists (see `tasks/todo.md`).

`coordination.rs`, `distortion.rs`, `composition.rs` are Phase 3/4 work, not present yet —
adding empty stubs now would be scaffolding-for-later, which AGENTS.md §21 and ponytail
both rule out. `disorder.rs` jumped the nominal phase order ahead of Phase 3
(coordination/distortion) because its one no-threshold check (occupancy-sum) has no
dependency on anything Phase 3 would add — it only needed the PBC coincidence detection
`separation.rs` already had.

## Data flow

```text
caller's structure (impl PeriodicStructureView)
        │
        ▼
   input_quality::check()  ──▶ if fatal: short-circuit, Verdict::InvalidInput
        │  ok
        ▼
   separation::check(), disorder::check()  (independent components; more join here later)
        │
        ▼
   aggregate findings ──▶ OverallAssessment (verdict, dominant findings)
        │
        ▼
   MaterialDiagnosticReport
```

`input_quality` runs first and is fatal (short-circuits everything after it, verdict
`InvalidInput`) for: empty structure, non-finite lattice, non-positive/singular cell
volume, non-finite fractional coordinates. It is *not* fatal for invalid occupancy
(`INPUT_INVALID_OCCUPANCY`) — occupancy does not participate in the geometry `separation`
and `disorder` compute, so those components still run and their results are still
meaningful. This fatal/non-fatal split is a judgment call documented here because
AGENTS.md states the principle ("入力が壊れている場合、後続診断を無理に実行しては
いけません") without enumerating which specific checks are fatal.

Verdict decision (`decide_verdict` in `lib.rs`) keys on the maximum `Severity` present,
not on which finding codes are present: an `Info`-severity finding alone (currently only
`DISORDER_PRESENT`) leaves the verdict at `StructurallyConsistent`, since AGENTS.md §7.7
is explicit that disorder is not itself an anomaly.

## Why a trait instead of a concrete struct

`PeriodicStructureView` is the boundary chosen in `docs/chematic-prerequisites.md`
because chematic's default branch has no periodic-structure type to depend on yet.
Depending on a trait rather than mikiwame's own concrete "Structure" type means:

* callers with their own structure type (including a future chematic type) implement
  the trait instead of converting into a mikiwame-owned struct;
* mikiwame never grows an independent, competing structure representation
  (AGENTS.md §4's explicit prohibition).

## Score/finding separation

Per AGENTS.md §6, `OverallAssessment` keeps `anomaly_burden`, `confidence`, and
`applicability` (in `ApplicabilityAssessment`) as separate fields — they are computed
independently and never averaged into one number. `Score01` is a newtype enforcing
`0.0..=1.0` and finiteness at construction, so an invalid score cannot be constructed,
only reported as `None`/absent.

## Deferred (named, not built)

* `coordination`/`distortion`/`composition` diagnostics — Phase 3/4. `disorder`'s
  no-threshold subset shipped early; see the module layout section above.
* CLI (`src/bin/mikiwame.rs`) — Phase 5.
* Corpus/prototype similarity — optional, post-v0.1 per AGENTS.md §10.
* `MikiwameHandoff` (gugen handoff type) — future, per AGENTS.md §18, not built now.
* `README_ja.md` — English README only for now; translation is not a design decision.
* `mikiwame-cli` split — only if the CLI grows large enough to warrant it (AGENTS.md §12).
