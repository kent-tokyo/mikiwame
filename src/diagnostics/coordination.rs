//! Coordination number / local environment checks (AGENTS.md §7.4) — the
//! geometry-only baseline. Not implemented: oxidation-state-dependent
//! "expected" coordination number (`COORDINATION_UNDERCOORDINATED`/
//! `OVERCOORDINATED`, needs Phase 4's composition work) and polyhedral
//! distortion (AGENTS.md §7.5, needs its own cited ideal-polyhedron
//! reference set — see `tasks/todo.md`).
//!
//! ## Method
//!
//! For each site with a resolvable element and no positional disorder (see
//! "What gets skipped" below):
//!
//! 1. Find every candidate neighbor within a generous, structure-wide search
//!    radius via `chematic_crystal::PeriodicStructure::neighbors_within`
//!    (exact, reciprocal-lattice-bounded search).
//! 2. Keep only candidates whose *specific* pairwise distance is within
//!    `r_center + r_neighbor + epsilon` — covalent radii from Cordero et al.
//!    2008 (`crate::radii`), `epsilon` = [`BOND_TOLERANCE_ANGSTROM`] (0.4 Å;
//!    Šidlauskaitė et al. 2026, arXiv:2601.02017, and PackFlow 2025
//!    independently converge on this value for the same kind of
//!    radius-sum-plus-tolerance bond heuristic).
//! 3. Sort surviving candidates by distance and find the *largest relative
//!    gap* between consecutive distances. Everything up to and including the
//!    gap is the resolved coordination shell; its size is the coordination
//!    number.
//!
//! Step 2 alone is not sufficient, and this is a documented, tested finding
//! from this round's design work, not an assumption: a pure radius-sum+
//! epsilon cutoff applied uniformly to every candidate species over-counts
//! whenever a same-scale cation happens to fall within its own generous
//! self-cutoff. Verified by hand against this crate's own fixtures before
//! implementation: CsCl's Cs gets 8 Cl + 6 Cs = 14 "neighbors" without step
//! 3, not the textbook 8 (Cs–Cs, at 4.123 Å, sits inside Cs+Cs+epsilon =
//! 5.28 Å); perovskite's Ti gets 6 O + 8 Sr = 14, not 6 (Ti–Sr, at 3.382 Å,
//! sits inside Ti+Sr+epsilon = 3.95 Å); perovskite's Sr is the sharpest
//! case, with *three* candidate shells (12 O at 2.761 Å, 8 Ti at 3.382 Å, 6
//! Sr at 3.905 Å) all surviving step 2 — step 3 must pick the largest of the
//! *two* gaps between them (O→Ti ratio ≈1.225 vs. Ti→Sr ratio ≈1.155) to
//! land on the textbook 12, not merely the first gap it sees. Step 3 is what
//! actually determines the shell boundary; step 2 only bounds the search.
//! See `tests/known_good_fixtures.rs` for the regression test proving step 3
//! is load-bearing (a version of the check using only step 2 is shown to
//! give the wrong answer on exactly these fixtures).
//!
//! "Largest relative gap" is a documented, not proven-optimal, choice: for a
//! structure with more than two candidate shells where the *first* gap is
//! smaller than a *later* one, this method would report too many neighbors.
//! No fixture currently in this repo exercises that case. Left as a known
//! limitation rather than guarded against with an unvalidated extra rule
//! (see `tasks/todo.md`).
//!
//! ## What gets skipped, and why (never silently defaulted)
//!
//! - The whole component, if `chematic_crystal::Lattice::from_matrix` or
//!   `PeriodicStructure::neighbors_within` rejects the structure (near-
//!   singular/too-short-axis lattice, or a neighbor search implying an
//!   absurd number of candidate images) — `input_quality`'s own
//!   `LATTICE_SINGULAR` check is fatal only for non-positive volume, so a
//!   structure can reach here with a lattice this crate still can't build a
//!   geometry object from. No naive-fallback neighbor search exists to fall
//!   back to (unlike `structure_view::minimum_image`, there was never a
//!   from-scratch one in mikiwame here), so the component reports
//!   `ComponentStatus::Skipped` rather than inventing one.
//! - A site whose element symbol doesn't resolve to a `chematic_core::Element`,
//!   or whose element falls outside Cordero et al.'s Z=1–96 coverage: no
//!   citable radius, so no citable cutoff for it as a center.
//! - A site that positionally coincides with a different-element site
//!   (mikiwame's disorder representation): coordination number for a
//!   disordered position needs a modeling decision (per-species vs.
//!   combined) this round doesn't make. Also excluded from being counted as
//!   *another* site's neighbor, for the same reason — a real neighbor is
//!   still there, but the reported coordination number then undercounts it;
//!   noted via `SiteLocalEnvironment::limitations`, not hidden.
//!
//! Each skip is recorded on that site's `SiteLocalEnvironment` entry via
//! `not_computed_reason`, never silently substituted.

