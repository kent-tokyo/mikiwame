//! `mikiwame` CLI: `analyze`, `batch`, `explain`, `doctor` (AGENTS.md §14).
//!
//! Structure JSON input (used by `analyze` and `batch`) has this shape —
//! this schema is CLI-local, not part of the library's public API, so it can
//! change independently of `MaterialDiagnosticReport`'s `schema_version`:
//!
//! ```json
//! {
//!   "lattice": [[5.6402, 0.0, 0.0], [0.0, 5.6402, 0.0], [0.0, 0.0, 5.6402]],
//!   "sites": [
//!     {"element": "Na", "fractional": [0.0, 0.0, 0.0], "occupancy": 1.0}
//!   ]
//! }
//! ```
//!
//! `batch` reads one such JSON object per line (JSONL) and writes one
//! [`MaterialDiagnosticReport`] JSON object per line.

use std::env;
use std::fs;
use std::process::ExitCode;

use mikiwame::{
    AnalysisConfig, ComponentStatus, MaterialDiagnosticReport, OwnedStructure, Site, analyze,
    analyze_batch,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct StructureFile {
    lattice: [[f64; 3]; 3],
    sites: Vec<Site>,
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("analyze") => cmd_analyze(&args[1..]),
        Some("batch") => cmd_batch(&args[1..]),
        Some("explain") => cmd_explain(&args[1..]),
        Some("doctor") => {
            cmd_doctor();
            Ok(())
        }
        Some(other) => Err(format!(
            "unknown subcommand '{other}'; expected analyze, batch, explain, or doctor"
        )),
        None => Err("expected a subcommand: analyze, batch, explain, or doctor".to_string()),
    }
}

/// Returns the value following `--name` in `args`, if present.
fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn read_structure_file(path: &str) -> Result<OwnedStructure, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("reading {path}: {e}"))?;
    let file: StructureFile =
        serde_json::from_str(&text).map_err(|e| format!("parsing {path}: {e}"))?;
    Ok(OwnedStructure::new(file.lattice, file.sites))
}

fn cmd_analyze(args: &[String]) -> Result<(), String> {
    let path = args
        .first()
        .ok_or("analyze requires a structure file path")?;
    let format = flag(args, "--format").unwrap_or("json");
    let structure = read_structure_file(path)?;
    let report = analyze(&structure, &AnalysisConfig::default());
    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&report).map_err(|e| format!("{e}"))?;
            println!("{json}");
        }
        "markdown" => println!("{}", render_markdown(&report)),
        other => {
            return Err(format!(
                "unknown --format '{other}'; expected json or markdown"
            ));
        }
    }
    Ok(())
}

fn cmd_batch(args: &[String]) -> Result<(), String> {
    let path = args
        .first()
        .ok_or("batch requires a structures.jsonl path")?;
    let output = flag(args, "--output").ok_or("batch requires --output <path>")?;

    let text = fs::read_to_string(path).map_err(|e| format!("reading {path}: {e}"))?;
    let mut structures = Vec::new();
    let mut skipped = 0usize;
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<StructureFile>(line) {
            Ok(file) => structures.push(OwnedStructure::new(file.lattice, file.sites)),
            Err(e) => {
                // A malformed line is a file-format problem local to that
                // line, not a structure mikiwame could analyze — skip it
                // rather than aborting the rest of a potentially large
                // batch, same spirit as analyze_batch not letting one
                // structure's result affect another's.
                eprintln!("warning: {path} line {}: {e}, skipping", index + 1);
                skipped += 1;
            }
        }
    }
    if structures.is_empty() && skipped > 0 {
        return Err(format!(
            "no valid structures found in {path} ({skipped} line(s) skipped)"
        ));
    }

    let reports = analyze_batch(&structures, &AnalysisConfig::default());
    let mut out = String::new();
    for report in &reports {
        out.push_str(&serde_json::to_string(report).map_err(|e| format!("{e}"))?);
        out.push('\n');
    }
    fs::write(output, out).map_err(|e| format!("writing {output}: {e}"))?;
    if skipped > 0 {
        println!(
            "wrote {} report(s) to {output} ({skipped} line(s) skipped, see warnings above)",
            reports.len()
        );
    } else {
        println!("wrote {} report(s) to {output}", reports.len());
    }
    Ok(())
}

