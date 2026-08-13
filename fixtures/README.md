# Fixtures

No external data files ship in v0.1 (CIF reading is not implemented yet — see
`tasks/todo.md`). The one fixture used by `tests/diagnostics.rs` is built directly in
Rust: the conventional cubic unit cell of rock salt (NaCl), space group Fm-3m (No.
225), 4 formula units, lattice parameter a = 5.6402 Å.

* **Source**: standard crystallographic knowledge (e.g. R. W. G. Wyckoff, *Crystal
  Structures*, Vol. 1); the lattice parameter matches the commonly cited room-temperature
  value for NaCl. Not sourced from a proprietary database.
* **License**: the structure itself is public-domain crystallographic fact; no
  copyrighted dataset is embedded.

All "broken" variants used in tests are derived programmatically from this fixture
(mutating one field at a time), not stored as separate files.
