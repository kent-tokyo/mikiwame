# Scientific scope

This restates AGENTS.md §2/§3/§17 as the checklist implementation must satisfy. It is
not new policy; AGENTS.md is authoritative if this drifts.

## The question mikiwame answers

> For this periodic material structure, what structural anomalies, input-quality
> problems, unusual coordination environments, or composition/charge concerns exist,
> and what is the evidence for each?

## Explicitly out of scope (do not guess at these)

Thermodynamic stability, formation energy, band gap, synthesizability, synthesis
conditions/precursors, patentability, novelty-as-value-judgment, clinical/safety
suitability. mikiwame v0.1 has no DFT, no force/energy prediction, no phonon/band/DOS,
no phase diagrams, no ML interatomic potentials, no reaction/process planning.

## Claims policy (enforced in doc comments, README, and CLI text)

Allowed: "structural anomaly detected", "unusual local coordination", "unusually short
periodic distance", "no neutral formal oxidation-state assignment found", "outside the
validated domain", "evidence is ambiguous", "review recommended".

Forbidden: "unstable", "will fail to synthesize", "is a new material", "patentable",
"physically impossible", "safe", "superior performance". `Verdict::StructurallyConsistent`
must be documented at its definition site as *not* meaning stable or synthesizable.

## v0.1 structural scope

3D periodic crystals only; explicit atom positions + lattice; finite non-negative
occupancy; primarily inorganic; conventional crystallographic unit cell. Small-molecule
crystals/MOFs may be read but are not accuracy-guaranteed. Surfaces, interfaces,
amorphous, polymers, and large defect structures are out of scope or reported as low
applicability — mikiwame does not silently run its site-collision/coordination logic on
them and call the result meaningful.

## Threshold discipline (AGENTS.md §21)

A finding ships in v0.1 only if it needs no invented empirical constant, or the
constant's source is recorded in provenance. Implemented in this phase:

* `INPUT_EMPTY_STRUCTURE`, `INPUT_NONFINITE_COORDINATE`, `INPUT_INVALID_OCCUPANCY`,
  `INPUT_UNKNOWN_ELEMENT` — logical/enumerable checks, no threshold.
* `LATTICE_SINGULAR` — cell volume non-positive or lattice matrix numerically singular;
  a mathematical property, not a tuned constant.
* `SITE_DUPLICATE` — exact (within float tolerance used for round-trip identity, not a
  chemistry judgment) coincidence of two same-element sites under PBC.
* `DISORDER_PRESENT` — coincidence of two or more *different*-element sites under PBC;
  informational, not an anomaly (AGENTS.md §7.7).
* `DISORDER_OCCUPANCY_SUM_EXCEEDS_ONE` — a disordered group's occupancies summing above
  1.0; a site cannot be more than fully occupied, a logical fact not a tuned constant.
* Coordination number / local environment (AGENTS.md §7.4, `diagnostics/coordination.rs`,
  reported via `MaterialDiagnosticReport::local_environment`, not a finding — descriptive
  data present for a clean structure too). Candidate neighbors bounded by covalent radii
  (Cordero et al. 2008, `src/radii.rs`) summed pairwise plus a tolerance
  epsilon = 0.4 Å, citable via Šidlauskaitė et al. 2026 (*Determination of bonding radii
  from small-molecule crystal structures*, arXiv:2601.02017) and PackFlow (2025), which
  independently converge on the same constant for the same style of bond heuristic. That
  bound alone is not the shell boundary — see `docs/validation.md` for why a pure
  radius-sum+epsilon cutoff over-counts CsCl and perovskite's Ti, and how the actual shell
  boundary (largest relative gap in the sorted candidate-distance list) is what fixes it.
  Method name/cutoff/table version recorded in `Provenance::coordination_method` and
  `radius_table_version` (AGENTS.md §7.4's explicit requirement). Not implemented:
  `COORDINATION_UNDERCOORDINATED`/`OVERCOORDINATED` (need an oxidation-state-dependent
  expected value, Phase 4) and a `COORDINATION_AMBIGUOUS` finding (needs its own citable
  cutoff on the reported `shell_gap_ratio` — see `tasks/todo.md`).

Deferred to a later phase because they need a cited, versioned reference table before
they can carry a threshold honestly:

* `SITE_SEVERE_OVERLAP` / `SITE_UNUSUALLY_SHORT_DISTANCE` (AGENTS.md §7.3) — the
  elemental-radius table itself is now sourced and versioned (Cordero et al. 2008,
  `src/radii.rs`), but the table alone turned out not to be enough: a naive
  `observed < covalent_radius_sum` comparison false-positives on ionic bonding
  (demonstrated against the shipped perovskite fixture — see `docs/validation.md`).
  Blocked on a second, separate decision now: either oxidation-state-aware ionic radii
  (Phase 4) or a species-independent absolute-distance floor with its own citable basis.
* `LATTICE_EXTREME_ASPECT_RATIO` / `LATTICE_POORLY_CONDITIONED` — "extreme" needs a
  documented criterion, not a guessed cutoff.
* Polyhedral distortion (AGENTS.md §7.5) and composition/oxidation-state diagnostics
  (§7.6) — Phase 3/4, each needs its own cited method/table (an ideal-polyhedron reference
  set, and a formal oxidation-state table, respectively). Coordination number above and
  disorder's no-threshold subset both shipped early — neither needed one.

See `tasks/todo.md` for the list this produces.
