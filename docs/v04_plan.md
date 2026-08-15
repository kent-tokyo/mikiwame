# mikiwame v0.4 plan: quantify local structure, validate against a real corpus

Investigation/design round only — no production Rust code, `Cargo.toml` version, or
`SCHEMA_VERSION` changed by this document. Presented for review; nothing here is
implemented yet.

## Context

mikiwame 0.3.x (CIF input, then the 0.3.1 correctness fix) moved the project from
"diagnose a hand-built structure" to "read a real file" — but reading isn't the same as
explaining. `local_environment` today reports coordination number, an aggregated
neighbor-species breakdown, and a single ambiguity signal (`shell_gap_ratio`), but not
*which* atoms are neighbors, at what distance, or how distorted the resulting polyhedron
is from an ideal one — all of which AGENTS.md §7.4/§7.5 ask for. 0.3.x's own 31-site
differential validation against pymatgen is real evidence the coordination-number method
is sound, but on 5 idealized textbook structures, not real experimental data.

v0.4's theme: quantify local structure, and validate that quantification against real
(not idealized) structures. Two work streams that don't block each other:

- **mikiwame-side**: expose neighbor-level data (Phase 1) and a first, deliberately narrow
  polyhedral-distortion baseline (Phase 4), validated against a real P1 CIF corpus
  (Phases 2, 3, 5).
- **chematic-side**: a written proposal for typed symmetry operations (below), explicitly
  *not* a v0.4 dependency — no chematic issue/PR exists for it yet (confirmed via
  `gh issue list`/`gh pr list` during the 0.3.1 round), unlike the CIF adapter itself,
  which had an already-merged PR waiting only on a release. Gating v0.4 on unstarted
  external work would repeat a failure mode this project has avoided twice already:
  waiting for a real dependency is fine, blocking *all* forward progress on one that
  doesn't exist yet is not.

## Recommended development order

```
Phase 1: neighbor-level report model, SCHEMA_VERSION 2 -> 3
   |
   +--> Phase 3: pymatgen differential validation extension
   |    (needs Phase 1's data; can run against the existing 5
   |     idealized fixtures immediately, before Phase 2 exists)
   |
Phase 2: P1 CIF corpus + manifest (data curation, not code --
   independent of Phase 1, can run in parallel)
   |
   +--> Phase 3, full run against the real corpus once both
        Phase 1 and Phase 2 are done
        |
        v
Phase 4: tetrahedral/octahedral descriptive distortion metrics
   (sequenced after Phase 1 because it consumes the same
    per-neighbor displacement vectors Phase 1 already surfaces --
    avoids computing them twice)
   |
   v
Phase 5: run distortion metrics over the corpus, look at the
   actual distribution before anyone decides anything about
   thresholds (still descriptive-only output afterward -- a
   distribution informs a *future* threshold decision, it does
   not become one automatically)
   |
   v
Phase 6: docs, semver audit, CHANGELOG, mikiwame 0.4.0
```

---

## Phase 1: neighbor-level report model (`SCHEMA_VERSION` 2 → 3)

### What already exists internally

