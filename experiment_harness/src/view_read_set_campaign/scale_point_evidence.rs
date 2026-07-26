//! One fresh server's complete evidence at the single scale point it measured.

use anyhow::{anyhow, ensure, Result};
use serde::Serialize;

use crate::view_read_set_campaign::cell_statistic::CellStatistic;
use crate::view_read_set_campaign::channel_evidence::ChannelEvidence;
use crate::view_read_set_campaign::composition_validation::validated_composition::ValidatedComposition;
use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;
use crate::view_read_set_campaign::scale_point::ScalePoint;

/// How many channels a complete attempt measures — the length of the frozen execution order, read
/// from it rather than written as a literal four, so the two cannot disagree.
pub(crate) const CHANNEL_COUNT: usize = MeasurementChannel::EXECUTION_ORDER.len();

/// All four channels measured at one scale point, plus the composition check that scale point
/// passed.
///
/// This is what one fresh server yields. Every scale point is separately provisioned, so an
/// attempt's evidence is exactly this — never a ladder. Whole-ladder completeness is a *separate*
/// claim, enforced at the analysis boundary by
/// [`UnrelatedGlobalRowsLadderEvidence`](super::unrelated_global_rows_ladder_evidence::UnrelatedGlobalRowsLadderEvidence).
///
/// **Who can construct it.** Any code in the crate, through [`Self::sealed`] — but only by supplying
/// four well-formed, channel-specific [`ChannelEvidence`] values and a [`ValidatedComposition`]. The
/// channel values assert only that each reduction is paired with its own sample shape; the latter is
/// unforgeable
/// (private fields, sole constructor colocated with its comparison), so a sealed
/// `ScalePointEvidence` cannot exist without a composition check having actually run against
/// retained rows.
///
/// **What sealing proves.** That all four channels of
/// [`MeasurementChannel::EXECUTION_ORDER`] are present, exactly once each, in that frozen order —
/// structurally, by conversion into a fixed-size array after a positional check, mirroring
/// [`EvidenceArtifact::sealed`](crate::entity_owner_pilot::evidence_artifact::EvidenceArtifact).
/// And that the composition was checked at *this* scale point, not another: a finding from a
/// different rung cannot be attached to this evidence.
///
/// **What it does not prove.** That the measurements are genuine. Each [`ChannelEvidence`] binds a
/// reduction to its own sample shape and nothing more; genuineness is a property of the driver's
/// measurement path. Sealing adds channel completeness and scale agreement to that, and no more.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ScalePointEvidence {
    scale: ScalePoint,
    channels: [ChannelEvidence; CHANNEL_COUNT],
    composition: ValidatedComposition,
}

impl ScalePointEvidence {
    /// Seal one attempt's evidence, failing loud unless its channels are exactly the frozen
    /// execution order and its composition finding belongs to the same scale point.
    pub(crate) fn sealed(
        scale: ScalePoint,
        channels: Vec<ChannelEvidence>,
        composition: ValidatedComposition,
    ) -> Result<Self> {
        ensure!(
            composition.scale() == scale,
            "the composition finding was checked at {:?}, not the {:?} this evidence claims; a \
             finding from another scale point can never stand in for this one",
            composition.scale(),
            scale,
        );
        let collected = channels.len();
        for (position, evidence) in channels.iter().enumerate() {
            let expected = MeasurementChannel::EXECUTION_ORDER.get(position).copied();
            ensure!(
                expected == Some(evidence.channel()),
                "a complete attempt's channels must be the frozen execution order \
                 {:?}; position {position} holds {:?}",
                MeasurementChannel::EXECUTION_ORDER,
                evidence.channel(),
            );
        }
        let channels: [ChannelEvidence; CHANNEL_COUNT] = channels.try_into().map_err(|_| {
            anyhow!(
                "a complete attempt must measure exactly {CHANNEL_COUNT} channels, got {collected}"
            )
        })?;
        Ok(Self {
            scale,
            channels,
            composition,
        })
    }

    /// The scale point this whole attempt held fixed while it measured.
    pub(crate) fn scale(&self) -> ScalePoint {
        self.scale
    }

    /// Every channel's evidence, in the frozen execution order.
    pub(crate) fn channels(&self) -> &[ChannelEvidence; CHANNEL_COUNT] {
        &self.channels
    }

    /// This scale point's `S` for `channel` — one of the two values an endpoint factor
    /// `T = S_last / S_first` is built from.
    ///
    /// Total: sealing proved every channel of the frozen order is present, so there is no missing
    /// case for a caller to handle wrongly.
    pub(crate) fn statistic(&self, channel: MeasurementChannel) -> CellStatistic {
        self.channels
            .iter()
            .find(|evidence| evidence.channel() == channel)
            .expect("sealing proved every channel of the frozen execution order is present")
            .statistic()
    }

    /// The composition check this scale point passed — where a reader goes to re-run the comparison
    /// against the retained rows.
    pub(crate) fn composition(&self) -> &ValidatedComposition {
        &self.composition
    }
}
