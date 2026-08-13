# Competitive landscape (Phase 0 note)

Existing tools that overlap partially with mikiwame's problem space:

* **pymatgen** (Python) — general materials structure toolkit; has structure
  analysis, oxidation-state guessing, coordination analysis. Broad, not
  explanation-contract-first: returns Python objects/scores, not a stable
  machine-readable finding schema with severity/confidence/applicability separated.
* **Robocrystallographer** (Python, built on pymatgen) — generates natural-language
  structure descriptions. Closest in spirit ("explain the structure"), but optimizes
  for prose readability over structured, machine-checkable evidence.
* **SMACT** (Python) — composition/oxidation-state plausibility screening. Overlaps
  with mikiwame §7.6, but is composition-only, no lattice/site/coordination diagnostics.
* **matminer** (Python) — featurization for ML, not diagnosis; produces feature
  vectors, not findings with evidence.
* **spglib** (C, with bindings) — symmetry/space-group determination. A dependency
  candidate for future symmetry-aware checks, not a competitor for diagnosis itself.
* **CHGNet / MatGL** (Python, ML interatomic potentials) — energy/force prediction.
  Answers a different question (is this stable/what is its energy) that mikiwame
  explicitly refuses to answer without doing the calculation.

None of the above is Rust-native, WASM-embeddable, or ships a single typed diagnostic
report contract (finding code + severity + confidence + applicability + evidence,
separated fields) as its primary interface. That contract, not richer prose or broader
property coverage, is mikiwame's differentiation (AGENTS.md §11).

This is a scope-orienting note, not an exhaustive feature-by-feature audit; deepen it
only if a design decision actually hinges on a specific competitor behavior.
