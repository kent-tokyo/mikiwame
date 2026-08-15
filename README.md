# mikiwame (見極め)

[日本語](README_ja.md)

Explainable materials structure diagnostics for periodic crystal structures, in Rust.

`mikiwame` looks at a 3D periodic crystal structure and explains **what** is
structurally unusual about it, **where**, and **on what evidence** — not a single
opaque score. See [`docs/scientific_scope.md`](docs/scientific_scope.md) for exactly
what it does and does not claim.

**Status: pre-1.0, early** (crate version tracks features, not this milestone — see
[`AGENTS.md`](AGENTS.md)'s own "v0.1" scope definition for what's in/out for now). Only
the checks that need no invented empirical threshold have shipped so far. See
[`tasks/todo.md`](tasks/todo.md) for what's deferred and why, and
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
construction. With the optional `cif` feature, [`mikiwame::cif::read_cif`](src/cif.rs)
parses a CIF file into an `OwnedStructure` via `chematic-mol`'s occupancy/disorder-preserving
adapter — see that module's doc comment for what it does and does not diagnose.

## CLI

```bash
cargo run --bin mikiwame -- analyze structure.json --format markdown
cargo run --bin mikiwame -- analyze structure.json --format json   # default
cargo run --bin mikiwame -- batch structures.jsonl --output reports.jsonl
cargo run --bin mikiwame -- explain report.json --finding SITE_DUPLICATE
cargo run --bin mikiwame -- doctor
```

`structure.json` (and each line of `structures.jsonl`) is `{"lattice": [[..],[..],[..]],
"sites": [{"element": "Na", "fractional": [0.0,0.0,0.0], "occupancy": 1.0}, ...]}` — see
[`src/bin/mikiwame.rs`](src/bin/mikiwame.rs)'s module doc comment. This is a CLI-local
schema, independent of the report's `schema_version`. The CLI lives behind the `cli` Cargo
feature (on by default; `cargo build --no-default-features` gives a pure-library build
without pulling in `serde_json`).

`analyze` also accepts a `.cif` path directly (`cargo run --bin mikiwame -- analyze
structure.cif`), when built with the optional `cif` feature: `cargo build --features
cli,cif` (or `cargo install mikiwame --features cif`). `cif` is not on by default — unlike
`cli`, it pulls in `chematic-mol`'s full dependency tree. A CIF that `chematic-mol` cannot
parse or validate (bad occupancy, missing cell parameters, etc.) is a CLI error, not a
diagnosed report; a CIF declaring symmetry beyond P1 is rejected outright (also a CLI
error) rather than analyzed, since `chematic-mol` never expands symmetry operations —
analyzing just the asymmetric unit as if it were the complete cell would misreport
coordination numbers and near-neighbor distances. Export/expand such a CIF to P1 first.
`batch` stays JSONL-only. `doctor` reports whether CIF support is compiled in.

## Relationship to `chematic`

The public input boundary is still mikiwame's own minimal, read-only
`PeriodicStructureView` trait — deliberately, not a stopgap: `chematic-crystal`'s types
validate and reject malformed input at construction, while mikiwame's whole premise is
diagnosing malformed input, not refusing it. Internally, though, `mikiwame` now depends
on [`chematic-crystal`](https://crates.io/crates/chematic-crystal) for periodic-boundary
geometry (exact minimum-image distance, periodic neighbor search). See
[`docs/chematic-prerequisites.md`](docs/chematic-prerequisites.md) for the full
reasoning.

## Not yet implemented

Polyhedral distortion, composition/oxidation-state diagnostics, and any threshold-based
check that would need an uncited constant ("extreme" lattice aspect ratio, oxidation-state
tables).

Disorder's no-threshold subset (`DISORDER_PRESENT`, `DISORDER_OCCUPANCY_SUM_EXCEEDS_ONE`)
and coordination number / local environment (`MaterialDiagnosticReport::local_environment`,
AGENTS.md §7.4 — covalent radii from Cordero et al. 2008 bound the neighbor search, the
actual shell resolved by the largest relative gap in candidate distances; see
[`docs/scientific_scope.md`](docs/scientific_scope.md)) have both shipped.
`SITE_SEVERE_OVERLAP`/`SITE_UNUSUALLY_SHORT_DISTANCE` still need a second, separate
decision beyond the radius table alone; see [`docs/validation.md`](docs/validation.md) for
why. See [`tasks/todo.md`](tasks/todo.md) for the full list.

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
