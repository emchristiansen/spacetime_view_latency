//! The compile-time run-role marker trait: lifts [`RunRole`] into a type parameter.

use crate::analysis::validate::arm_run::ArmRun;
use crate::analysis::validate::control_run::ControlRun;
use crate::plan::run_role::RunRole;

/// Lifts the runtime [`RunRole`] into a type parameter so a
/// [`TrustedRun<R>`](super::trusted_run::TrustedRun) carries its role *in the type*. Implemented only by
/// the two zero-sized markers [`ArmRun`] and [`ControlRun`]; the `sealed::Sealed` supertrait closes the
/// set so no other type can ever be a `RunKind`, keeping the arm/control distinction total.
///
/// [`Self::ROLE`] is the runtime role each marker stands for; the sole role-specific minting path
/// cross-checks it against a coordinate's recorded [`RunRole`] before a `TrustedRun<R>` is assembled, so
/// the type-level role can never disagree with the recorded one.
pub(crate) trait RunKind: sealed::Sealed {
    /// The runtime role this marker corresponds to.
    const ROLE: RunRole;
}

impl RunKind for ArmRun {
    const ROLE: RunRole = RunRole::Arm;
}

impl RunKind for ControlRun {
    const ROLE: RunRole = RunRole::Control;
}

mod sealed {
    use crate::analysis::validate::arm_run::ArmRun;
    use crate::analysis::validate::control_run::ControlRun;

    /// Private supertrait implemented only for the two markers in this module, so [`RunKind`](super::RunKind)
    /// cannot be implemented anywhere else in the crate.
    pub(crate) trait Sealed {}

    impl Sealed for ArmRun {}

    impl Sealed for ControlRun {}
}
