//! Whether a run is the arm under test or its matched control.

/// Whether a run exercises the arm under test or its matched direct-table control.
///
/// This is a read-only label on a [`crate::plan::run::Run`]. It is deliberately not
/// combinable with a [`crate::plan::cell::Cell`] to build a run: `Run` construction
/// is private to the plan module, so no caller can form a mismatched arm/control
/// product from a `RunRole`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, clap::ValueEnum)]
pub enum RunRole {
    /// The module-view arm under test.
    Arm,
    /// The direct-base-table control run.
    Control,
}
