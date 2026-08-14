//! Covalent radius reference data — embedded per owner approval (elemental
//! radius table source decided: Cordero et al. 2008), but **not yet consumed
//! by any diagnostic**. See `docs/validation.md` for why: a naive
//! `observed_distance < covalent_radius_sum` comparison produces a false
//! positive on the already-shipped ideal-perovskite fixture (Ti–O in SrTiO3
//! is a textbook-normal 1.9525 Å bond, below the covalent-radii sum of
//! 2.26 Å) — covalent radii predict bonded distance well for covalent
//! networks (diamond, zinc blende) but not for ionic bonding, and most
//! inorganic crystals mikiwame targets are at least partly ionic. Doing this
//! correctly needs oxidation-state-aware ionic radii, not this table alone.
//!
//! Source: Cordero, B.; Gómez, V.; Platero-Prats, A. E.; Revés, M.;
//! Echeverría, J.; Cremades, E.; Barragán, F.; Alvarez, S. "Covalent radii
//! revisited." *Dalton Trans.* **2008**, 2832–2838. DOI: 10.1039/B801115J,
//! Table 2.
//!
//! Values were cross-checked against MolSSI QCElemental's
//! `alvarez_2008_covalent_radii` (github.com/MolSSI/QCElemental,
//! `qcelemental/data/alvarez_2008_covalent_radii.py`, itself transcribed
//! from the same Table 2) rather than hand-transcribed from the paper alone,
//! to catch transcription error.
//!
//! Coverage: Z = 1 (H) through 96 (Cm) — the paper reports nothing beyond Cm.
//! [`covalent_radius_angstrom`] returns `None` outside this coverage; callers
//! must not substitute a default value.
//!
//! Disambiguation: the source table gives more than one value for a few
//! elements (hybridization- or spin-state-dependent), but [`crate::Site`]
//! carries neither. One value was picked per element rather than left an
//! implicit default:
//! * carbon: the sp3 value (0.76 Å) — the most commonly used default
//!   single-bond radius for unspecified hybridization.
//! * Mn, Fe, Co: the low-spin value — matches the single value commonly
//!   quoted for these elements where a source doesn't distinguish spin state
//!   (e.g. Fe 1.32 Å).

/// Identifies which table and disambiguation rules `covalent_radius_angstrom`
/// uses. Recorded in `Provenance::radius_table_version` by
/// `diagnostics::coordination`, the first (and so far only) consumer.
pub(crate) const RADIUS_TABLE_VERSION: &str = "cordero-2008-table2";

#[rustfmt::skip]
const TABLE: &[(&str, f64)] = &[
    ("H", 0.31), ("He", 0.28), ("Li", 1.28), ("Be", 0.96), ("B", 0.84),
    ("C", 0.76), ("N", 0.71), ("O", 0.66), ("F", 0.57), ("Ne", 0.58),
    ("Na", 1.66), ("Mg", 1.41), ("Al", 1.21), ("Si", 1.11), ("P", 1.07),
    ("S", 1.05), ("Cl", 1.02), ("Ar", 1.06), ("K", 2.03), ("Ca", 1.76),
    ("Sc", 1.70), ("Ti", 1.60), ("V", 1.53), ("Cr", 1.39), ("Mn", 1.39),
    ("Fe", 1.32), ("Co", 1.26), ("Ni", 1.24), ("Cu", 1.32), ("Zn", 1.22),
    ("Ga", 1.22), ("Ge", 1.20), ("As", 1.19), ("Se", 1.20), ("Br", 1.20),
    ("Kr", 1.16), ("Rb", 2.20), ("Sr", 1.95), ("Y", 1.90), ("Zr", 1.75),
    ("Nb", 1.64), ("Mo", 1.54), ("Tc", 1.47), ("Ru", 1.46), ("Rh", 1.42),
    ("Pd", 1.39), ("Ag", 1.45), ("Cd", 1.44), ("In", 1.42), ("Sn", 1.39),
    ("Sb", 1.39), ("Te", 1.38), ("I", 1.39), ("Xe", 1.40), ("Cs", 2.44),
    ("Ba", 2.15), ("La", 2.07), ("Ce", 2.04), ("Pr", 2.03), ("Nd", 2.01),
    ("Pm", 1.99), ("Sm", 1.98), ("Eu", 1.98), ("Gd", 1.96), ("Tb", 1.94),
    ("Dy", 1.92), ("Ho", 1.92), ("Er", 1.89), ("Tm", 1.90), ("Yb", 1.87),
    ("Lu", 1.87), ("Hf", 1.75), ("Ta", 1.70), ("W", 1.62), ("Re", 1.51),
    ("Os", 1.44), ("Ir", 1.41), ("Pt", 1.36), ("Au", 1.36), ("Hg", 1.32),
    ("Tl", 1.45), ("Pb", 1.46), ("Bi", 1.48), ("Po", 1.40), ("At", 1.50),
    ("Rn", 1.50), ("Fr", 2.60), ("Ra", 2.21), ("Ac", 2.15), ("Th", 2.06),
    ("Pa", 2.00), ("U", 1.96), ("Np", 1.90), ("Pu", 1.87), ("Am", 1.80),
    ("Cm", 1.69),
];

