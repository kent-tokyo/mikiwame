//! Diagnostic components. See `docs/architecture.md` for the pipeline order
//! and which checks are fatal (short-circuit later components).

pub(crate) mod coordination;
pub(crate) mod disorder;
pub(crate) mod input_quality;
pub(crate) mod separation;