use std::collections::HashMap;

use chematic_core::Element;
use chematic_crystal::{FractionalCoord, Occupancy, PeriodicSite, PeriodicStructure, SiteSpecies};

use crate::finding::Finding;
use crate::radii::{self, covalent_radius_angstrom};
use crate::report::{NeighborSpeciesCount, SiteLocalEnvironment};
use crate::structure_view::{PeriodicStructureView, coincidence_groups};

/// Sensitivity constant added to a pair's summed covalent radii to decide
/// whether a candidate distance is plausibly a bonded neighbor at all (step
/// 2 in the module doc comment) — the search's outer bound, not the shell
/// boundary. Šidlauskaitė et al. 2026 (arXiv:2601.02017) and PackFlow (2025)
/// each independently use 0.4 Å; both describe a commonly-used range of
/// 0.35–0.45 Å for this constant.
const BOND_TOLERANCE_ANGSTROM: f64 = 0.4;

// Numerical-identity tolerance for grouping positionally-coincident sites
// before building the geometry object — same value and justification as
// separation::DUPLICATE_TOLERANCE_ANGSTROM / disorder::COINCIDENCE_TOLERANCE_ANGSTROM,
// kept independent per this project's existing convention.
const COINCIDENCE_TOLERANCE_ANGSTROM: f64 = 1e-6;

/// Result of running coordination checks.
pub(crate) struct Outcome {
    pub(crate) findings: Vec<Finding>,
    pub(crate) local_environment: Vec<SiteLocalEnvironment>,
    /// `Some(reason)` if the whole component was skipped (geometry object
    /// construction or the neighbor search itself failed); see `lib.rs`.
    pub(crate) skipped: Option<String>,
}

/// Description recorded in `Provenance::coordination_method` whenever this
/// component ran at all (`Outcome::skipped.is_none()`).
pub(crate) fn method_description() -> String {
    format!(
        "covalent-radius-sum-plus-tolerance(epsilon={BOND_TOLERANCE_ANGSTROM}A, largest-relative-gap shell detection, {})",
        radii::RADIUS_TABLE_VERSION
    )
}

/// One coincidence group (positionally-merged site or sites), resolved to
/// its `chematic_crystal` species list if every member's element is known
/// and has a citable radius.
struct Group {
    /// Original site indices in this group.
    site_indices: Vec<usize>,
    /// `Some` only for a single-species group whose one element has a
    /// Cordero radius — the only case this round computes a coordination
    /// number for, either as a center or as a countable neighbor.
    resolved: Option<(Element, f64)>,
}

