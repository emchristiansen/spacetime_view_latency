//! The complete evidence of one attempt, tagged with the axis it was measured on.

use anyhow::{ensure, Result};
use serde::Serialize;

use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;
use crate::view_read_set_campaign::scale_point_evidence::ScalePointEvidence;

/// One attempt's complete evidence, in the variant of the axis it was swept on.
///
/// `AttemptOutcome::Complete` is keyed by one fresh-server
/// [`AttemptKey`], so this holds exactly that key's single scale
/// point — never a six-rung ladder. Whole-ladder completeness is the separate claim made by
/// [`UnrelatedGlobalRowsLadderEvidence`](super::unrelated_global_rows_ladder_evidence::UnrelatedGlobalRowsLadderEvidence),
/// at the analysis boundary, from six of these.
///
/// **Why an enum over an axis field.** Each axis's evidence is interpreted against its own frozen
/// ladder and endpoint factor `F`, so the axis is not a label on otherwise-uniform evidence — it
/// selects the arithmetic. A later axis arrives as its own variant together with the driver that can
/// measure it, and a reader who has not handled it fails to compile rather than silently pooling it.
///
/// **Who can construct it.** Only [`Self::for_attempt`]. The variant payload
/// [`AxisBoundEvidence`] is named `pub(crate)` because it appears in this `pub(crate)` enum's
/// interface, so the guarantee is not its own visibility: its single field is private and it has no
/// constructor, so it can only be built by the struct literal in this file. No other module —
/// sibling, parent, or elsewhere in the crate — can therefore write
/// `EvidenceArtifact::UnrelatedGlobalRows(..)`, because it has nothing to put inside. That is what
/// makes "the variant's axis always agrees with the scale point inside it" structural rather than
/// conventional; with one axis declared the claim is currently unfalsifiable in practice, and the
/// wrapper is what keeps it true when the second axis lands.
///
/// **What it does not claim.** Nothing beyond its payload: that the observations are genuine remains
/// a property of the driver's measurement path, exactly as for
/// [`ScalePointEvidence`].
#[derive(Debug, Clone, Serialize)]
pub(crate) enum EvidenceArtifact {
    /// Evidence measured while sweeping rows held by an identity other than the subscriber.
    UnrelatedGlobalRows(AxisBoundEvidence),
}

/// A scale point's evidence, bound to the artifact variant that names its axis.
///
/// Exists purely to close off direct variant construction: its field is private and it exposes no
/// constructor, so only [`EvidenceArtifact::for_attempt`] — in this file — can build one. It adds no
/// field of its own and is serialized transparently, so the ledger shape is the evidence itself.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub(crate) struct AxisBoundEvidence {
    evidence: ScalePointEvidence,
}

impl EvidenceArtifact {
    /// Tag one attempt's sealed evidence with its axis, failing loud unless the evidence was
    /// measured at exactly the scale point that attempt's identity names.
    ///
    /// Checking the whole scale point rather than only the axis is deliberate: an attempt measures
    /// one scale and one scale only, so evidence from a different rung of the same axis is just as
    /// wrong as evidence from a different axis, and both are caught here rather than at whichever
    /// later consumer happens to compare them.
    pub(crate) fn for_attempt(attempt: AttemptKey, evidence: ScalePointEvidence) -> Result<Self> {
        ensure!(
            evidence.scale() == attempt.scale(),
            "the evidence was measured at {:?} but its attempt identity names {:?}; a fresh-server \
             attempt measures exactly the scale point it is keyed by",
            evidence.scale(),
            attempt.scale(),
        );
        Ok(match attempt.scale().axis() {
            ExperimentAxis::UnrelatedGlobalRows => {
                Self::UnrelatedGlobalRows(AxisBoundEvidence { evidence })
            }
        })
    }

    /// Which axis this evidence was measured on, read off the variant rather than stored beside it.
    pub(crate) fn axis(&self) -> ExperimentAxis {
        match self {
            Self::UnrelatedGlobalRows(..) => ExperimentAxis::UnrelatedGlobalRows,
        }
    }

    /// The scale point evidence itself, whichever axis carried it.
    pub(crate) fn scale_point_evidence(&self) -> &ScalePointEvidence {
        match self {
            Self::UnrelatedGlobalRows(bound) => &bound.evidence,
        }
    }
}
