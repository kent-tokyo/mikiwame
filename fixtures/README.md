# Fixtures

No external data files ship in v0.1 (CIF reading is not implemented yet — see
`tasks/todo.md`). All fixtures are built directly in Rust, in `tests/diagnostics.rs`,
`tests/metamorphic.rs`, and `tests/known_good_fixtures.rs`.

## Known-good structures

Standard crystallographic knowledge (e.g. R. W. G. Wyckoff, *Crystal Structures*, Vol.
1); lattice parameters match commonly cited room-temperature values. Not sourced from a
proprietary database — the structures themselves are public-domain crystallographic
fact, no copyrighted dataset is embedded.

| Structure | Space group | a (Å) | Used in |
|---|---|---|---|
| Rock salt (NaCl) | Fm-3m (225) | 5.6402 | `tests/diagnostics.rs`, `tests/metamorphic.rs` |
| CsCl | Pm-3m (221) | 4.123 | `tests/known_good_fixtures.rs` |
| Diamond cubic | Fd-3m (227) | 3.567 | `tests/known_good_fixtures.rs` |
| Zinc blende (ZnS) | F-43m (216) | 5.41 | `tests/known_good_fixtures.rs` |
| Ideal perovskite (SrTiO3) | Pm-3m (221) | 3.905 | `tests/known_good_fixtures.rs` |

Deliberately not included: wurtzite, rutile, and spinel (also candidates in AGENTS.md
§15.1) each have at least one free internal positional parameter (wurtzite's `u`,
rutile's `x`, spinel's oxygen `u`) that would need a specific cited source to reproduce
accurately, unlike the structures above where every site sits at a fixed
high-symmetry Wyckoff position with no free parameter to get wrong from memory. Graphite
is deferred alongside them for consistency, not because it has a free parameter. See
`docs/scientific_scope.md`'s threshold-discipline section for the same reasoning applied
to diagnostic thresholds.

All "broken"/transformed variants used in tests (duplicated sites, invalid occupancy,
rotated lattices, supercells, ...) are derived programmatically from these fixtures
(mutating or transforming one field at a time), not stored as separate files.