pub(crate) fn check<S: PeriodicStructureView>(structure: &S) -> Outcome {
    let sites = structure.sites();

    let crystal_lattice = match chematic_crystal::Lattice::from_matrix(*structure.lattice()) {
        Ok(lattice) => lattice,
        Err(err) => {
            return Outcome {
                findings: Vec::new(),
                local_environment: Vec::new(),
                skipped: Some(format!(
                    "lattice rejected by chematic_crystal::Lattice::from_matrix: {err}"
                )),
            };
        }
    };

    // Group sites by exact PBC coincidence so a disordered (multi-species)
    // position becomes one chematic_crystal::PeriodicSite, matching that
    // type's native multi-species design -- reuses the same grouping
    // diagnostics::disorder uses, not a second implementation.
    let (raw_groups, _fallback_limitation) =
        coincidence_groups(structure, COINCIDENCE_TOLERANCE_ANGSTROM);

    let mut site_to_group: HashMap<usize, usize> = HashMap::new();
    let mut groups: Vec<Group> = Vec::with_capacity(raw_groups.len());
    for (group_index, site_indices) in raw_groups.into_iter().enumerate() {
        for &i in &site_indices {
            site_to_group.insert(i, group_index);
        }
        let resolved = match site_indices.as_slice() {
            [only] => Element::from_symbol(&sites[*only].element)
                .zip(covalent_radius_angstrom(&sites[*only].element)),
            _ => None,
        };
        groups.push(Group {
            site_indices,
            resolved,
        });
    }

    let crystal_sites: Vec<PeriodicSite> = groups
        .iter()
        .map(|group| build_periodic_site(sites, group))
        .collect();
    let crystal_structure = PeriodicStructure::new(crystal_lattice, crystal_sites)
        .expect("each PeriodicSite was already validated individually above");

    let max_present_radius = groups
        .iter()
        .filter_map(|g| g.resolved.map(|(_, r)| r))
        .fold(0.0_f64, f64::max);
    let search_radius = 2.0 * max_present_radius + BOND_TOLERANCE_ANGSTROM;

    let neighbors = match crystal_structure.neighbors_within(search_radius) {
        Ok(neighbors) => neighbors,
        Err(err) => {
            return Outcome {
                findings: Vec::new(),
                local_environment: Vec::new(),
                skipped: Some(format!("neighbor search failed: {err}")),
            };
        }
    };

    let mut neighbors_by_group: HashMap<usize, Vec<&chematic_crystal::PeriodicNeighbor>> =
        HashMap::new();
    for n in &neighbors {
        neighbors_by_group
            .entry(n.center_index)
            .or_default()
            .push(n);
    }

    // No FindingCode ships from this component yet -- see the "ambiguous"
    // note further down. Kept as a real (if always-empty) Vec, not removed,
    // since Outcome/check's shape is what the near-future ambiguity finding
    // will populate.
    let findings: Vec<Finding> = Vec::new();
    let mut local_environment = Vec::with_capacity(sites.len());

    for (site_index, _site) in sites.iter().enumerate() {
        let group_index = site_to_group[&site_index];
        let group = &groups[group_index];

        if group.site_indices.len() > 1 {
            local_environment.push(SiteLocalEnvironment {
                site_index,
                coordination_number: None,
                neighbor_species: Vec::new(),
                shell_gap_ratio: None,
                not_computed_reason: Some(
                    "site belongs to a disordered (multi-species) position; combined \
                     coordination number for disorder is not computed in v0.1"
                        .to_string(),
                ),
                limitations: Vec::new(),
            });
            continue;
        }

        let Some((_center_element, center_radius)) = group.resolved else {
            local_environment.push(SiteLocalEnvironment {
                site_index,
                coordination_number: None,
                neighbor_species: Vec::new(),
                shell_gap_ratio: None,
                not_computed_reason: Some(format!(
                    "no covalent radius available for element \"{}\" (unrecognized, or outside \
                     Cordero et al. 2008's Z=1-96 coverage)",
                    sites[site_index].element
                )),
                limitations: Vec::new(),
            });
            continue;
        };

        let candidates = neighbors_by_group.get(&group_index).into_iter().flatten();
        let mut excluded_unresolvable = 0usize;
        let mut included: Vec<(f64, Element)> = Vec::new();
        for n in candidates {
            let neighbor_group = &groups[n.neighbor_index];
            let Some((neighbor_element, neighbor_radius)) = neighbor_group.resolved else {
                excluded_unresolvable += 1;
                continue;
            };
            let pairwise_cutoff = center_radius + neighbor_radius + BOND_TOLERANCE_ANGSTROM;
            if n.distance <= pairwise_cutoff {
                included.push((n.distance, neighbor_element));
            }
        }
        included.sort_by(|a, b| a.0.total_cmp(&b.0));

        let (shell, gap_ratio) = resolve_shell(&included);

        let mut neighbor_species: HashMap<&'static str, usize> = HashMap::new();
        for (_, element) in shell {
            *neighbor_species.entry(element.symbol()).or_insert(0) += 1;
        }
        let mut neighbor_species: Vec<NeighborSpeciesCount> = neighbor_species
            .into_iter()
            .map(|(element, count)| NeighborSpeciesCount {
                element: element.to_string(),
                count,
            })
            .collect();
        neighbor_species.sort_by(|a, b| a.element.cmp(&b.element));

        let mut limitations = Vec::new();
        if excluded_unresolvable > 0 {
            limitations.push(format!(
                "{excluded_unresolvable} candidate neighbor(s) excluded: disordered position or \
                 unresolvable element"
            ));
        }

        let coordination_number = shell.len();
        // A COORDINATION_AMBIGUOUS finding is deliberately not fired in
        // v0.1: `shell_gap_ratio` (below) is already the honest ambiguity
        // signal (larger = more clearly separated; `None` only means "one
        // clean, unambiguous shell with nothing else nearby" -- see
        // `resolve_shell`'s doc comment, and the bug that finding fixed).
        // Turning a *small-but->1.0* ratio into a binary "ambiguous" verdict
        // needs its own cutoff, which is exactly the kind of threshold
        // AGENTS.md §21 says not to invent without a citable basis. Left as
        // a follow-up (see `tasks/todo.md`) rather than shipped with a
        // guessed number.

        local_environment.push(SiteLocalEnvironment {
            site_index,
            coordination_number: Some(coordination_number),
            neighbor_species,
            shell_gap_ratio: gap_ratio,
            not_computed_reason: None,
            limitations,
        });
    }

    Outcome {
        findings,
        local_environment,
        skipped: None,
    }
}

