//! The compile-time marker tagging a trusted run as the matched control.

/// A zero-sized, uninhabited type-level marker tagging a
/// [`TrustedRun`](super::trusted_run::TrustedRun) as the matched direct-base-table control. It carries
/// no value — it exists only as the type parameter `R` of `TrustedRun<ControlRun>`, so a control run
/// and an arm run are distinct types. This is what makes a
/// [`MatchedBlock`](super::matched_block::MatchedBlock) with two controls (or two arms) unrepresentable
/// rather than merely discouraged.
///
/// Its [`RunKind`](super::run_kind::RunKind) impl fixes the corresponding runtime
/// [`RunRole::Control`](crate::plan::run_role::RunRole), which the sole role-specific minting path
/// cross-checks against a coordinate's recorded role.
pub(crate) enum ControlRun {}
