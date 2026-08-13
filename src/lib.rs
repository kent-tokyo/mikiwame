//! `mikiwame` (見極め): explainable materials structure diagnostics.
//!
//! Given a 3D periodic crystal structure, `mikiwame` explains what is
//! structurally unusual about it, where, and on what evidence — it does not
//! return a single opaque "goodness" score.
//!
//! # What this answers
//!
//! Input-quality problems, lattice/cell validity, and (as more diagnostic
//! components land) coordination, distortion, composition, and disorder
//! anomalies — each as a machine-readable [`finding::Finding`] with its own
//! severity, confidence, and evidence.
//!
//! # What this does not answer
//!
//! Thermodynamic stability, formation energy, band structure, or
//! synthesizability. [`model::Verdict::StructurallyConsistent`] is not a claim
//! that a structure is stable or synthesizable. See `docs/scientific_scope.md`
//! for the full claims policy.
//!
//! # Status
//!
//! v0.1, early: only the checks that need no invented empirical threshold have
//! shipped so far (see `tasks/todo.md` for what is deferred and why).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod config;
mod diagnostics;
pub mod error;
pub mod finding;
pub mod model;
pub mod provenance;
mod radii;
pub mod report;
pub mod structure_view;

pub use config::AnalysisConfig;
pub use error::MikiwameError;
pub use finding::{Evidence, Finding, FindingCode, FindingScope, NumericEvidence};
pub use model::{ApplicabilityLevel, ClosedRange, MetricCode, Score01, Severity, Unit, Verdict};
pub use provenance::Provenance;
pub use report::{
    ApplicabilityAssessment, ComponentAssessment, ComponentName, ComponentStatus, InputSummary,
    MaterialDiagnosticReport, OverallAssessment, SCHEMA_VERSION, Suggestion,
};
pub use structure_view::{OwnedStructure, PeriodicStructureView, Site};

use std::collections::HashSet;

/// Analyzes one structure and returns its diagnostic report.
///
/// Unlike the illustrative signature in AGENTS.md §13, this returns
/// [`MaterialDiagnosticReport`] directly rather than a `Result`: every
/// malformed-input case mikiwame checks for in v0.1 is represented as a
/// finding inside a normally-returned report (`Verdict::InvalidInput`), not as
/// an error — there is currently no condition under which analysis of a
/// well-formed [`PeriodicStructureView`] fails to produce a report. See
/// `docs/architecture.md` for the reasoning; [`error::MikiwameError`] is used
/// by fallible value constructors ([`Score01::new`], [`ClosedRange::new`])
/// instead.
pub fn analyze<S: PeriodicStructureView>(
    structure: &S,
    _config: &AnalysisConfig,
) -> MaterialDiagnosticReport {
    let mut findings = Vec::new();
    let mut components = Vec::new();

    let input_quality_outcome = diagnostics::input_quality::check(structure);
    let fatal = input_quality_outcome.fatal;
    findings.extend(input_quality_outcome.findings);
    components.push(ComponentAssessment {
        name: ComponentName::InputQuality,
        status: ComponentStatus::Ran,
    });

    if fatal {
        for name in [ComponentName::SiteSeparation, ComponentName::Disorder] {
            components.push(ComponentAssessment {
                name,
                status: ComponentStatus::Skipped {
                    reason: "input quality check found a fatal problem".to_string(),
                },
            });
        }
        return build_report(
            model::Verdict::InvalidInput,
            fatal,
            findings,
            components,
            input_summary(structure),
        );
    }

    findings.extend(diagnostics::separation::check(structure));
    components.push(ComponentAssessment {
        name: ComponentName::SiteSeparation,
        status: ComponentStatus::Ran,
    });

    findings.extend(diagnostics::disorder::check(structure));
    components.push(ComponentAssessment {
        name: ComponentName::Disorder,
        status: ComponentStatus::Ran,
    });

    let verdict = decide_verdict(&findings);
    build_report(
        verdict,
        fatal,
        findings,
        components,
        input_summary(structure),
    )
}

/// Analyzes each structure independently; one structure's result never
/// affects another's, and one bad structure never prevents the rest of the
/// batch from being reported. Input order is preserved.
pub fn analyze_batch<S: PeriodicStructureView>(
    structures: &[S],
    config: &AnalysisConfig,
) -> Vec<MaterialDiagnosticReport> {
    structures.iter().map(|s| analyze(s, config)).collect()
}

fn input_summary<S: PeriodicStructureView>(structure: &S) -> InputSummary {
    let elements: HashSet<&str> = structure
        .sites()
        .iter()
        .map(|site| site.element.as_str())
        .collect();
    InputSummary {
        site_count: structure.sites().len(),
        distinct_element_count: elements.len(),
    }
}

/// `Info`-severity findings (e.g. `DISORDER_PRESENT`) don't move the verdict
/// off `StructurallyConsistent` on their own — AGENTS.md §7.7 is explicit
/// that disorder is not itself an anomaly, and severity, not code identity,
/// is what verdict decisions key on (AGENTS.md §9).
fn decide_verdict(findings: &[Finding]) -> model::Verdict {
    match findings.iter().map(|f| f.severity).max() {
        None | Some(Severity::Info) => model::Verdict::StructurallyConsistent,
        Some(Severity::Critical) => model::Verdict::StrongAnomalyDetected,
        Some(_) => model::Verdict::ReviewRecommended,
    }
}

/// Finding codes at the highest severity present, in first-seen order,
/// without duplicates.
fn dominant_findings(findings: &[Finding]) -> Vec<FindingCode> {
    let Some(max_severity) = findings.iter().map(|f| f.severity).max() else {
        return Vec::new();
    };
    let mut seen = HashSet::new();
    findings
        .iter()
        .filter(|f| f.severity == max_severity)
        .map(|f| f.code)
        .filter(|code| seen.insert(*code))
        .collect()
}

fn build_report(
    verdict: model::Verdict,
    fatal: bool,
    findings: Vec<Finding>,
    components: Vec<ComponentAssessment>,
    input: InputSummary,
) -> MaterialDiagnosticReport {
    let dominant = dominant_findings(&findings);
    // v0.1's checks are exact/deterministic (no fuzzy heuristics), so a
    // successfully-computed verdict is reported at full confidence; nothing
    // yet produces a partial-confidence verdict.
    let confidence = Score01::new(1.0).expect("1.0 is a valid Score01");
    let applicability = if fatal {
        ApplicabilityAssessment {
            level: ApplicabilityLevel::NotApplicable,
            reasons: vec!["input could not be validated".to_string()],
        }
    } else {
        // v0.1 does not yet detect out-of-domain structure classes (surfaces,
        // amorphous, polymers — AGENTS.md §5); everything that passes input
        // quality is treated as fully applicable until that lands.
        ApplicabilityAssessment {
            level: ApplicabilityLevel::FullyApplicable,
            reasons: Vec::new(),
        }
    };
    MaterialDiagnosticReport {
        schema_version: SCHEMA_VERSION,
        input,
        overall: OverallAssessment {
            verdict,
            anomaly_burden: None,
            confidence,
            dominant_findings: dominant,
        },
        applicability,
        components,
        findings,
        suggestions: Vec::new(),
        provenance: Provenance::current(),
    }
}
