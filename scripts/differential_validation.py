#!/usr/bin/env python3
"""Differential validation: mikiwame's coordination numbers vs. pymatgen's CrystalNN.

AGENTS.md §15.4 asks for differential comparison against pymatgen/spglib where
possible. This script covers the coordination-number slice only, on the five
structures `tests/known_good_fixtures.rs` already treats as known-good in the
Rust suite (NaCl, CsCl, diamond, zinc blende, ideal cubic perovskite) --
identical lattice constants and fractional coordinates, so this is a genuine
comparison, not a re-derivation from a different source.

Not wired into `cargo test` or CI: this is a Python-based, manually-reproduced
check, not part of the Rust quality gate. See docs/validation.md for the
recorded results and how to interpret a disagreement (not necessarily a bug --
see the perovskite-O note below).

Setup (isolated virtualenv, does not touch system Python):
    python3 -m venv .venv-differential-validation
    .venv-differential-validation/bin/pip install pymatgen
    .venv-differential-validation/bin/python3 scripts/differential_validation.py
"""

from pymatgen.core import Lattice, Structure
from pymatgen.analysis.local_env import CrystalNN

# mikiwame's own coordination numbers, established (with the underlying
# geometry hand-verified before implementation) in
# tests/known_good_fixtures.rs and src/diagnostics/coordination.rs. Keyed by
# (structure_name, site_label) -> coordination number.
MIKIWAME_CN = {
    ("NaCl", "Na"): 6,
    ("NaCl", "Cl"): 6,
    ("CsCl", "Cs"): 8,
    ("CsCl", "Cl"): 8,
    ("diamond", "C"): 4,
    ("zinc_blende", "Zn"): 4,
    ("zinc_blende", "S"): 4,
    ("perovskite", "Sr"): 12,
    ("perovskite", "Ti"): 6,
    ("perovskite", "O"): 2,
}


def nacl():
    a = 5.6402
    lattice = Lattice.cubic(a)
    species = ["Na"] * 4 + ["Cl"] * 4
    coords = [
        [0.0, 0.0, 0.0], [0.5, 0.5, 0.0], [0.5, 0.0, 0.5], [0.0, 0.5, 0.5],
        [0.5, 0.5, 0.5], [0.0, 0.0, 0.5], [0.0, 0.5, 0.0], [0.5, 0.0, 0.0],
    ]
    return Structure(lattice, species, coords)


def cscl():
    a = 4.123
    lattice = Lattice.cubic(a)
    return Structure(lattice, ["Cs", "Cl"], [[0.0, 0.0, 0.0], [0.5, 0.5, 0.5]])


def diamond():
    a = 3.567
    lattice = Lattice.cubic(a)
    coords = [
        [0.0, 0.0, 0.0], [0.5, 0.5, 0.0], [0.5, 0.0, 0.5], [0.0, 0.5, 0.5],
        [0.25, 0.25, 0.25], [0.75, 0.75, 0.25], [0.75, 0.25, 0.75], [0.25, 0.75, 0.75],
    ]
    return Structure(lattice, ["C"] * 8, coords)


def zinc_blende():
    a = 5.41
    lattice = Lattice.cubic(a)
    zn = [[0.0, 0.0, 0.0], [0.5, 0.5, 0.0], [0.5, 0.0, 0.5], [0.0, 0.5, 0.5]]
    s = [[0.25, 0.25, 0.25], [0.75, 0.75, 0.25], [0.75, 0.25, 0.75], [0.25, 0.75, 0.75]]
    return Structure(lattice, ["Zn"] * 4 + ["S"] * 4, zn + s)


def perovskite():
    a = 3.905
    lattice = Lattice.cubic(a)
    species = ["Sr", "Ti", "O", "O", "O"]
    coords = [
        [0.0, 0.0, 0.0], [0.5, 0.5, 0.5],
        [0.5, 0.5, 0.0], [0.5, 0.0, 0.5], [0.0, 0.5, 0.5],
    ]
    return Structure(lattice, species, coords)


STRUCTURES = {
    "NaCl": nacl(),
    "CsCl": cscl(),
    "diamond": diamond(),
    "zinc_blende": zinc_blende(),
    "perovskite": perovskite(),
}

# Default ("chemical bond" style) weighting.
cnn_default = CrystalNN()
# Geometric-only mode, per CrystalNN's own docstring: distance_cutoffs=None,
# x_diff_weight=0, porous_adjustment=False -- closer in spirit to mikiwame's
# purely geometric (no electronegativity weighting) method.
cnn_geometric = CrystalNN(distance_cutoffs=None, x_diff_weight=0, porous_adjustment=False)


def main():
    header = f"{'structure':<12} {'site':<5} {'mikiwame':>9} {'CrystalNN(default)':>19} {'CrystalNN(geometric)':>21}  agree?"
    print(header)
    print("-" * len(header))
    for name, structure in STRUCTURES.items():
        seen_labels = set()
        for i, site in enumerate(structure):
            label = site.species_string
            if label in seen_labels:
                continue  # one representative site per element is enough (fixtures are symmetric)
            seen_labels.add(label)

            mikiwame_cn = MIKIWAME_CN[(name, label)]
            default_cn = round(cnn_default.get_cn(structure, i))
            geometric_cn = round(cnn_geometric.get_cn(structure, i))
            agree = "yes" if (mikiwame_cn == default_cn == geometric_cn) else "DIFFERS"
            print(
                f"{name:<12} {label:<5} {mikiwame_cn:>9} {default_cn:>19} {geometric_cn:>21}  {agree}"
            )


if __name__ == "__main__":
    main()