/// Builds one `chematic_crystal::PeriodicSite` for a coincidence group.
/// Occupancy is split evenly across the group's members: fidelity to the
/// raw input isn't needed here (only position/species identity are read by
/// the neighbor search), and splitting evenly guarantees
/// `PeriodicSite`'s occupancy-sum validation always passes regardless of the
/// raw input's own (possibly invalid) occupancy values, which mikiwame's own
/// `INPUT_INVALID_OCCUPANCY`/`DISORDER_OCCUPANCY_SUM_EXCEEDS_ONE` findings
/// already report independently from the raw input.
fn build_periodic_site(sites: &[crate::structure_view::Site], group: &Group) -> PeriodicSite {
    let representative_fractional = sites[group.site_indices[0]].fractional;
    let species_count = group.site_indices.len() as f64;
    let species: Vec<SiteSpecies> = group
        .site_indices
        .iter()
        .map(|&i| {
            let element = Element::from_symbol(&sites[i].element).unwrap_or(Element::H);
            SiteSpecies {
                element,
                occupancy: Occupancy::new(1.0 / species_count)
                    .expect("1.0/N for N>=1 is always finite, positive, and <= 1.0"),
            }
        })
        .collect();
    PeriodicSite::new(
        species,
        FractionalCoord::new(representative_fractional),
        None,
    )
    .expect(
        "finite fractional (guaranteed by fatal input_quality checks already having \
                 run), non-empty species, and an occupancy sum of exactly 1.0 are all satisfied",
    )
}

