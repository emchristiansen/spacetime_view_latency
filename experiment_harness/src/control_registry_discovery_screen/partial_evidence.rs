//! What a failed attempt observed before it failed.

use serde::Serialize;

use crate::control_registry_discovery_screen::four_way_composition::FourWayComposition;
use crate::control_registry_discovery_screen::rejected_apply_nanos::RejectedApplyNanos;

/// The evidence a failed attempt genuinely holds, including the raw duration of a timed apply that
/// completed but was never admitted as evidence.
///
/// **Rejected samples are retained.** The spec requires raw per-sample retention, so a timed apply
/// that completed keeps its nanoseconds even when the host bracket could not be closed, a validation
/// subscription failed, the four-way composition mismatched, or the duration was nonpositive — the
/// last case retaining a literal zero.
///
/// **Retention is safe because of a visibility boundary, not a convention.** The duration is held as
/// [`RejectedApplyNanos`], a tuple struct whose field is private to *its own* module. These variants
/// are `pub(crate)` and so are their fields — Rust gives variant fields the enum's visibility and
/// offers no way to narrow them — but destructuring one yields an opaque `RejectedApplyNanos` that
/// this module cannot look inside either. There is therefore no typed path from a rejected sample to
/// [`ColdApplyEvidence::sealed`](super::cold_apply_evidence::ColdApplyEvidence::sealed), which takes
/// a bare `u128`.
///
/// [`ColdApplyEvidence`](super::cold_apply_evidence::ColdApplyEvidence) remains the sole
/// evidence-bearing duration type, sealing only from the live measurement path with a strictly
/// positive duration and an exactly matched composition.
///
/// The composition *is* readable, because a semantic failure's diagnostic content is precisely which
/// caches diverged, and a composition carries no duration to build evidence from.
///
/// **No `Debug`**, propagated from [`RejectedApplyNanos`]: a derived rendering here would recover
/// through this type exactly what that one withholds. The same omission carries up through every
/// containing type.
#[derive(Clone, Copy, Serialize)]
pub(crate) enum PartialEvidence {
    /// The attempt failed before its timed apply completed. There is no sample.
    NothingObserved,
    /// The timed apply completed; the four caches were never read. Reached when the post-measurement
    /// host observation or a validation subscription failed first.
    RejectedSample { apply_nanos: RejectedApplyNanos },
    /// The timed apply completed and all four caches were read. Retained in full, because which
    /// combination of caches diverged is the diagnostic content of a semantic failure.
    RejectedSampleAndComposition {
        apply_nanos: RejectedApplyNanos,
        composition: FourWayComposition,
    },
}

impl PartialEvidence {
    /// Whether a completed timed apply's raw duration is retained here.
    ///
    /// Answers only the yes/no question the cross-field invariants need. It deliberately does not
    /// hand back the number.
    pub(crate) fn has_sample(self) -> bool {
        match self {
            Self::NothingObserved => false,
            Self::RejectedSample { .. } | Self::RejectedSampleAndComposition { .. } => true,
        }
    }

    /// The four-way composition this attempt read, when it got that far.
    pub(crate) fn composition(self) -> Option<FourWayComposition> {
        match self {
            Self::NothingObserved | Self::RejectedSample { .. } => None,
            Self::RejectedSampleAndComposition { composition, .. } => Some(composition),
        }
    }
}
