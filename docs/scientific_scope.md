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

Deferred to a later phase because they need a cited, versioned reference table before
they can carry a threshold honestly:

* `SITE_SEVERE_OVERLAP` / `SITE_UNUSUALLY_SHORT_DISTANCE` — needs an elemental-radius
  table with a recorded source/version (AGENTS.md §7.3).
* `LATTICE_EXTREME_ASPECT_RATIO` / `LATTICE_POORLY_CONDITIONED` — "extreme" needs a
  documented criterion, not a guessed cutoff.
* Coordination, distortion, composition/oxidation-state diagnostics — Phase 3/4, each
  needs its own cited method/table. (Disorder's no-threshold subset above shipped early —
  it didn't need one.)

See `tasks/todo.md` for the list this produces.
