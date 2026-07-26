//! The evidence an attempt had already produced when it failed.

use anyhow::{ensure, Result};
use serde::Serialize;

use crate::view_read_set_campaign::channel_evidence::ChannelEvidence;
use crate::view_read_set_campaign::composition_validation::validated_composition::ValidatedComposition;
use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;
use crate::view_read_set_campaign::scale_point::ScalePoint;
use crate::view_read_set_campaign::scale_point_evidence::CHANNEL_COUNT;

/// Whatever an attempt had produced when it failed: an execution-order prefix of the channels, and
/// the composition finding if that check had already run.
///
/// Deliberately a different type from
/// [`ScalePointEvidence`](super::scale_point_evidence::ScalePointEvidence) rather than the same type
/// with optional fields: only a complete attempt can seal that one, so "analysis selects one
/// *complete* attempt per logical slot" is enforced by the types rather than by a check someone must
/// remember.
///
/// **The one state this excludes.** All four channels *and* a validated composition — because that
/// combination is not a partial attempt at all, it is exactly the evidence
/// `ScalePointEvidence::sealed` accepts, and admitting it here would let a complete attempt be
/// recorded as a failure. Everything short of it is a state a real attempt can reach:
///
/// - fewer than four channels, with or without a composition finding, depending on whether the
///   attempt died before or after that check;
/// - **all four channels with no composition finding** — the attempt finished measuring and then
///   failed during composition validation, or before its observed rows could be durably retained.
///
/// That last case is why the bound is `<= CHANNEL_COUNT` rather than `<`. The spec freezes the
/// order of the four channels but says nothing about where composition validation sits among them,
/// so a driver that happens to validate composition first would make it unreachable — and a driver
/// that does not, would not. Relying on that would be assuming an ordering contract that has not
/// been frozen.
///
/// An empty value is likewise a real state: the attempt reached a measurable configuration and died
/// before the first channel finished.
///
/// **What a retained composition means.** [`ValidatedComposition`] is unforgeable, so `Some` is a
/// finding that really was produced against retained rows, kept rather than discarded merely because
/// the attempt later failed. `None` says only that the check had not completed.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct PartialEvidence {
    scale: ScalePoint,
    channels: Vec<ChannelEvidence>,
    composition: Option<ValidatedComposition>,
}

impl PartialEvidence {
    /// Seal what a failed attempt had produced, failing loud unless its channels are an
    /// execution-order prefix and the whole is genuinely short of complete.
    pub(crate) fn sealed(
        scale: ScalePoint,
        channels: Vec<ChannelEvidence>,
        composition: Option<ValidatedComposition>,
    ) -> Result<Self> {
        ensure!(
            channels.len() <= CHANNEL_COUNT,
            "an attempt measures at most the {CHANNEL_COUNT} frozen channels, got {}",
            channels.len(),
        );
        ensure!(
            !(channels.len() == CHANNEL_COUNT && composition.is_some()),
            "all {CHANNEL_COUNT} channels together with a validated composition is a complete \
             attempt, which seals a ScalePointEvidence rather than partial evidence",
        );
        for (position, evidence) in channels.iter().enumerate() {
            let expected = MeasurementChannel::EXECUTION_ORDER.get(position).copied();
            ensure!(
                expected == Some(evidence.channel()),
                "partial evidence must be an execution-order prefix; position {position} holds \
                 {:?}",
                evidence.channel(),
            );
        }
        if let Some(composition) = composition.as_ref() {
            ensure!(
                composition.scale() == scale,
                "the retained composition finding was checked at {:?}, not the {scale:?} this \
                 attempt measured",
                composition.scale(),
            );
        }
        Ok(Self {
            scale,
            channels,
            composition,
        })
    }

    /// The scale point this attempt was measuring when it failed. Read by
    /// [`TerminalAttemptRecord::sealed`](super::terminal_attempt_record::TerminalAttemptRecord) to
    /// prove the retained evidence belongs to the identity it is filed under.
    pub(crate) fn scale(&self) -> ScalePoint {
        self.scale
    }
}
