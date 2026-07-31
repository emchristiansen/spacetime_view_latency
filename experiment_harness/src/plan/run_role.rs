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

impl RunRole {
    /// A stable canonical token naming this role, for deterministic identity derivation and
    /// machine-readable evidence.
    ///
    /// Deliberately an explicit `&'static str` contract rather than `Debug`, `ValueEnum`, or serde
    /// output, exactly as [`Role::canonical_tag`](crate::roles::role::Role::canonical_tag) and
    /// [`Cell::canonical_tag`](crate::plan::cell::Cell::canonical_tag) are: the token reaches the
    /// artifact directory name that a fresh-server attempt's recorded evidence lives under, so a
    /// variant rename or a clap attribute must not be able to move it.
    pub fn canonical_tag(self) -> &'static str {
        match self {
            RunRole::Arm => "arm",
            RunRole::Control => "control",
        }
    }
}
