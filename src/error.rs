//! Typed errors for mikiwame.

use std::fmt;

/// Errors raised when constructing a value that carries an explicit type-level
/// invariant (a bounded score, a closed numeric range).
///
/// Malformed *structures* are not reported through this type: they become
/// findings inside a normally-returned [`crate::MaterialDiagnosticReport`] with
/// `Verdict::InvalidInput`, so that mikiwame can still explain what is wrong.
/// See `docs/architecture.md` for why `analyze` does not return `Result`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum MikiwameError {
    /// A score value was not finite or fell outside `0.0..=1.0`.
    InvalidScore {
        /// The rejected value.
        value: f64,
    },
    /// A closed range's bounds were not finite or `min` exceeded `max`.
    InvalidRange {
        /// The rejected lower bound.
        min: f64,
        /// The rejected upper bound.
        max: f64,
    },
}

impl fmt::Display for MikiwameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidScore { value } => {
                write!(f, "score {value} is not finite or not within 0.0..=1.0")
            }
            Self::InvalidRange { min, max } => {
                write!(f, "range [{min}, {max}] is not finite or min exceeds max")
            }
        }
    }
}

impl std::error::Error for MikiwameError {}
