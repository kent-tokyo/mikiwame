//! CIF (Crystallographic Information File) input, via `chematic-mol`'s
//! occupancy-aware `crystal` feature adapter — mikiwame does not reimplement
//! CIF parsing itself (AGENTS.md's isolate-CIF-I/O-in-an-adapter-layer
//! principle).
//!
//! This is a convenience input path for *valid* CIFs only: `chematic-mol`
//! validates and rejects malformed input (bad occupancy, occupancy sum over
//! `1.0`, missing cell parameters, etc.) before mikiwame ever constructs a
//! structure from it, the same reject-at-construction model
//! `docs/chematic-prerequisites.md` documents for `chematic_crystal` more
//! generally. Concretely, this means three finding codes are structurally
//! unreachable for any successfully-parsed CIF: `INPUT_UNKNOWN_ELEMENT`
//! (an unresolvable element symbol fails CIF parsing itself),
//! `INPUT_INVALID_OCCUPANCY`, and `DISORDER_OCCUPANCY_SUM_EXCEEDS_ONE`
//! (an occupancy sum over tolerance fails `PeriodicSite` construction).
//! Callers who need mikiwame's own diagnosis of a malformed structure use
//! the JSON/[`crate::structure_view::OwnedStructure`] input path instead,
//! where those checks run.
//!
//! `PeriodicSite::label` (e.g. `"Na1"`) has no home in
//! [`crate::structure_view::Site`] and is dropped — not a CIF-specific
//! regression, since the JSON input path never had per-site labels either.

use chematic_crystal::PeriodicStructure;

pub use chematic_mol::cif::{CifPeriodicError, CifSymmetryStatus};

use crate::structure_view::{OwnedStructure, Site};

/// The result of reading a CIF file: the parsed structure, plus whether the
/// file declared symmetry this adapter did not expand.
///
/// `chematic-mol`'s CIF reader never expands symmetry operations — the
/// returned sites are the complete unit cell only when
/// [`CifStructure::symmetry`] is [`CifSymmetryStatus::P1`]; otherwise they
/// are only the asymmetric unit as literally listed in the file. Surfaced
/// here rather than hidden so a caller can tell the difference.
#[derive(Debug)]
pub struct CifStructure {
    /// The parsed structure, converted into mikiwame's own structure
    /// boundary.
    pub structure: OwnedStructure,
    /// Whether the CIF declared symmetry beyond P1 that was not expanded.
    pub symmetry: CifSymmetryStatus,
}

/// Reads a CIF file's periodic structure.
///
/// A CIF `chematic-mol` cannot parse or validate is rejected outright
/// (`Err`), not converted into a diagnosable `InvalidInput` report — see
/// this module's doc comment for why, and for which mikiwame finding codes
/// this input path cannot produce as a result.
///
/// Multi-species sites (CIF's convention for positional/substitutional
/// disorder) flatten into multiple [`Site`]s at the same fractional
/// position with different elements/occupancies, matching mikiwame's
/// existing disorder representation (see [`Site`]'s doc comment) — no
/// separate handling is needed downstream for CIF-sourced disorder.
pub fn read_cif(input: &str) -> Result<CifStructure, CifPeriodicError> {
    let result = chematic_mol::cif::parse_cif_periodic_structure(input)?;
    Ok(CifStructure {
        structure: to_owned_structure(&result.structure),
        symmetry: result.symmetry,
    })
}

fn to_owned_structure(structure: &PeriodicStructure) -> OwnedStructure {
    let lattice = structure.lattice().matrix();
    let sites = structure
        .sites()
        .iter()
        .flat_map(|site| {
            let fractional = site.fractional.0;
            site.species.iter().map(move |species| Site {
                element: species.element.symbol().to_string(),
                fractional,
                occupancy: species.occupancy.value(),
            })
        })
        .collect();
    OwnedStructure::new(lattice, sites)
}
