//! What a failed attempt observed before it failed.

use serde::Serialize;

use crate::control_registry_discovery_screen::four_way_composition::FourWayComposition;

/// The evidence a failed attempt genuinely holds.
///
/// The Pilot's partial evidence is a strict prefix of a ladder it walks inside one attempt. This
/// screen walks no ladder — at `sample_count = 1` an attempt either sealed its one cold apply or it
/// did not — so the narrow analogue is what was observed of the *composition* before the failure.
///
/// Deliberately unable to hold a duration. A cold apply becomes evidence only by sealing as
/// [`ColdApplyEvidence`](super::cold_apply_evidence::ColdApplyEvidence), which rejects a nonpositive
/// statistic and an unmatched composition; letting partial evidence carry a raw duration would
/// create a second, unvalidated path by which a rejected sample could re-enter analysis.
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) enum PartialEvidence {
    /// The attempt failed before reading the four caches.
    NothingObserved,
    /// The four caches were read and did not all match. Retained in full, because which combination
    /// of caches diverged is the diagnostic content of a semantic failure.
    ObservedComposition { composition: FourWayComposition },
}