`diagnostics::coordination::check()` already computes, per site, individual
`(distance, Element)` pairs for every candidate that passes the pairwise
radius-sum-plus-epsilon cutoff (step 2 of the module's own documented method) — not just
the ones that survive the largest-relative-gap step (step 3) into the final shell.
Everything beyond the gap boundary but still within the cutoff is computed and then
**discarded** before `check()` returns. Separately, `chematic_crystal::PeriodicNeighbor`
(what `neighbors_within` actually returns) already carries `neighbor_index: usize`,
`image: [i32; 3]`, `displacement: [f64; 3]`, and `distance: f64` — materially richer than
what survives into mikiwame's own `included: Vec<(f64, Element)>` today. Exposing
neighbor-level data is mostly "stop throwing away data already computed", not new
geometry work — low implementation risk.

### Refinement to the originally sketched `NeighborRecord`

The sketch proposed `site_index`, `element`, `distance_angstrom`, `occupancy`,
`included_in_first_shell`. Two changes, found by reading `PeriodicNeighbor`'s actual
fields:

- Rename `site_index` → `neighbor_site_index`. Living inside
  `SiteLocalEnvironment.neighbors` (whose own `site_index` is the *center*), an
  unqualified `site_index` on the neighbor record is ambiguous about which site it means.
- **Add `image: [i32; 3]`, and document that `neighbor_site_index` is not unique per
  record — that's correct, not a bug.** The same neighbor site can appear more than once
  via different periodic images. Concretely: rock salt's conventional 8-atom cell has
  only 3 distinct Cl site indices, but Na's 6-fold coordination comes from each of those 3
  Cl sites appearing via *two* different image translations (e.g. `[1,0,0]` and
  `[-1,0,0]`), each a geometrically distinct nearest-neighbor instance that must be
  counted separately. Without `image`, a consumer can't tell "6 distinct neighbor
  instances" from "3 sites counted twice by mistake" apart from trusting the count —
  exactly the kind of thing this project's evidence-first stance says not to hide.
- `occupancy: f64` — kept for forward compatibility, but its doc comment should say
  plainly that it is *always* `1.0` in v0.4: `check()` already excludes any neighbor
  belonging to a disordered (multi-species) coincidence group entirely, so nothing that
  reaches the candidate list today is ever partially occupied. The field's presence
  should not imply mikiwame weighs disordered neighbors yet — it doesn't; that's separate,
  unscoped-for-v0.4 work.

**Unique key: `(neighbor_site_index, image)`, not `neighbor_site_index` alone.** Confirmed
by reading `PeriodicNeighbor`'s fields directly: two `NeighborRecord`s can legitimately
share the same `neighbor_site_index` while being different, equally real neighbors, as
long as their `image` differs (rock salt's Na, above, is exactly this case).

**A center site can be its own neighbor, and that's correct, not a self-reference bug.**
In a single-atom primitive cell (simple cubic, BCC, FCC — one site, one element), every
neighbor of that site *is* that same site, reached via a non-zero periodic image
(`neighbor_site_index == center's own site_index`, `image != [0, 0, 0]`). An
implementation that excludes a candidate because `neighbor_site_index == center_index`
(a plausible-looking "don't count yourself" guard) would silently zero out coordination
number for every simple-lattice single-site structure — this must be caught by a test, not
discovered after shipping. **Add simple-cubic, BCC, and FCC one-site primitive-cell
fixtures to the Phase 1 test plan specifically to exercise this** (existing fixtures like
NaCl/CsCl/diamond all have ≥2 distinct sites, so none of them would catch this class of
bug — the same "a same-shaped-but-degenerate case that existing fixtures can't catch"
reasoning already used elsewhere in this project, e.g. `structure_view.rs`'s hexagonal
`cell_volume` test existing specifically because the crate's other fixtures are all
cubic).

```rust
/// One candidate neighbor considered for a site's coordination shell -- not
/// just the ones that ended up counted. `included_in_first_shell = false`
/// entries are exactly what `shell_gap_ratio` measures separation against:
/// what was just outside the resolved shell boundary, computed internally
/// and discarded before this field existed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NeighborRecord {
    /// Index into the structure's site list -- the neighbor, not the
    /// center (see the containing `SiteLocalEnvironment::site_index` for
    /// that). Not unique across one site's neighbor list: the same
    /// neighbor site can appear more than once via different periodic
    /// images (see `image`).
    pub neighbor_site_index: usize,
    /// Element symbol of the neighbor.
    pub element: String,
    /// Which periodic image of `neighbor_site_index` this specific
    /// neighbor instance is, matching
    /// `chematic_crystal::PeriodicNeighbor::image`.
    pub image: [i32; 3],
    /// Euclidean distance from the center to this neighbor image, in
    /// Angstrom.
    pub distance_angstrom: f64,
    /// The neighbor's occupancy. Always `1.0` in v0.4: a neighbor
    /// belonging to a disordered (multi-species) position is excluded
    /// entirely before this list is built, not weighted in.
    pub occupancy: f64,
    /// Whether this candidate survived the largest-relative-gap step and
    /// is counted toward `coordination_number`. `false` entries are what
    /// `shell_gap_ratio` measures separation against.
    pub included_in_first_shell: bool,
}
```

`SiteLocalEnvironment` gains:

```rust
    /// Every candidate neighbor considered (within the radius-sum+epsilon
    /// search bound), not just the ones counted -- see
    /// `NeighborRecord::included_in_first_shell`. Empty exactly when
    /// `coordination_number` is `None`.
    pub neighbors: Vec<NeighborRecord>,
```

### Backward compatibility

`SiteLocalEnvironment`/`MaterialDiagnosticReport` are both already `#[non_exhaustive]` —
external Rust code can never construct them via struct literal, and any `match` already
needs a wildcard arm, so adding a field breaks nothing at the type level. `#[non_exhaustive]`
does not affect the derived `Serialize`/`Deserialize` impls (they bypass literal-construction
rules), so a generic JSON consumer (Python, `jq`, anything not using mikiwame's own Rust
types) sees a purely additive field. A Rust consumer deserializing into their *own* mirror
struct only breaks with `#[serde(deny_unknown_fields)]` (not something mikiwame controls)
on a *newer* report, or by deserializing an *older* report into a struct requiring the new
field non-optionally — exactly why `SCHEMA_VERSION` exists as a self-declared version a
consumer branches on. **Recommendation: bump to `3`, following the exact precedent of the
1→2 bump for adding `local_environment` itself** (see `SCHEMA_VERSION`'s own doc comment)
— no `#[serde(default)]` needed, since that field didn't get one either and this crate's
compatibility story already routes through the version number.

### Implementation cost

Small. `included` becomes `Vec<NeighborRecord>` built directly from each
`PeriodicNeighbor` (its `neighbor_index`/`image` are already available, nothing new to
compute), and `resolve_shell` marks `included_in_first_shell` on each entry instead of
truncating the slice. `neighbor_species` (the existing aggregate) stays as-is — a derived
convenience view existing consumers (the markdown renderer, docs) already depend on.

---

## Phase 2: P1 CIF corpus from the Crystallography Open Database

### Research findings (live-verified against crystallography.net)

- **Query mechanics, confirmed live**: `https://www.crystallography.net/cod/result`
  accepts `spacegroup=P 1` (the space matters — `spacegroup=P1` with no space returns 1
  hit instead of thousands) or `space_group_number=1`, plus element/formula/cell/metadata
  filters. `include_theoretical`/`include_duplicates`/`include_errors` are excluded by
  default.
- **Retrieval, confirmed live**: a single request with `format=zip` returns every matching
  CIF in one HTTP call — tested at 104 entries (368KB, valid zip, no errors), well above
  a 40-60 target. The right mechanism; not 40-60 individual GETs, not the stale ~5GB
  full-database dump.
- **A load-bearing correctness finding, not a formality**: COD's own indexed `sg`/
  `sgNumber` search fields can *disagree with the CIF's own content*. One real,
  live-verified example: COD entry 4000133 is indexed `sg='P 1'`, but its actual CIF has
  `_space_group_IT_number 2` and a 2-operator symmetry loop (`x,y,z` / `-x,-y,-z`) —
  genuinely P-1 (centrosymmetric), not P1. Affects ~17 of 4,390 P1-indexed entries
  (~0.4%). **Consequence: COD's search filter is a candidate list, never a verified-P1
  guarantee — every downloaded CIF must independently pass
  `mikiwame::cif::read_cif`'s own `CifSymmetryStatus::P1` check before entering the
  manifest.** Same "verify against the real artifact, don't trust the source's own
  metadata" discipline this project has applied elsewhere (chematic release timing,
  the perovskite Ti–O radius false-positive). mikiwame's own P1 check already requires
  *both* the space-group name to normalize to `"p1"` *and* the IT number to be `1`, so
  COD's broader `space_group_number=1` filter (which also matches non-primitive
  `C1`/`A1`/`I1`/`F1`/`B1` centered settings) is naturally excluded by mikiwame's own gate
  regardless — another reason `read_cif` itself is the real verification step, not either
  COD field alone.
- **Diversity — a reproducible estimate, not a fixed fact.** Of COD's 4,390 `P1`-indexed
  entries (as queried during this planning round), ~253-359 contain no carbon (a crude
  carbon-absence proxy for "inorganic-looking"; sampled formulas — AlCl3, Na4CoO3,
  GaTe4O11, CuVO3, Sm(BO3) — look well-matched to mikiwame's target domain). This is
  4-6x headroom over a 40-60 target, but COD grows daily (534,674 total entries as of this
  query) and the classification rule is crude (a legitimate inorganic hydrate-carbonate
  mineral was excluded by the no-carbon filter in the sample pulled). **Any number quoted
  from this research must carry its own provenance, not be treated as a stable fact**:
  `estimated from a COD query performed 2026-08-16; classification rule: contains no
  carbon (crude organic/inorganic proxy); candidate count at query time: 4,390 P1-indexed
  total, ~253-359 passing the no-carbon filter`. The corpus-fetching script itself should
  log its own query and date the same way (see the acceptance pipeline below) so this
  number is regeneratable, not just cited from this document.
- **Licensing, confirmed**: CC0 (public domain), directly from COD's homepage and license
  link. Attribution is *requested*, not required — every CIF already self-carries
  `_publ_author_name`/`_journal_*`/DOI plus a COD source line, so vendoring CIFs with
  original headers intact carries attribution forward automatically. Standard database
  citation: Gražulis et al. (2009), *J. Appl. Cryst.* 42(4), 726-729,
  doi:10.1107/S0021889809016690.
- **One documentation gap, reported plainly**: COD's wiki doesn't explicitly address
  subset-redistribution-inside-a-third-party-repository (checked their redeposit-policy
  and citing pages; neither covers it). Not a real legal risk — CC0 unambiguously permits
  redistribution — but worth recording that this exact scenario isn't spelled out in
  COD's own FAQ.
- **One tension worth surfacing, not hiding**: `crystallography.net/robots.txt` is a
  blanket `Disallow: /`, while COD's own wiki simultaneously documents and invites
  exactly this kind of REST API/curl usage as the intended access method.
  `robots.txt` conventionally targets exhaustive automated crawling, not a single
  documented API call for a few dozen records, and COD's own docs are the more specific
  and more directly-on-point guidance — but the contradiction is real, so it's recorded
  rather than silently assumed away.

### Design

- **Retrieval**: one `format=zip` request against `/cod/result`, filtered to
  `space_group_number=1` (or `spacegroup=P 1`) with element-family filters run as a few
  separate small queries (halides, oxides, chalcogenides, ...) rather than one giant dump,
  to keep the candidate set hand-curatable.
- **No live network access during `cargo test`**: fixtures are vendored files
  (`fixtures/cif-p1/structures/cod-<id>.cif`), fetched once by a curation script
  (`scripts/fetch_cod_corpus.py`) run by hand — the same manually-reproduced, not
  CI-wired pattern `scripts/differential_validation.py` already established.
- **Target composition**: 40-60 structures spanning ionic (halides), covalent, oxides,
  chalcogenides, semiconductors, perovskite-related, at least a few genuinely-distorted
  (non-ideal) tetrahedral/octahedral examples, and a handful with disorder — hand-selected
  from the verified candidate pool, not auto-sampled.
- **Placement**: `fixtures/cif-p1/` sits alongside (not merged with) the
  already-recorded, still-undecided `fixtures/synthetic/` shared-JSON-fixture direction
  from `tasks/todo.md` — that refactor is an independent decision not re-opened here.

### Acceptance pipeline — COD's search index is candidate extraction only; the CIF body is the source of truth

The sg/sgNumber contradiction found this round means no single field from COD's search
index may be trusted as "this structure is verified." Every candidate must pass all of
the following before entering the manifest, not just the P1 check:

1. **COD query** — extracts *candidates* only, nothing more. Never treated as pre-verified.
2. **Parse with `mikiwame::cif::read_cif`** (the actual shipped adapter) — a CIF that
   fails to parse at all is dropped.
3. **Require `symmetry == CifSymmetryStatus::P1`** — the hard gate already decided for
   0.3.1; anything else is dropped, regardless of what COD's own `sg`/`sgNumber` fields
   claimed.
4. **Also parseable by pymatgen** (`pymatgen.io.cif.CifParser` or equivalent) — a CIF
   `chematic-mol` accepts but pymatgen rejects (or vice versa) is a real cross-tool
   disagreement worth knowing about before it's silently in a "known-good" corpus, dropped
   from this round's corpus either way (a genuine parser-disagreement case is interesting
   enough to record separately, not just discard, but doesn't block the rest of the
   pipeline).
5. **chematic and pymatgen must agree on site count, element set, and per-site occupancy**
   for the accepted structure — this is the direct check for "the CIF claims P1 but
   actually only contains the asymmetric unit," the specific corrupted-CIF failure mode
   that's otherwise hard to detect: if two independent parsers built on independent CIF
   readers agree on exactly what atoms exist and where, that's real cross-validation, not
   just "one tool didn't crash."
6. **No non-finite values, no occupancy anomalies (sum > 1 + tolerance), no duplicate
   sites** — reuses mikiwame's own existing `INPUT_NONFINITE_COORDINATE`/
   `INPUT_INVALID_OCCUPANCY`/`SITE_DUPLICATE` checks (run the candidate through `analyze`
   itself and require a clean `Verdict::StructurallyConsistent` input-quality pass before
   it's corpus-worthy — dogfooding mikiwame's own diagnostics as part of curating its own
   test data).
7. **Where available, cross-check `_chemical_formula_sum` and `_cell_formula_units_Z`**
   against the composition reconstructed from the actual parsed atom-site occupancies —
   another independent signal against "P1-labeled but incomplete" CIFs, on top of #5's
   chematic-vs-pymatgen cross-check. Not always present in every CIF (older/converted
   files sometimes omit these tags), so this is a bonus check when available, not a hard
   gate that would reject an otherwise-valid file solely for a missing optional tag.
8. **Record the exact COD query and retrieval timestamp used**, plus the source CIF's
   SHA-256, in the manifest — makes the corpus's provenance regeneratable, not just a
   frozen snapshot with no record of how it was assembled.

Manifest (`fixtures/cif-p1/manifest.jsonl`, one JSON object per line):

```json
{
  "id": "cod-1010563",
  "source": "COD",
  "source_identifier": "1010563",
  "source_url": "https://www.crystallography.net/cod/1010563.cif",
  "source_query": "space_group_number=1&nel1=C&format=zip",
  "retrieved_at": "2026-08-16T00:00:00Z",
  "license": "CC0-1.0",
  "citation": "Gražulis et al. 2009, J. Appl. Cryst. 42(4), 726-729, doi:10.1107/S0021889809016690",
  "formula": "AlCl3",
  "original_space_group_h_m": "P 1",
  "original_space_group_number": 1,
  "verified_p1_by_mikiwame": true,
  "verified_parseable_by_pymatgen": true,
  "chematic_pymatgen_composition_agree": true,
  "formula_z_consistent": true,
  "site_count": 8,
  "sha256": "<hash of the vendored CIF file, exact bytes>"
}
```

`verified_p1_by_mikiwame: true` is what a test re-checks by actually re-running
`read_cif` on the vendored file, catching drift between the manifest's claim and the
file's real content — same reasoning extends to `verified_parseable_by_pymatgen` and
`chematic_pymatgen_composition_agree`, both re-checkable, not just recorded once and
trusted forever.

---

## Phase 3: differential validation extension

Once `NeighborRecord` exists, mikiwame's JSON output carries full per-neighbor detail.
pymatgen's `CrystalNN` exposes equivalent detail via `get_nn_info(structure, site_index)`
— **exact key names/shape should be re-verified against the actually-installed pymatgen
version at implementation time**, not assumed from memory, matching this project's
existing discipline of never trusting an API surface without checking the real installed
package.

Comparison splits into genuinely separate metrics, since they can disagree independently
and for different reasons:

1. **Coordination number** — already done, unchanged.
2. **Neighbor element multiset match** — compare the *multiset* of neighbor elements per
   site (not an ordered list; neither tool has a reason to agree on order). Report a
   per-site boolean and an aggregate match rate.
3. **Neighbor distance agreement** — for sites where the multiset matches, pair neighbors
   by greedy nearest-distance matching within each matched element, compute per-pair
   absolute distance error, report aggregate max/mean absolute error — matching
   `docs/validation.md`'s existing style of reporting real numbers, not just pass/fail.
4. **First-shell membership** — once gap-boundary candidates are visible via
   `included_in_first_shell = false`, check whether pymatgen's own boundary agrees. This
   is the least-precedented comparison: pymatgen's cutoff logic is a *different*,
   independently-defensible algorithm from mikiwame's largest-relative-gap step, so
   **disagreement here is not automatically a mikiwame bug** — report it, but document
   explicitly (same as the existing perovskite-O "2 vs. 6" writeup) that it measures
   "do two defensible methods draw the boundary in the same place," not correctness.
5. **Distortion metrics** (once Phase 4 ships) — compare against whatever pymatgen exposes
   here (possibly `pymatgen.analysis.chemenv`, a heavier dependency than `CrystalNN`
   alone) — a separate research pass at implementation time, not assumed now.

Report format extends the existing per-site table plus a summary distinct from the
existing "N mismatches out of M sites" line: structures/sites compared, CN exact-match
rate (existing), neighbor-set exact-match rate (new), distance error mean/max (new),
shell-boundary agreement rate (new, explicitly labeled as "two methods, not a correctness
check"), and mikiwame's abstain count (sites where `coordination_number` is `None` —
tracked as a feature, not a defect: not answering when unsure should count in mikiwame's
favor, not against it).

This runs against the existing 5 idealized fixtures first (fast, no corpus needed, proves
the new comparison code is right where the answer is already known) before running
against the Phase 2 corpus.

---

## Phase 4: polyhedral distortion baseline (tetrahedral/octahedral)

### Research findings (citations verified via WebSearch)

Two methods have well-established citations specific to periodic inorganic/mineralogical
crystallography, compute from measured geometry with no iterative fitting, and are simple
enough to implement and unit-test with confidence:

1. **Quadratic elongation (λ) and bond angle variance (σ²)** — Robinson, K.; Gibbs, G.V.;
   Ribbe, P.H. (1971). "Quadratic Elongation: A Quantitative Measure of Distortion in
   Coordination Polyhedra." *Science* 172(3983), 567-570. doi:10.1126/science.172.3983.567.
   One of the most-cited papers in mineralogical crystallography; the field's default.
   - λ = (1/n) Σᵢ(lᵢ/l₀)², lᵢ = measured center-to-vertex distances, l₀ = center-to-vertex
     distance of the regular polyhedron of the *same volume*, n = 6 or 4. Ideal: λ = 1.
   - σ² = (1/(m-1)) Σᵢ(θᵢ - θ₀)², θᵢ = measured bond angles, θ₀ = ideal angle (90° for an
     octahedron's 12 *cis* angles — the 3 *trans* angles excluded; 109.47° for a
     tetrahedron's 6 angles). Ideal: σ² = 0.
2. **Baur's distortion index (DI)** — Baur, W.H. (1974). "The geometry of polyhedral
   distortions. Predictive relationships for the phosphate group." *Acta Cryst.* B30,
   1195-1215. doi:10.1107/S0567740874004560.
   - DI = (1/n) Σᵢ|dᵢ - d_mean|/d_mean. Ideal: DI = 0. Entirely self-referential (no
     external "ideal" bond length needed, unlike λ) — even simpler than #1.
5. **Central-site-to-vertex-centroid distance, plain form** — needs **no citation**: it's a
   direct geometric measurement (Euclidean distance from the central atom to the
   arithmetic mean of *unwrapped Cartesian* neighbor positions — see below), not a fitted
   statistic. Important distinction found during research: the literature term
   "eccentricity" (Balić-Žunić & Makovicky 1996, *Acta Cryst.* B52, 78-81,
   doi:10.1107/S0108768195008251, implemented in IVTON for inorganic crystal structures)
   refers to a *different*, fitted quantity — displacement from the "best centre" that
   minimizes variance of distances to all vertices, found iteratively, not the plain
   centroid. **mikiwame does not implement or claim Balić-Žunić & Makovicky's method or
   its "eccentricity" — the field is named `central_to_vertex_centroid_distance_angstrom`
   specifically to avoid reusing the literature term for a different computation and
   misattributing a citation it doesn't match.**

Two more were researched and **excluded from the v0.4 baseline**:

3. **τ4 / τ4′** (Yang, Powell & Houser 2007; Okuniewski et al. 2015) — real, citable
   four-coordinate tetrahedral-vs-square-planar index, but no evidence it's established in
   periodic inorganic/mineralogical crystallography specifically; it's a molecular
   coordination-chemistry metric (Cu/Zn/Ni complexes). Using it here would itself need
   flagging as a domain mismatch.
4. **Continuous Shape Measures (CShM)** — Pinsky & Avnir (1998), *Inorg. Chem.* 37(21),
   5575-5582; crystal-domain use confirmed via Link & Niewa (2023), "Polynator,"
   *J. Appl. Cryst.* 56, 1855-1864 (a CShM-based tool built explicitly for periodic
   CIF/crystal-structure analysis). The most rigorous method found — a genuine
   ideal-template-fit residual via optimal rotation/scaling/vertex-permutation
   (Procrustes-style) — and the best future candidate for real CN=4 type-classification.
   **Deferred**: the Procrustes/permutation-search implementation is meaningfully harder
   to get right and independently verify than #1/#2/#5's closed-form sums — same "wait for
   a later round rather than ship under time pressure with a correctness risk" pattern
   already applied to external dependencies, just internal this time.

Honest caveat: the exact formulas for CShM and Balić-Žunić/Makovicky's "best centre" were
confirmed via strong, consistent secondary corroboration (software docs, citing papers),
not by reading the paywalled primary PDFs directly. v0.4's baseline doesn't depend on
either, so this doesn't block v0.4 — but a later round picking up CShM should get the
primary sources first.

### v0.4 baseline

Quadratic elongation, bond angle variance, Baur's DI, and plain vertex-centroid distance —
all four computed directly from measured bond lengths/angles, no fitting except λ's
same-volume regular-polyhedron reference (a closed-form computation from the polyhedron's
own measured volume, not a lookup table). Exact formulas, including the octahedral
cis/trans angle-pairing method that's the real implementation risk here, are pinned down
below rather than left to be reinvented at implementation time.

**No automatic tetrahedral-vs-square-planar classification in v0.4.** No established,
citable, crystal-domain-appropriate discriminator is ready today (τ4 is citable but
domain-mismatched; CShM is domain-appropriate but deferred). Consistent with this
project's own hard-learned lesson (the coordination-ambiguity cutoff found backwards
before shipping, see `docs/validation.md`): report the geometric descriptors for every
CN=4/CN=6 site, and leave `candidate_geometry` as `"ambiguous"` rather than guess.

```json
{
  "site_index": 4,
  "coordination_number": 6,
  "candidate_geometry": "octahedral",
  "quadratic_elongation": 1.012,
  "bond_angle_variance_deg2": 3.72,
  "baur_distortion_index": 0.0041,
  "central_to_vertex_centroid_distance_angstrom": 0.018,
  "classification": "descriptive_only",
  "limitations": []
}
```

`candidate_geometry` is populated only from `coordination_number` (4 → "tetrahedral or
square-planar, unclassified" / 6 → "octahedral"), plus `"ambiguous"` for anything else —
never inferred from the distortion metrics themselves in v0.4, exactly to avoid the
invented-classifier trap. `classification` is always `"descriptive_only"`: no finding, no
contribution to `overall.verdict`/`anomaly_burden`.

### Report shape: minimal (distance + aggregates only), not raw vectors

Two options were weighed for how much geometry the *report* itself exposes:

- **A (minimal)**: `NeighborRecord` carries `distance_angstrom` only (as already specced in
  Phase 1); distortion metrics are computed internally and only the aggregated numbers
  (λ, σ², DI, vertex-centroid distance) are reported.
- **B (evidence-first)**: also expose each neighbor's Cartesian displacement vector from
  the center (`displacement_cartesian_angstrom: [f64; 3]`) in `NeighborRecord`, so a
  consumer could recompute angles/distortion independently from the report alone.

**v0.4 takes A.** B is closer to this project's usual evidence-first instinct, but two
concrete costs tip it the other way for a first cut: it roughly doubles per-neighbor JSON
size, and it breaks the existing metamorphic-invariance testing style
(`tests/metamorphic.rs` already tests that a rigid lattice rotation doesn't change
verdicts/finding codes) — a rotated structure's raw displacement *vectors* change even
though every distance, dot product, and angle between them is invariant, so a metamorphic
test over B would need to compare derived quantities (distance, angle) rather than the raw
vectors directly, which is extra test-design surface for a v0.4 first cut. B stays a
candidate for a later round if a real consumer need for raw vectors shows up; nothing in
A's schema forecloses adding it later (additive field, same `#[non_exhaustive]` story as
Phase 1).

### Internal computation must use image-unwrapped Cartesian vectors — never a plain average of fractional coordinates

This is an implementation requirement, not a reporting-format choice, and needs to be
explicit in the plan because getting it wrong would silently corrupt every metric: a
neighbor's true position for geometry purposes is
`lattice.frac_to_cart(neighbor_fractional + image)`, i.e. the fractional coordinate
*translated by its periodic image* before conversion to Cartesian — exactly the
`PeriodicNeighbor::displacement` vector chematic-crystal already computes internally
(see Phase 1's "what already exists" section). Averaging *fractional* coordinates directly
(without applying each neighbor's own `image` first) breaks the instant any coordinating
polyhedron straddles a unit-cell boundary — for example, an octahedron whose six ligands
include one just past the far edge of the cell wrapped back to fractional `~0.98` instead
of `~-0.02` would pull the naive fractional average toward completely the wrong centroid.
Both the bond-angle computation (needs true 3D vectors between neighbor pairs) and the
centroid-displacement metric (needs the true unwrapped centroid) depend on this being done
right; both should be computed from the same internally-built unwrapped Cartesian
positions, not two independent implementations that could disagree.

### Formulas, precisely (implementation-blocking detail, not just citations)

**Baur's distortion index**: D = (1/n) Σᵢ |lᵢ − l_avg| / l_avg, where lᵢ are the n
measured center-to-ligand bond lengths and l_avg is their own arithmetic mean — a pure
bond-length-variability measure, entirely self-referential (needs no external "ideal"
length).

**Quadratic elongation**: λ = (1/n) Σᵢ (lᵢ / l₀)², where l₀ is **the center-to-vertex
distance of a regular polyhedron of the same volume as the measured (possibly distorted)
one** — not the mean bond length, and not a fixed constant. Computing l₀ requires two
steps:
1. Compute the measured polyhedron's actual volume V from its real (unwrapped Cartesian)
   vertex positions. For CN=4 (tetrahedral), this is the direct tetrahedron-volume formula
   from the 4 ligand positions: V = (1/6)|(v₂−v₁)·((v₃−v₁)×(v₄−v₁))|. For CN=6
   (octahedral), this needs the polyhedron's face topology (which vertex triples form the
   8 triangular faces) to decompose into tetrahedra from the center — **the same
   opposite-vertex-pairing problem described below for bond-angle variance, not a second,
   independent piece of geometry**; solve it once, reuse for both volume and angle-variance.
2. Invert the standard volume-circumradius relationship for a *regular* polyhedron of that
   volume to get l₀: for a regular octahedron, V = (4/3)·R³, so l₀ = R = (3V/4)^(1/3); for
   a regular tetrahedron, V = (8√3/27)·R³, so l₀ = R = ((27/(8√3))·V)^(1/3). These are
   direct consequences of solid geometry (not something needing their own citation beyond
   Robinson/Gibbs/Ribbe's already-cited method) but are exactly the kind of detail worth
   pinning down now rather than reinventing differently at implementation time.

**Bond angle variance**: σ² = (1/(m−1)) Σᵢ (θᵢ − θ₀)², where θᵢ are measured L-center-L
angles. **The angle set and θ₀ are not the same for every pair — this is the single
highest implementation-accident risk in Phase 4**, per direct concern already raised:

- **Tetrahedron (CN=4)**: no cis/trans distinction exists. All C(4,2) = 6 pairwise angles
  are compared against the single ideal angle θ₀ = 109.47°, m = 6.
- **Octahedron (CN=6)**: the 6 ligands form 3 *trans* (opposite) pairs and the remaining
  C(6,2) − 3 = 12 pairs are *cis*. **These must not be pooled against one θ₀.** Trans
  pairs compare against θ₀ = 180° (3 angles); cis pairs compare against θ₀ = 90° (12
  angles); m = 15 total (both angle sets contribute to the same variance sum, each against
  its own θ₀ — this is how Robinson et al.'s original definition treats the two angle
  types together). Determining which pairs are trans is a one-time perfect-matching
  problem on the 6 ligand displacement vectors: for each pair, compute the cosine of the
  angle between their (unwrapped Cartesian) displacement vectors from the center; the 3
  pairs whose cosine is closest to −1 (angle closest to 180°) are trans. **This pairing
  can become genuinely unstable for a real, sufficiently distorted or mischaracterized
  site** — see the Go/No-Go criteria below, where an unstable cis/trans pairing is an
  explicit defer-to-v0.5 condition for that specific site's distortion output, not
  something to force through with a best-effort guess.

### Depends on Phase 1

Needs `NeighborRecord`'s underlying displacement data (already carried by
`PeriodicNeighbor::displacement`, even though it isn't exposed in the v0.4 report itself —
see "report shape" above) to compute inter-neighbor angles and the unwrapped centroid — a
consumer of geometry Phase 1's internal computation already builds, not a new geometry
path.

---

## Phase 5: corpus-wide metric distribution

Once Phases 2 and 4 both exist: run the distortion metrics across every corpus structure,
tabulate the distribution (min/max/mean/percentiles) of λ, σ², DI, and vertex-centroid
distance, cross-referenced against Phase 3's differential-validation results on the
same corpus. Output is a distribution — informing a *future* threshold decision, not
becoming one automatically. Same discipline as everywhere else in this project: a
distribution is evidence to reason from, not license to invent a cutoff.

## Phase 6: docs, semver, release

Update README.md/README_ja.md, CHANGELOG.md, `tasks/todo.md`, `ROADMAP.md`,
`docs/chematic-prerequisites.md`/`docs/validation.md` as appropriate. Version: `0.4.0` —
additive at the Rust-type level (`#[non_exhaustive]` throughout), but the JSON schema
shape changes materially (new `neighbors`/distortion fields), matching this project's own
established precedent that a schema-shape change is minor-version-worthy even though nothing
breaks existing Rust callers.

---

## chematic symmetry-expansion proposal (parallel track, not a v0.4 gate)

Already sketched in `docs/chematic-prerequisites.md`'s 2026-08-15 addendum; refined here
with explicit acceptance criteria. Not implemented in mikiwame, not filed as an actual
chematic issue/PR yet — a written proposal only.

```rust
// chematic-crystal
pub struct SymmetryOperation {
    pub rotation: [[i32; 3]; 3],
    pub translation: [Rational; 3],
}
impl SymmetryOperation {
    pub fn apply(&self, coord: FractionalCoord) -> FractionalCoord;
}
pub fn expand_asymmetric_unit(
    structure: &PeriodicStructure,
    operations: &[SymmetryOperation],
    tolerance: f64,
) -> Result<PeriodicStructure, CrystalError>;
```

```rust
// chematic-mol
pub struct CifPeriodicResult {
    pub structure: PeriodicStructure,
    pub symmetry: CifSymmetryStatus,
    pub symmetry_operations: Vec<SymmetryOperation>, // empty when P1
}
```

**Acceptance criteria**:

1. **Typed parse of CIF symop strings** (`-x, y, -z+1/2` etc.), including negative
   coefficients, fractional translations (at minimum halves/thirds/quarters/sixths — the
   denominators that actually occur in real space groups), and axis permutation. Malformed
   operators are a parse error, not silently skipped (fail-closed, matching every other
   CIF-adapter decision this project has made).
2. **`[0, 1)` wrapping** via the same `rem_euclid`-style convention
   `FractionalCoord::wrapped` already uses elsewhere in chematic-crystal.
3. **Special-position deduplication** — an operation mapping a site onto itself or an
   already-generated site (within tolerance) must not create a duplicate.
4. **Disorder-aware species merging** — two sites landing at the same position with
   different species merge into one multi-species `PeriodicSite`, still passing its own
   occupancy-sum validation.
5. **Deterministic ordering** — same input twice, same output order — needed for
   mikiwame's own `deterministic mode: true` claim (`Provenance::deterministic`, surfaced
   via `doctor`) to remain true once this is consumed.
6. **Correctly reduces to P1** — an empty/identity-only operation list leaves the input
   unchanged (mod ordering); the trivial case must be exact, since it's the easiest to
   regression-test and the easiest to get subtly wrong.
7. **Round-trip fixture** — expand a real, published space group's full symop list (e.g.
   chematic-mol's own existing C2/c test fixture) and check the result against a
   known-correct expansion (cross-checked against pymatgen's own space-group machinery,
   or a hand-verified textbook expansion for a small, well-known group).

Non-P1 expansion becomes a v0.5 *candidate* once/if this lands upstream — not promised for
a specific version yet, since there's no upstream timeline to commit against.

---

## Scientific risks

1. **Distortion metrics with no citable basis would violate AGENTS.md §21 outright.**
   Mitigated by design (descriptive-only, never a finding) and by only shipping the two
   methods with directly-verified crystallography-specific citations.
2. **CN=4 ambiguity, if resolved by an invented cutoff, repeats a mistake this project
   already caught once** (see `docs/validation.md`'s coordination-ambiguity writeup).
   Mitigated by shipping no automatic classification at all in v0.4.
3. **A P1-only corpus is not neutral evidence.** COD's P1 subset skews organic
   (91.5% contain C and H); the confirmed inorganic-leaning subset (~253-359 entries) is
   real but a minority. Differential-validation results from this corpus should be
   reported with that composition explicit, not presented as validating the full
   applicable-structure-class claim `doctor` already makes.
4. **pymatgen's neighbor-boundary algorithm is not ground truth.** Already true for
   coordination number (documented in `docs/validation.md`'s perovskite-O writeup);
   extending comparison to per-neighbor/shell-boundary level increases the surface where
   two legitimately-different methods disagree without either being wrong. Report
   numbers, explain the reason, don't chase agreement for its own sake.
5. **COD's own metadata can be wrong** (the sg/sgNumber contradiction found this round).
   Mitigated by never trusting COD's filter as ground truth — `read_cif`'s own
   `CifSymmetryStatus::P1` check is the actual gate.

## Test plan

- **Phase 1**: extend `resolve_shell`'s existing tests (CsCl/perovskite-Ti "14 narrowed to
  N" cases already exist) to assert excluded candidates appear with
  `included_in_first_shell: false`. Extend `tests/known_good_fixtures.rs` to assert a full
  neighbor list (element + count + image multiplicity) on at least one fixture. **New:**
  one-site primitive-cell fixtures for simple cubic, BCC, and FCC, asserting (a) the
  correct coordination number (6, 8, 12 respectively) and (b) that every neighbor record's
  `neighbor_site_index` equals the structure's own single site index with a non-zero
  `image` — the case an accidental `neighbor_site_index == center_index` exclusion guard
  would silently break, and the case no existing fixture (all ≥2 distinct sites) exercises.
- **Phase 2**: a test verifying every manifest entry's `sha256` matches its actual vendored
  file, and that `original_space_group`/`verified_p1_by_mikiwame` claims are consistent
  with what `read_cif` reports for that file today — catches manifest/data drift.
- **Phase 3**: re-run against the 5 idealized fixtures first (fast, answer already known)
  before trusting the extended comparison against the real corpus.
- **Phase 4**: hand-verified cases against exact geometric constructions (a perfect
  regular tetrahedron/octahedron built directly from angles, not from any existing
  fixture) asserting metrics come out at their ideal value (λ=1, σ²=0, DI=0) — the same
  "verify the ideal case by construction first" discipline already used elsewhere (see
  `structure_view.rs`'s hexagonal `cell_volume` test, chosen specifically because a
  cubic-only fixture can't catch a transposition bug).

## Go/no-go conditions (per phase, not one blanket v0.4 gate)

- **Phase 1 ships if**: the backward-compatibility analysis holds under actual
  implementation, and the new "previously-discarded candidates now visible" test passes,
  including the simple-cubic/BCC/FCC self-neighbor-via-nonzero-image cases.
- **Phase 2 ships if**: a sufficiently diverse 40-60 structure sample can actually be
  hand-curated from the pool passing the full 8-point acceptance pipeline above — if it
  turns out too narrow, ship a smaller, explicitly-scoped corpus with the gap documented
  rather than something unrepresentative presented as comprehensive.
- **Phase 4 ships if**: at least one distortion metric has a verified citation — already
  satisfied (two do). If this changes during implementation, Phase 4 does not ship at all
  rather than shipping an invented formula.
- **v0.4 overall ships if**: Phases 1-3 are done (no unverified-citation dependency).
  Phase 4 ships alongside them per its own gate above.

### v0.4 Go/No-Go criteria (cross-cutting, checked against real implementation output)

**Go if all of:**

- Neighbor identity is stable as `(neighbor_site_index, image)` — no two distinct real
  neighbor instances collide on this key, no legitimate neighbor is dropped by it.
- Simple cubic, BCC, FCC, and NaCl all correctly enumerate their periodic-image neighbors
  (the specific case Phase 1's new fixtures above exist to catch).
- 40+ real P1 CIFs can be fixed (finalized, vendored) after passing the full acceptance
  pipeline.
- chematic and pymatgen agree on site count/species/occupancy for every accepted corpus
  structure (acceptance-pipeline criterion #5, checked at scale across the whole corpus,
  not just spot-checked).
- Neighbor distance error between mikiwame and pymatgen stays within numerical tolerance
  across the corpus (Phase 3's distance-agreement metric).
- Every distortion metric evaluates to its exact ideal value on a hand-constructed ideal
  tetrahedron/octahedron (λ=1, σ²=0, DI=0, vertex-centroid distance=0).
- Distortion metrics move monotonically or otherwise reasonably as synthetic distortion is
  incrementally increased on a test structure (a sanity check that the metrics respond to
  distortion in the expected direction, not just that they hit the right value at zero
  distortion).

**No-Go for v0.4 (defer to v0.5 or later), per condition — not a reason to block the whole
release, since these are independently scoped:**

- Octahedral cis/trans vertex pairing doesn't stabilize for a real corpus site (the
  bond-angle-variance risk flagged above) — that *site's* distortion output is omitted
  (`not_computed_reason` recorded, same pattern as coordination number's own "not
  computed, not guessed" convention), not forced through with an unstable guess.
- A metric's value changes depending on which periodic image was chosen for an otherwise
  equivalent neighbor (would indicate a real bug in the unwrapped-Cartesian-vector
  construction, not a scope question — this is a hard implementation bug to fix before
  shipping, not a deferrable scope item).
- Coordination-polyhedron definition for a disordered (multi-species) site remains
  undefined (already out of scope going in — see below — but recorded here as a Go/No-Go
  condition too since the risk is "someone tries to force a definition under release
  pressure," not "no one remembers this is unscoped").
- Shape classification would require an arbitrary/uncited threshold to resolve (the exact
  trap `candidate_geometry: "ambiguous"` already exists to avoid).
- COD corpus quality control isn't actually reproducible from the manifest + curation
  script (acceptance-pipeline criterion #8's whole point).

## Explicitly out of scope for v0.4

Oxidation states/charge neutrality, radius-only short-distance findings, any invented
coordination-ambiguity threshold, a mikiwame-authored CIF symop parser, CIF-directory
batch processing, prototype/structure-type similarity, an overall anomaly score, and any
actual non-P1 symmetry expansion. Same reasoning already recorded in `tasks/todo.md` for
each — no new information this round changes any of them.
