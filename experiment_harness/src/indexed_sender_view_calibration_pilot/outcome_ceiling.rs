//! The widest conclusion any record from this pilot may support.

use serde::Serialize;

/// The ceiling on what this pilot's evidence can be read as.
///
/// **One variant, written from a frozen constant onto every record.** The spec's decision authorizes
/// this pilot to produce evidence for freezing `W` and *nothing else*: no performance conclusion, no
/// scaling conclusion, no candidate outcome, no site disposition, no Arm/Control ratio. Carrying that
/// as a single-variant enum means no attempt can report having run under a wider ceiling, and a
/// later reader holding only the ledger sees the limit stated on the line rather than having to know
/// it from the spec.
///
/// **What this does and does not enforce.** A single-variant enum excludes nothing at runtime:
/// adding a wider variant and writing it in [`MethodFacts::frozen`](super::method_facts::MethodFacts)
/// is a two-line diff inside this module, as quiet as editing a comment. What the type genuinely buys
/// is that the ceiling is *carried onto every serialized ledger line* — `MethodFacts` has private
/// fields and one constructor, and all five record shapes hold one — so a ledger-only reader sees the
/// limit on the line, and widening it is an edit visible in review. A review tripwire, not a grant
/// requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum OutcomeCeiling {
    /// Evidence for choosing a within-cell sample count, and nothing further.
    MethodCalibrationOnly,
}

/// The ceiling every record of this pilot carries.
pub(crate) const CALIBRATION_ONLY: OutcomeCeiling = OutcomeCeiling::MethodCalibrationOnly;
