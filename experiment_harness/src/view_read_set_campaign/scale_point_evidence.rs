//! One fresh server's complete evidence at the single scale point it measured.
//!
//! The type lives in the private, *childless* inline module [`sealed`] because everything sealing
//! proves — four channels in the frozen execution order, a composition finding from *this* scale
//! point, and delivery evidence from a measured window that actually closed — is carried by the
//! constructor rather than by the field types. A private field is visible
//! to its declaring module **and every descendant**, so a `#[cfg(test)] mod tests` child, or any
//! child added later, could write the struct literal and attach another rung's composition finding to
//! this evidence. `sealed` has no children, so [`ScalePointEvidence::sealed`] really is the only
//! door.
//!
//! [`CHANNEL_COUNT`] stays outside `sealed`: it is a derived `usize` constant read by
//! [`partial_evidence`](crate::view_read_set_campaign::partial_evidence), and a constant carries no
//! invariant that sole minting could protect.

use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;

/// How many channels a complete attempt measures — the length of the frozen execution order, read
/// from it rather than written as a literal four, so the two cannot disagree.
pub(crate) const CHANNEL_COUNT: usize = MeasurementChannel::EXECUTION_ORDER.len();

mod sealed {
    use anyhow::{anyhow, ensure, Result};
    use serde::Serialize;

    use crate::view_read_set_campaign::cell_statistic::CellStatistic;
    use crate::view_read_set_campaign::channel_evidence::ChannelEvidence;
    use crate::view_read_set_campaign::composition_validation::validated_composition::ValidatedComposition;
    use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;
    use crate::view_read_set_campaign::scale_point::ScalePoint;
    use crate::view_read_set_campaign::scale_point_evidence::CHANNEL_COUNT;
    use crate::view_read_set_campaign::subscriber_delivery::subscriber_delivery_evidence::SubscriberDeliveryEvidence;

    /// All four channels measured at one scale point, the composition check that scale point
    /// passed, and what its subscriber's connection received while measuring.
    ///
    /// This is what one fresh server yields. Every scale point is separately provisioned, so an
    /// attempt's evidence is exactly this — never a ladder. Whole-ladder completeness is a *separate*
    /// claim: it is a fact about six *selected* attempts of one `(candidate, block, role, axis,
    /// version)`, so it can only be established downstream of
    /// `ReconciledCampaign`,
    /// where
    /// `UnrelatedGlobalRowsLadderEvidence`
    /// makes it.
    ///
    /// **Who can construct it.** Only [`Self::sealed`], and only with four well-formed,
    /// channel-specific [`ChannelEvidence`] values, a [`ValidatedComposition`], and a
    /// [`SubscriberDeliveryEvidence`]. Every field is private to this childless module, so no struct
    /// literal anywhere in the crate can stand in for that call. The channel values assert only that
    /// each reduction is paired with its own sample shape; the composition is unforgeable (private
    /// fields, sole constructor confined the same way), and the delivery evidence is unforgeable in
    /// a stronger sense still, so this constructor is reachable only from a real measurement path.
    ///
    /// **What sealing proves.** That all four channels of
    /// [`MeasurementChannel::EXECUTION_ORDER`] are present, exactly once each, in that frozen order —
    /// structurally, by conversion into a fixed-size array after a positional check, mirroring
    /// [`EvidenceArtifact::sealed`](crate::entity_owner_pilot::evidence_artifact::EvidenceArtifact).
    /// That the composition was checked at *this* scale point, not another: a finding from a
    /// different rung cannot be attached to this evidence. And that a measured window closed, since
    /// delivery evidence exists only for one that did.
    ///
    /// **What it does not prove.** That the measurements are genuine. Each [`ChannelEvidence`] binds
    /// a reduction to its own sample shape and nothing more; genuineness is a property of the
    /// driver's measurement path. Sealing adds channel completeness and scale agreement to that, and
    /// no more.
    #[derive(Debug, Clone, Serialize)]
    pub(crate) struct ScalePointEvidence {
        scale: ScalePoint,
        channels: [ChannelEvidence; CHANNEL_COUNT],
        composition: ValidatedComposition,
        delivery: SubscriberDeliveryEvidence,
    }

    impl ScalePointEvidence {
        /// Seal one attempt's evidence, failing loud unless its channels are exactly the frozen
        /// execution order and its composition finding belongs to the same scale point.
        ///
        /// **Why delivery evidence is required rather than optional.** A complete attempt is one
        /// whose measured window opened and closed, so it always has delivery facts; an attempt that
        /// failed has no defined close and carries none, which is why
        /// [`PartialEvidence`](crate::view_read_set_campaign::partial_evidence::PartialEvidence)
        /// takes no such argument. Requiring it here is what rules out a complete attempt recorded
        /// with default, reconstructed, or partially observed delivery — at the cost that the
        /// channel and scale checks below are now proved by inspection rather than by a pure test.
        pub(crate) fn sealed(
            scale: ScalePoint,
            channels: Vec<ChannelEvidence>,
            composition: ValidatedComposition,
            delivery: SubscriberDeliveryEvidence,
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
                    "a complete attempt must measure exactly {CHANNEL_COUNT} channels, got \
                     {collected}"
                )
            })?;
            Ok(Self {
                scale,
                channels,
                composition,
                delivery,
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
        /// Total: sealing proved every channel of the frozen order is present, so there is no
        /// missing case for a caller to handle wrongly.
        pub(crate) fn statistic(&self, channel: MeasurementChannel) -> CellStatistic {
            self.channels
                .iter()
                .find(|evidence| evidence.channel() == channel)
                .expect("sealing proved every channel of the frozen execution order is present")
                .statistic()
        }

        /// The composition check this scale point passed — where a reader goes to re-run the
        /// comparison against the retained rows.
        pub(crate) fn composition(&self) -> &ValidatedComposition {
            &self.composition
        }

        /// What the measured subscriber's connection received while this evidence was produced.
        ///
        /// Supporting evidence, recorded beside the composition finding rather than inside it: it
        /// describes the connection over the measured window, and unlike the finding it cannot be
        /// replayed from retained artifacts.
        pub(crate) fn delivery(&self) -> &SubscriberDeliveryEvidence {
            &self.delivery
        }
    }
}

pub(crate) use sealed::ScalePointEvidence;
