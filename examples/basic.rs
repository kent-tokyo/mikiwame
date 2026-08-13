//! Basic usage: analyze a clean structure and a broken one.
//!
//! Run with `cargo run --example basic`. Its output is copied verbatim into
//! README.md's usage section (AGENTS.md §23: README examples must match real
//! output).

use mikiwame::{AnalysisConfig, OwnedStructure, Site, analyze};

fn nacl(duplicate_second_site: bool) -> OwnedStructure {
    let a = 5.6402;
    let lattice = [[a, 0.0, 0.0], [0.0, a, 0.0], [0.0, 0.0, a]];
    let na = |f: [f64; 3]| Site {
        element: "Na".to_string(),
        fractional: f,
        occupancy: 1.0,
    };
    let cl = |f: [f64; 3]| Site {
        element: "Cl".to_string(),
        fractional: f,
        occupancy: 1.0,
    };
    let second_na = if duplicate_second_site {
        [0.0, 0.0, 0.0] // moved onto the first Na
    } else {
        [0.5, 0.5, 0.0]
    };
    let sites = vec![
        na([0.0, 0.0, 0.0]),
        na(second_na),
        na([0.5, 0.0, 0.5]),
        na([0.0, 0.5, 0.5]),
        cl([0.5, 0.5, 0.5]),
        cl([0.0, 0.0, 0.5]),
        cl([0.0, 0.5, 0.0]),
        cl([0.5, 0.0, 0.0]),
    ];
    OwnedStructure::new(lattice, sites)
}

fn main() {
    let config = AnalysisConfig::default();

    let report = analyze(&nacl(false), &config);
    println!("clean NaCl: {:?}", report.overall.verdict);

    let report = analyze(&nacl(true), &config);
    println!("duplicated-site NaCl: {:?}", report.overall.verdict);
    for finding in &report.findings {
        println!(
            "  {:?} {}: {}",
            finding.severity, finding.code, finding.explanation
        );
    }
}
