//! Analysis configuration.

/// Configuration for [`crate::analyze`] and [`crate::analyze_batch`].
///
/// Empty in v0.1: no diagnostic component has a tunable threshold yet (every
/// check that ships needs none — see `docs/scientific_scope.md`). Fields will
/// be added as thresholded checks land; `#[non_exhaustive]` means existing
/// callers using `AnalysisConfig::default()` won't break when that happens.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct AnalysisConfig {}
