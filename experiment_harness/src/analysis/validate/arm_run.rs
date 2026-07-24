//! The compile-time marker tagging a trusted run as the arm under test.

/// A zero-sized, uninhabited type-level marker tagging a
/// [`TrustedRun`](super::trusted_run::TrustedRun) as the module-view arm under test. It carries no
/// value — it exists only as the type parameter `R` of `TrustedRun<ArmRun>`, so an arm run and a
/// control run are distinct types. This is what makes a [`MatchedBlock`](super::matched_block::MatchedBlock)
/// with two arms (or two controls) unrepresentable rather than merely discouraged.
///
/// Its [`RunKind`](super::run_kind::RunKind) impl fixes the corresponding runtime
/// [`RunRole::Arm`](crate::plan::run_role::RunRole), which the sole role-specific minting path
/// cross-checks against a coordinate's recorded role.
pub(crate) enum ArmRun {}
