# mikiwame (見極め)

Explainable materials structure diagnostics for periodic crystal structures, in Rust.

`mikiwame` looks at a 3D periodic crystal structure and explains **what** is
structurally unusual about it, **where**, and **on what evidence** — not a single
opaque score. See [`docs/scientific_scope.md`](docs/scientific_scope.md) for exactly
what it does and does not claim.

**Status: v0.1, early.** Only the checks that need no invented empirical threshold have
shipped so far. See [`tasks/todo.md`](tasks/todo.md) for what's deferred and why, and
[`docs/architecture.md`](docs/architecture.md) for the design.

## What this is not

`mikiwame` does not predict thermodynamic stability, formation energy, band structure,
or synthesizability, and it never uses words like "unstable" or "will synthesize"
without having actually computed them (it hasn't — that's out of scope, see
[`docs/scientific_scope.md`](docs/scientific_scope.md)). `Verdict::StructurallyConsistent`
means no structural anomaly was found by the checks that ran — it is not a stability or
synthesizability claim.

## Usage

```rust
use mikiwame::{analyze, AnalysisConfig, OwnedStructure, Site};

let config = AnalysisConfig::default();
let report = analyze(&structure, &config); // structure: impl PeriodicStructureView

println!("{:?}", report.overall.verdict);
for finding in &report.findings {
    println!("{:?} {}: {}", finding.severity, finding.code, finding.explanation);
}
```

Real output, from [`examples/basic.rs`](examples/basic.rs) (`cargo run --example basic`)
comparing a clean rock-salt (NaCl) fixture against a copy with one site moved onto
another:

```text
clean NaCl: StructurallyConsistent
duplicated-site NaCl: StrongAnomalyDetected
  Critical SITE_DUPLICATE: sites 0 and 1 (both Na) coincide under periodic boundary conditions (separation 0.000e0 Å)
```

`analyze` takes anything implementing [`PeriodicStructureView`](src/structure_view.rs) —
your own structure type, or [`OwnedStructure`](src/structure_view.rs) for direct
construction. There is no CIF reader or CLI yet (see below).

## Relationship to `chematic`

`mikiwame` depends only on its own minimal, read-only `PeriodicStructureView` trait —
`chematic`'s current default branch has no periodic/occupancy-aware structure type to
build on. See [`docs/chematic-prerequisites.md`](docs/chematic-prerequisites.md) for
the investigation and what would need to exist in `chematic` for a tighter integration.

## Not yet implemented

CIF/file I/O, the CLI (`analyze`/`batch`/`explain`/`doctor`), coordination/distortion/
composition/disorder diagnostics, and any threshold-based check that would need an
uncited constant (element radii, "extreme" lattice aspect ratio, oxidation-state
tables). See [`tasks/todo.md`](tasks/todo.md).

## Quality gate

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --no-default-features
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
cargo audit
```

## License

Dual-licensed under MIT or Apache-2.0, at your option.
