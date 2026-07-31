//! The untrusted wire form of an attempt's run role.

use serde::Deserialize;

/// The wire form of [`RunRole`](crate::plan::run_role::RunRole).
///
/// **Both variants are mirrored, and only `Arm` is admitted.** This is the one key component whose
/// source type genuinely has two values, so it is also the one where an unvalidated field would be
/// most consequential: admitting a `Control` line would let this analyzer's per-`W` output be read as
/// an Arm-versus-Control comparison, which §568 forbids the pilot to produce at all. The pilot cannot
/// mint a Control identity — `AttemptKey::calibration` always writes `Arm` — but that is a fact about
/// the writer, and this analyzer reads untrusted text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum RunRoleDto {
    /// The module-view arm under test — the only role §567's freeze contains.
    Arm,
    /// The direct-base-table control run. Decodable so a Control line is refused *by name* rather
    /// than as a malformed line, and never admitted.
    Control,
}