/// Returns the Cordero et al. 2008 covalent radius (Å) for `symbol`, or
/// `None` if `symbol` isn't a recognized element or falls outside the
/// paper's coverage (Z > 96). Never substitutes a default — see the module
/// doc comment.
pub(crate) fn covalent_radius_angstrom(symbol: &str) -> Option<f64> {
    TABLE
        .iter()
        .find(|(candidate, _)| *candidate == symbol)
        .map(|(_, radius)| *radius)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spot_check_known_values() {
        // One from each end of the table plus a disambiguated element,
        // checked against the source transcription — catches transcription
        // drift without re-asserting all 96 entries.
        assert_eq!(covalent_radius_angstrom("H"), Some(0.31));
        assert_eq!(covalent_radius_angstrom("O"), Some(0.66));
        assert_eq!(covalent_radius_angstrom("Ti"), Some(1.60));
        assert_eq!(covalent_radius_angstrom("Cs"), Some(2.44));
        assert_eq!(covalent_radius_angstrom("Cm"), Some(1.69));
    }

    #[test]
    fn unrecognized_or_out_of_coverage_symbols_are_none_not_a_default() {
        assert_eq!(covalent_radius_angstrom("Xx"), None);
        // Bk (Z=97) is a real element but beyond the paper's coverage (Cm,
        // Z=96 is the last entry); must not silently fall back to a value.
        assert_eq!(covalent_radius_angstrom("Bk"), None);
    }

    #[test]
    fn table_has_exactly_one_entry_per_element_z1_through_z96() {
        assert_eq!(TABLE.len(), 96);
        let mut seen = std::collections::HashSet::new();
        for (symbol, radius) in TABLE {
            assert!(seen.insert(*symbol), "duplicate entry for {symbol}");
            assert!(
                radius.is_finite() && *radius > 0.0,
                "bad radius for {symbol}"
            );
        }
    }

    /// Load-bearing negative result — see the module doc comment and
    /// `docs/validation.md`. Ti–O in the ideal cubic perovskite fixture
    /// (`tests/known_good_fixtures.rs::perovskite_is_structurally_consistent`,
    /// a = 3.905 Å, Ti at (0.5,0.5,0.5), O at (0.5,0.5,0.0)) is a textbook,
    /// already-shipped known-good bond at a/2 = 1.9525 Å — well *inside* the
    /// sum of Cordero covalent radii for Ti and O (1.60 + 0.66 = 2.26 Å).
    /// A diagnostic that flagged `observed < covalent_radius_sum` as
    /// "unusually short" would false-positive on this structure. Covalent
    /// radii predict bonded distance well for covalent networks (see the
    /// diamond/zinc-blende fixtures, both comfortably *above* their radius
    /// sum) but this table alone cannot safely gate ionic bonding, which is
    /// most of what mikiwame's target inorganic crystals are made of.
    #[test]
    fn expected_distance_from_covalent_radii_is_unsafe_for_ionic_bonds() {
        let a = 3.905_f64;
        let ti_o_observed = a / 2.0;
        let ti_o_covalent_radius_sum =
            covalent_radius_angstrom("Ti").unwrap() + covalent_radius_angstrom("O").unwrap();
        assert!(
            ti_o_observed < ti_o_covalent_radius_sum,
            "if this ever stops holding, the perovskite Ti-O distance or the Ti/O radii \
             changed — re-check whether SITE_UNUSUALLY_SHORT_DISTANCE can be built on this \
             table alone before removing this test"
        );
    }
}
