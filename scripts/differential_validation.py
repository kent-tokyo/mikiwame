#!/usr/bin/env python3
"""Differential validation: mikiwame's *actual* coordination numbers vs. pymatgen's CrystalNN.

AGENTS.md §15.4 asks for differential comparison against pymatgen/spglib where
possible. This script covers the coordination-number slice: it builds the
`mikiwame` CLI, runs `analyze --format json` on the same five structures
`tests/known_good_fixtures.rs` treats as known-good (NaCl, CsCl, diamond, zinc
blende, ideal cubic perovskite -- identical lattice constants and fractional
coordinates), reads the *real* `coordination_number` out of each report's
`local_environment`, and compares it against pymatgen's `CrystalNN` computed
on the identical structure.

This is deliberately end-to-end, not a comparison against a hand-maintained
expected-value table: an earlier version of this script hardcoded mikiwame's
expected coordination numbers and only proved that *those expectations*
agreed with pymatgen, which would go silently stale if the Rust
implementation ever regressed. Running the actual built binary and parsing
its actual output means a real regression in `diagnostics::coordination`
would show up here as a mismatch, not just in `cargo test`.

Not wired into `cargo test` or CI: this is a Python-based, manually-reproduced
check, not part of the Rust quality gate. Exits non-zero on any mismatch, so
it can still be used as a pass/fail gate by hand or in a future CI step.

Setup (isolated virtualenv, does not touch system Python). This script rebuilds
`mikiwame` itself (`cargo build --bin mikiwame`) every run, so the venv only
needs pymatgen:
    python3 -m venv .venv-differential-validation
    .venv-differential-validation/bin/pip install pymatgen
    .venv-differential-validation/bin/python3 scripts/differential_validation.py
"""

import json
import subprocess
import sys
import tempfile
import warnings
from importlib.metadata import version as pkg_version
from pathlib import Path

from pymatgen.core import Lattice, Structure
from pymatgen.analysis.local_env import CrystalNN

REPO_ROOT = Path(__file__).resolve().parent.parent
MIKIWAME_BIN = REPO_ROOT / "target" / "debug" / "mikiwame"


def structure_fixture(name):
    """Returns (lattice_matrix, [(element, fractional), ...]) for one fixture,
    with lattice constants and site positions identical to
    tests/known_good_fixtures.rs."""
    if name == "NaCl":
        a = 5.6402
        sites = [
            ("Na", [0.0, 0.0, 0.0]), ("Na", [0.5, 0.5, 0.0]),
            ("Na", [0.5, 0.0, 0.5]), ("Na", [0.0, 0.5, 0.5]),
            ("Cl", [0.5, 0.5, 0.5]), ("Cl", [0.0, 0.0, 0.5]),
            ("Cl", [0.0, 0.5, 0.0]), ("Cl", [0.5, 0.0, 0.0]),
        ]
    elif name == "CsCl":
        a = 4.123
        sites = [("Cs", [0.0, 0.0, 0.0]), ("Cl", [0.5, 0.5, 0.5])]
    elif name == "diamond":
        a = 3.567
        sites = [
            ("C", f) for f in [
                [0.0, 0.0, 0.0], [0.5, 0.5, 0.0], [0.5, 0.0, 0.5], [0.0, 0.5, 0.5],
                [0.25, 0.25, 0.25], [0.75, 0.75, 0.25], [0.75, 0.25, 0.75], [0.25, 0.75, 0.75],
            ]
        ]
    elif name == "zinc_blende":
        a = 5.41
        zn = [[0.0, 0.0, 0.0], [0.5, 0.5, 0.0], [0.5, 0.0, 0.5], [0.0, 0.5, 0.5]]
        s = [[0.25, 0.25, 0.25], [0.75, 0.75, 0.25], [0.75, 0.25, 0.75], [0.25, 0.75, 0.75]]
        sites = [("Zn", f) for f in zn] + [("S", f) for f in s]
    elif name == "perovskite":
        a = 3.905
        sites = [
            ("Sr", [0.0, 0.0, 0.0]), ("Ti", [0.5, 0.5, 0.5]),
            ("O", [0.5, 0.5, 0.0]), ("O", [0.5, 0.0, 0.5]), ("O", [0.0, 0.5, 0.5]),
        ]
    else:
        raise ValueError(name)
    lattice = [[a, 0.0, 0.0], [0.0, a, 0.0], [0.0, 0.0, a]]
    return lattice, sites