fn cmd_explain(args: &[String]) -> Result<(), String> {
    let path = args.first().ok_or("explain requires a report file path")?;
    let code = flag(args, "--finding").ok_or("explain requires --finding <CODE>")?;

    let text = fs::read_to_string(path).map_err(|e| format!("reading {path}: {e}"))?;
    let report: MaterialDiagnosticReport =
        serde_json::from_str(&text).map_err(|e| format!("parsing {path}: {e}"))?;

    let matches: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code.as_str() == code)
        .collect();
    if matches.is_empty() {
        return Err(format!("no finding with code '{code}' in {path}"));
    }
    for finding in matches {
        println!(
            "{} ({:?}, confidence {:.2}, scope {:?})",
            finding.code,
            finding.severity,
            finding.confidence.get(),
            finding.scope
        );
        println!("  {}", finding.explanation);
        for evidence in &finding.evidence {
            println!("  evidence: {evidence:?}");
        }
        for limitation in &finding.limitations {
            println!("  limitation: {limitation}");
        }
    }
    Ok(())
}

fn cmd_doctor() {
    println!("mikiwame version: {}", env!("CARGO_PKG_VERSION"));
    println!("schema version: {}", mikiwame::SCHEMA_VERSION);
    println!(
        "chematic: not used (its default branch has no periodic-structure API yet; see docs/chematic-prerequisites.md)"
    );
    println!("enabled features: none");
    println!(
        "radius table: cordero-2008-table2 embedded, but not yet used by any diagnostic \
         (SITE_SEVERE_OVERLAP / SITE_UNUSUALLY_SHORT_DISTANCE not implemented; \
         see docs/validation.md for why the table alone is unsafe for that check, and tasks/todo.md)"
    );
    println!("oxidation-state table version: none (composition diagnostics not implemented)");
    println!("configured corpus: none");
    println!("deterministic mode: true");
    println!(
        "applicable structure classes: 3D periodic crystals with explicit atom positions and a lattice, finite non-negative occupancy; primarily inorganic; small-molecule crystals/MOFs are readable but not accuracy-guaranteed"
    );
    println!("known limitations: see docs/scientific_scope.md and tasks/todo.md");
}

fn render_markdown(report: &MaterialDiagnosticReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# mikiwame report (schema v{})\n\n",
        report.schema_version
    ));
    out.push_str(&format!("**Verdict**: {:?}\n\n", report.overall.verdict));
    out.push_str(&format!(
        "**Confidence**: {:.2}\n\n",
        report.overall.confidence.get()
    ));
    if let Some(burden) = report.overall.anomaly_burden {
        out.push_str(&format!("**Anomaly burden**: {:.2}\n\n", burden.get()));
    }
    out.push_str(&format!(
        "**Applicability**: {:?}\n\n",
        report.applicability.level
    ));
    out.push_str(&format!(
        "**Input**: {} site(s), {} distinct element(s)\n\n",
        report.input.site_count, report.input.distinct_element_count
    ));

    if report.findings.is_empty() {
        out.push_str("No findings.\n\n");
    } else {
        out.push_str("## Findings\n\n");
        for finding in &report.findings {
            out.push_str(&format!(
                "- **{}** ({:?}, confidence {:.2}): {}\n",
                finding.code,
                finding.severity,
                finding.confidence.get(),
                finding.explanation
            ));
        }
        out.push('\n');
    }

    out.push_str("## Components\n\n");
    for component in &report.components {
        match &component.status {
            ComponentStatus::Ran => out.push_str(&format!("- {:?}: ran\n", component.name)),
            ComponentStatus::Skipped { reason } => {
                out.push_str(&format!("- {:?}: skipped ({reason})\n", component.name));
            }
            // ComponentStatus is #[non_exhaustive]: a future mikiwame release
            // may add a variant this CLI predates.
            _ => out.push_str(&format!("- {:?}: {:?}\n", component.name, component.status)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_finds_the_value_after_its_name() {
        let args = vec!["--format".to_string(), "markdown".to_string()];
        assert_eq!(flag(&args, "--format"), Some("markdown"));
        assert_eq!(flag(&args, "--output"), None);
    }

    #[test]
    fn flag_with_no_trailing_value_is_none() {
        let args = vec!["--format".to_string()];
        assert_eq!(flag(&args, "--format"), None);
    }

    #[test]
    fn render_markdown_includes_verdict_and_findings() {
        let lattice = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let sites = vec![Site {
            element: "Xx".to_string(),
            fractional: [0.0, 0.0, 0.0],
            occupancy: 1.0,
        }];
        let report = analyze(
            &OwnedStructure::new(lattice, sites),
            &AnalysisConfig::default(),
        );
        let markdown = render_markdown(&report);
        assert!(markdown.contains("ReviewRecommended"));
        assert!(markdown.contains("INPUT_UNKNOWN_ELEMENT"));
    }
}
