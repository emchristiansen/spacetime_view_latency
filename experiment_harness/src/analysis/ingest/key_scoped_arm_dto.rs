//! Untrusted mirror of [`KeyScopedArm`](crate::plan::key_scoped_arm::KeyScopedArm).

use serde::Deserialize;

/// The wire form of a key-scoped arm (F/F′), the externally-tagged unit variant string. An unknown
/// arm string is rejected by serde as it deserializes, closing the tag set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub(crate) enum KeyScopedArmDto {
    PointFilter,
    PointSemijoin,
}
