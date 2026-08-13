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
construction. There is no CIF reader yet (see below).

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
schema, independent of the report's `schema_version`; there is no CIF reader yet, so this
is the only supported file input. The CLI lives behind the `cli` Cargo feature (on by
default; `cargo build --no-default-features` gives a pure-library build without pulling
in `serde_json`).

## Relationship to `chematic`

`mikiwame` depends only on its own minimal, read-only `PeriodicStructureView` trait —
`chematic`'s current default branch has no periodic/occupancy-aware structure type to
build on. See [`docs/chematic-prerequisites.md`](docs/chematic-prerequisites.md) for
the investigation and what would need to exist in `chematic` for a tighter integration.

## Not yet implemented

CIF/file I/O, coordination/distortion/composition diagnostics (disorder's no-threshold
subset — `DISORDER_PRESENT`, `DISORDER_OCCUPANCY_SUM_EXCEEDS_ONE` — has shipped), and any
threshold-based check that would need an uncited constant ("extreme" lattice aspect ratio,
oxidation-state tables). Covalent radii (Cordero et al. 2008) are sourced and embedded but
not yet wired into a check — `SITE_SEVERE_OVERLAP`/`SITE_UNUSUALLY_SHORT_DISTANCE` need a
second, separate decision first; see [`docs/validation.md`](docs/validation.md) for why.
See [`tasks/todo.md`](tasks/todo.md).

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
