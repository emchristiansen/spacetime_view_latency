//! The host observations bracketing one measured attempt.

use serde::Serialize;

use crate::view_read_set_campaign::environment_sample::EnvironmentSample;

/// The pair of host observations taken immediately before and immediately after a measured attempt.
///
/// A pair, not two independent optional fields: the spec requires *both* around every measured
/// attempt, so a half-observed attempt is a shape this type simply cannot hold. An attempt that
/// never ran carries no value of this type at all rather than a value with holes in it.
///
/// The post-attempt observation never invalidates evidence. It is context for reading an absolute
/// latency — which is descriptive across host regimes regardless — not a gate applied in arrears.
///
/// [`EnvironmentSample`] is reused verbatim: it is *the host-observation record*, which the spec's
/// evidence lifecycle labels **active**. Those labels are symbol-level, so its residence in the
/// campaign directory does not make reading it an extension of the dormant namespace.
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) struct HostObservations {
    before: EnvironmentSample,
    after: EnvironmentSample,
}

impl HostObservations {
    /// Bracket an attempt with its two observations.
    pub(crate) fn bracketing(before: EnvironmentSample, after: EnvironmentSample) -> Self {
        Self { before, after }
    }
}
