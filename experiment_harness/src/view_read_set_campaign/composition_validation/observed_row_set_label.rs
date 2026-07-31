//! Which of an attempt's two observations a retained row set is.

use serde::Serialize;

use crate::view_read_set_campaign::campaign_params::{
    AFTER_SATURATED_ROW_SET_STEM, SEEDED_ROW_SET_STEM,
};

/// Which phase of the measured schedule a retained
/// [`ObservedRowSet`](super::observed_row_set::ObservedRowSet) was taken at, and therefore what it
/// is named on disk.
///
/// **Why a closed enum and not a `&str`.** The label becomes a path component under the attempt's
/// artifact directory. A free string would make `..`, `/`, and the empty name representable, so an
/// artifact could be written outside the directory it claims to belong to — and validating a string
/// at the persistence boundary would only reject those spellings, leaving every *other* misspelling
/// (a typo, a label reused across phases, a label naming a phase the transition does not describe)
/// silently acceptable. There are exactly two observations in a composition transition, so naming
/// them is a closed choice, and both projections are frozen constants: path traversal and label
/// drift are unrepresentable rather than rejected.
///
/// Deliberately parallel to
/// [`ExpectedPayloadState`](super::expected_payload_state::ExpectedPayloadState), which names the
/// same two phases for the *expectation* side. They stay separate types because this one answers
/// "where is the artifact", not "what must it contain": the after-phase expectation carries the
/// schedule whose payloads it requires, which a filename cannot and should not.
///
/// Freely constructible, and harmlessly so: naming which observation is being written asserts
/// nothing about what it holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum ObservedRowSetLabel {
    /// The observation taken after E4 and before E2's first measured write, where the spec's
    /// "before E2, every seeded row has the seeded payload" holds.
    SeededBeforeMeasurement,
    /// The observation taken once E1's batch has confirmed — the attempt's final state.
    AfterSaturatedBatch,
}

impl ObservedRowSetLabel {
    /// The frozen file stem this observation is retained under, as a single filename component.
    ///
    /// Total over both variants and drawn from
    /// [`campaign_params`](crate::view_read_set_campaign::campaign_params), so the recorded artifact
    /// paths are part of the preregistration rather than incidental strings at a call site.
    pub(crate) fn file_stem(self) -> &'static str {
        match self {
            Self::SeededBeforeMeasurement => SEEDED_ROW_SET_STEM,
            Self::AfterSaturatedBatch => AFTER_SATURATED_ROW_SET_STEM,
        }
    }
}