/// Finds the largest relative gap in a sorted-by-distance candidate list,
/// returning the candidates before and including that gap (the resolved
/// shell) and the gap ratio itself.
///
/// Returns `None` for the ratio (and the *whole* list as the shell) both
/// when there are fewer than 2 candidates, and — importantly — when every
/// consecutive pair is exactly tied (e.g. a symmetric site with several
/// equidistant neighbors and nothing else within the search radius, like
/// every octahedral neighbor in rock salt): a ratio of exactly `1.0` is not
/// a gap, it is the same shell repeating, so it must never be picked as the
/// boundary. Only a ratio *strictly greater than* `1.0` counts as an
/// observed gap.
fn resolve_shell(sorted: &[(f64, Element)]) -> (&[(f64, Element)], Option<f64>) {
    if sorted.len() < 2 {
        return (sorted, None);
    }
    let mut best_index: Option<usize> = None;
    let mut best_ratio = 1.0_f64;
    for i in 0..sorted.len() - 1 {
        // Both distances are positive: a pairwise cutoff of 0 can't survive
        // (BOND_TOLERANCE_ANGSTROM alone is already > 0), so this never
        // divides by zero.
        let ratio = sorted[i + 1].0 / sorted[i].0;
        if ratio > best_ratio {
            best_ratio = ratio;
            best_index = Some(i);
        }
    }
    match best_index {
        Some(i) => (&sorted[..=i], Some(best_ratio)),
        None => (sorted, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Proves step 3 (the gap step) is load-bearing, not decorative: the
    /// exact CsCl candidate distances that survive step 2 alone (8 Cl at
    /// 3.571 A, 6 Cs at 4.123 A -- both within Cs+Cs+epsilon = 5.28 A of a
    /// Cs center, see the module doc comment) total 14 candidates before
    /// the gap step, but `resolve_shell` must narrow that to the textbook
    /// 8. `tests/known_good_fixtures.rs::cscl_is_structurally_consistent`
    /// is the end-to-end version of this same check through `analyze`.
    #[test]
    fn resolve_shell_narrows_cscl_style_naive_fourteen_to_eight() {
        let cl = Element::from_symbol("Cl").expect("Cl is a known element");
        let cs = Element::from_symbol("Cs").expect("Cs is a known element");
        let mut candidates: Vec<(f64, Element)> = Vec::new();
        candidates.extend(std::iter::repeat_n((3.571, cl), 8));
        candidates.extend(std::iter::repeat_n((4.123, cs), 6));
        assert_eq!(candidates.len(), 14, "naive pairwise-cutoff-only count");

        let (shell, gap_ratio) = resolve_shell(&candidates);
        assert_eq!(shell.len(), 8);
        assert!(shell.iter().all(|(_, element)| *element == cl));
        assert!(gap_ratio.is_some_and(|r| (r - 4.123 / 3.571).abs() < 1e-9));
    }

    /// Same proof for perovskite's Ti: 6 O at 1.9525 A and 8 Sr at
    /// 3.382 A both survive step 2 (Ti+Sr+epsilon = 3.95 A), totaling 14,
    /// but the shell must resolve to the textbook 6.
    #[test]
    fn resolve_shell_narrows_perovskite_ti_style_naive_fourteen_to_six() {
        let o = Element::from_symbol("O").expect("O is a known element");
        let sr = Element::from_symbol("Sr").expect("Sr is a known element");
        let mut candidates: Vec<(f64, Element)> = Vec::new();
        candidates.extend(std::iter::repeat_n((1.9525, o), 6));
        candidates.extend(std::iter::repeat_n((3.382, sr), 8));
        assert_eq!(candidates.len(), 14, "naive pairwise-cutoff-only count");

        let (shell, _) = resolve_shell(&candidates);
        assert_eq!(shell.len(), 6);
        assert!(shell.iter().all(|(_, element)| *element == o));
    }

    /// A tied cluster with nothing else nearby (rock salt's 6 equidistant
    /// Cl around Na) must resolve to the whole cluster, not just its first
    /// entry -- the bug `resolve_shell`'s doc comment describes, found and
    /// fixed while verifying this module against the known-good fixtures
    /// before trusting it.
    #[test]
    fn resolve_shell_keeps_a_fully_tied_cluster_intact() {
        let cl = Element::from_symbol("Cl").expect("Cl is a known element");
        let candidates: Vec<(f64, Element)> = std::iter::repeat_n((2.8201, cl), 6).collect();

        let (shell, gap_ratio) = resolve_shell(&candidates);
        assert_eq!(shell.len(), 6);
        assert_eq!(gap_ratio, None);
    }

    /// Three-shell case (perovskite's Sr): the largest gap is between the
    /// first and second shells, not the second and third, and
    /// `resolve_shell` must pick that one, not merely the first gap it
    /// encounters.
    #[test]
    fn resolve_shell_picks_the_largest_of_two_gaps_not_the_first() {
        let o = Element::from_symbol("O").expect("O is a known element");
        let ti = Element::from_symbol("Ti").expect("Ti is a known element");
        let sr = Element::from_symbol("Sr").expect("Sr is a known element");
        let mut candidates: Vec<(f64, Element)> = Vec::new();
        candidates.extend(std::iter::repeat_n((2.761, o), 12));
        candidates.extend(std::iter::repeat_n((3.382, ti), 8));
        candidates.extend(std::iter::repeat_n((3.905, sr), 6));

        let (shell, gap_ratio) = resolve_shell(&candidates);
        assert_eq!(shell.len(), 12);
        assert!(shell.iter().all(|(_, element)| *element == o));
        assert!(gap_ratio.is_some_and(|r| (r - 3.382 / 2.761).abs() < 1e-9));
    }
}
