//! The shared serialization-boundary finite-float newtype: every stored `f64` that reaches the
//! authoritative JSON is proven finite at its mint boundary.

use serde::Serialize;

/// A finite `f64` — never NaN, never `±∞`. This is the single validated numeric projection type for every
/// newly stored serializable float in the report and its telemetry (residual sums, golden-section
/// brackets, `β`/`a`/`b`, milliseconds, plot coordinates, autocorrelation). It is *stronger* than the
/// workspace `F64`/NotNan primitive: NotNan still admits `±∞`, whereas non-finite floats are not valid
/// authoritative JSON, so infinities are forbidden here too.
///
/// The inner `f64` is private and every construction path asserts finiteness, so a `FiniteF64` in
/// existence is finite by construction rather than by caller discipline — a non-finite serializable
/// numeric state is unrepresentable. `#[serde(transparent)]` renders it as the bare number, so the JSON
/// shape is unchanged while the type carries the proof. Raw integer nanoseconds stay exact `u128`/`u64`
/// and never route through this lossy boundary.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(transparent)]
pub(crate) struct FiniteF64(f64);

impl FiniteF64 {
    /// Bind a finite float, failing fast on a NaN/`±∞` input. Mirrors the `beta` module's asserting mint
    /// boundaries (e.g. [`BetaCandidate::new`](crate::analysis::beta::beta_candidate::BetaCandidate)): a
    /// non-finite value is a loud panic at the boundary, not a representable serialized value.
    pub(crate) fn new(value: f64) -> Self {
        assert!(
            value.is_finite(),
            "a serialized numeric projection must be finite (no NaN/±∞), got {value}"
        );
        Self(value)
    }

    /// Bind a finite float fallibly: `Some` when finite, `None` on NaN/`±∞`. For sources whose
    /// finiteness is not guaranteed by construction, so the caller resolves the non-finite case into a
    /// typed variant rather than smuggling a sentinel through [`Self::new`].
    pub(crate) fn try_new(value: f64) -> Option<Self> {
        value.is_finite().then(|| Self(value))
    }

    /// The underlying finite value.
    pub(crate) fn get(self) -> f64 {
        self.0
    }
}