def build_mikiwame():
    """Rebuilds the CLI from the current checkout. Without this, a stale
    target/debug/mikiwame from a previous build could silently pass this
    script even after a real regression in diagnostics::coordination."""
    subprocess.run(["cargo", "build", "--bin", "mikiwame"], cwd=REPO_ROOT, check=True)


def run_mikiwame(lattice, sites):
    """Writes the CLI's JSON structure input, runs `mikiwame analyze --format
    json`, and returns the parsed report."""
    structure_json = {
        "lattice": lattice,
        "sites": [{"element": el, "fractional": f, "occupancy": 1.0} for el, f in sites],
    }
    with tempfile.TemporaryDirectory() as tmpdir:
        input_path = Path(tmpdir) / "structure.json"
        input_path.write_text(json.dumps(structure_json), encoding="utf-8")
        result = subprocess.run(
            [str(MIKIWAME_BIN), "analyze", str(input_path), "--format", "json"],
            cwd=REPO_ROOT, capture_output=True, text=True, check=True,
        )
    return json.loads(result.stdout)


def pymatgen_structure(lattice, sites):
    elements = [el for el, _ in sites]
    coords = [f for _, f in sites]
    return Structure(Lattice(lattice), elements, coords)


def main():
    build_mikiwame()
    if not MIKIWAME_BIN.exists():
        print(f"error: build succeeded but binary is missing: {MIKIWAME_BIN}", file=sys.stderr)
        return 1

    cnn_default = CrystalNN()
    cnn_geometric = CrystalNN(distance_cutoffs=None, x_diff_weight=0, porous_adjustment=False)

    header = f"{'structure':<12} {'site':<3} {'element':<5} {'mikiwame':>9} {'CrystalNN(default)':>19} {'CrystalNN(geometric)':>21}  agree?"
    print(header)
    print("-" * len(header))

    mismatches = 0
    mikiwame_version = None
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")  # expected: no oxidation states set (see docs/validation.md)
        for name in ["NaCl", "CsCl", "diamond", "zinc_blende", "perovskite"]:
            lattice, sites = structure_fixture(name)
            report = run_mikiwame(lattice, sites)
            mikiwame_version = report["provenance"]["mikiwame_version"]
            cn_by_site = {e["site_index"]: e["coordination_number"] for e in report["local_environment"]}
            pmg = pymatgen_structure(lattice, sites)

            for i, (element, _) in enumerate(sites):
                mikiwame_cn = cn_by_site.get(i)
                default_cn = round(cnn_default.get_cn(pmg, i))
                geometric_cn = round(cnn_geometric.get_cn(pmg, i))
                agree = mikiwame_cn == default_cn == geometric_cn
                if not agree:
                    mismatches += 1
                print(
                    f"{name:<12} {i:<3} {element:<5} {mikiwame_cn!s:>9} {default_cn:>19} "
                    f"{geometric_cn:>21}  {'yes' if agree else 'MISMATCH'}"
                )

    print()
    print(f"mikiwame version: {mikiwame_version}")
    print(f"pymatgen version: {pkg_version('pymatgen')}")
    print(f"{mismatches} mismatch(es) out of {sum(len(structure_fixture(n)[1]) for n in ['NaCl', 'CsCl', 'diamond', 'zinc_blende', 'perovskite'])} sites")
    return 1 if mismatches else 0


if __name__ == "__main__":
    sys.exit(main())
