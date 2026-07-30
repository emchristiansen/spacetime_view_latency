//! The preregistered populations an attempt's caches must hold once its appends are done.

use anyhow::{ensure, Result};
use serde::Serialize;
use spacetimedb_sdk::Identity;

use crate::indexed_sender_view_calibration_pilot::calibration_params::{
    MAX_PACED_SAMPLES, OWN_ACTIVITY_ID_BASE, OWN_CONTROL_UUID, SUBSCRIBER_OWN_ROWS,
    UNRELATED_ACTIVITY_ID_BASE, UNRELATED_CONTROL_UUID, UNRELATED_IDENTITY_BYTE,
};
use crate::indexed_sender_view_calibration_pilot::calibration_rung::CalibrationRung;
use crate::indexed_sender_view_calibration_pilot::expected_population::ExpectedPopulation;

/// The two preregistered row populations an attempt must end holding, derived from the rung and the
/// number of appends that actually completed.
///
/// **A description of rows, not of counts.** Each population carries its exact key range, owning
/// identity, control, and — through the frozen arithmetic recipe — the timestamp every one of its
/// rows must show. That is what lets
/// [`VerifiedPopulation::verify`](super::verified_population::VerifiedPopulation::verify) reject a
/// same-cardinality substitution, which a count could not.
///
/// **A function of the recorded count, not of `W_max`.** The measured mutation is an append, so the
/// subscriber's own slice grows by exactly one per completed sample: an attempt that recorded 400
/// appends must end with `S₀ + 400` own rows. Expecting the ceiling would make every partial attempt
/// fail a check it actually passed — and, worse, an attempt that silently stopped appending would
/// pass if the count were never consulted at all.
///
/// That growth is the append-only estimand's defining property, recorded here rather than hidden.
/// The screen this pilot calibrates controls it by matched index; the pilot does not control it,
/// because a single-rung method-calibration run has nothing to control it against.
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) struct CalibrationExpectation {
    own: ExpectedPopulation,
    unrelated: ExpectedPopulation,
    recorded_appends: u64,
}

impl CalibrationExpectation {
    /// The expectation for an attempt at `rung`, connected as `measured`, that completed
    /// `recorded_appends` appends.
    ///
    /// The measured identity is a parameter because it is whichever identity the client connected
    /// as, which is not knowable until runtime; the unrelated owner is derived from a frozen constant
    /// so it is knowable in advance. That the two are distinct — which the whole unrelated axis rests
    /// on — is *checked* here rather than assumed, via
    /// [`ensure_measured_is_not_the_unrelated_owner`].
    ///
    /// **`recorded_appends` is checked here rather than assumed.** It is the one input the frozen
    /// constants do not bound — the freeze proves the arithmetic only up to
    /// [`MAX_PACED_SAMPLES`], so an append count above the ceiling would reach a `checked_add`
    /// whose `expect` cites a proof that does not cover it. Rejecting it at this boundary is what
    /// makes every downstream overflow proof in [`calibration_params`] apply to real values:
    /// past this point `own_rows ≤ SUBSCRIBER_OWN_ROWS + MAX_PACED_SAMPLES`, which is exactly the
    /// bound `MAX_ACTIVITY_ID` and the timestamp derivation are stated over.
    pub(crate) fn after(
        rung: CalibrationRung,
        recorded_appends: u64,
        measured: Identity,
    ) -> Result<Self> {
        ensure_measured_is_not_the_unrelated_owner(measured)?;
        ensure!(
            recorded_appends <= u64::from(MAX_PACED_SAMPLES),
            "an attempt cannot have recorded {recorded_appends} appends when the frozen ceiling is \
             {MAX_PACED_SAMPLES}; the freeze's overflow proofs are stated over that bound",
        );
        let own_rows = SUBSCRIBER_OWN_ROWS
            .checked_add(recorded_appends)
            .expect("the check above bounds the append count by the freeze's proven ceiling");
        Ok(Self {
            own: ExpectedPopulation::new(
                "own",
                OWN_ACTIVITY_ID_BASE,
                own_rows,
                OWN_CONTROL_UUID,
                measured,
            ),
            unrelated: ExpectedPopulation::new(
                "unrelated",
                UNRELATED_ACTIVITY_ID_BASE,
                rung.unrelated_rows(),
                UNRELATED_CONTROL_UUID,
                unrelated_owner(),
            ),
            recorded_appends,
        })
    }

    /// The measured identity's own population — the seeded slice plus every completed append.
    pub(crate) fn own_population(self) -> ExpectedPopulation {
        self.own
    }

    /// The non-connecting other owner's population — the unrelated backing rows at this rung.
    pub(crate) fn unrelated_population(self) -> ExpectedPopulation {
        self.unrelated
    }

    /// Appends this expectation was computed for.
    ///
    /// Read by [`CalibrationSeries::recorded`](super::calibration_series::CalibrationSeries::recorded)
    /// to refuse a complete series whose composition was verified against a different number of
    /// appends. Without that cross-check a short attempt could pass verification against its own
    /// short expectation and then be sealed as though it were complete — the sample count, the
    /// verified append count, and the frozen ceiling must all agree with one another.
    pub(crate) fn recorded_appends(self) -> u64 {
        self.recorded_appends
    }
}

/// Fail loud unless the measured identity is distinct from the unrelated population's owner.
///
/// **The precondition the axis rests on, checked rather than assumed.** The arm is a pure
/// sender-equality filter — `indexed_control_activity_sender_view` is
/// `where user_identity == ctx.sender()` — so if the connecting identity *were*
/// [`unrelated_owner`], the unrelated rows would fall inside the measured read set and there would be
/// no unrelated axis at all. SSOT §562/§564 authorize sweeping *unrelated/global* rows; that method
/// would simply be absent.
///
/// **This does not produce false evidence, and the guard is not what stops it doing so.** The arm
/// cache would hold all 2,010 rows against a 1,010-row expectation, so
/// [`VerifiedPopulation::verify`](super::verified_population::VerifiedPopulation::verify) fails
/// closed on the 1,000 unrelated-range rows: no token is minted, so no
/// [`CalibrationRecorded`](super::attempted_outcome::AttemptedOutcome) outcome and no successful
/// series can be produced. The attempt still settles as a terminal record, as every attempt must — a
/// `Semantics` failure retaining the mismatch.
///
/// What goes wrong is the *classification*. Those faults read as a sender-scope leak, the gravest
/// charge against this candidate, when the actual fault is an identity-configuration collision. The
/// attempt is spent and the ledger's account of why is wrong.
///
/// Ordinary identity derivation makes the collision vanishingly unlikely; it does not make it
/// impossible, and nothing in the type system excludes it. An `Identity` is a 32-byte value and
/// [`unrelated_owner`] is a perfectly ordinary one, so the aliased state is structurally
/// representable. Negligible probability is not a proof, and it is not the standard this module holds
/// itself to elsewhere — which is the whole reason this is a check rather than a comment.
///
/// **Phase 2 must call this at the earliest point the connected identity is known, before seeding or
/// any other side effect.** [`CalibrationExpectation::after`] also enforces it, so an aliased
/// expectation is unconstructible — but that runs *after* the batch, and by then a whole attempt
/// would have been spent measuring the wrong thing.
pub(crate) fn ensure_measured_is_not_the_unrelated_owner(measured: Identity) -> Result<()> {
    ensure!(
        measured != unrelated_owner(),
        "the measured identity {} is the frozen unrelated owner, so the unrelated population would \
         sit inside the sender-scoped read set and this attempt would not have an unrelated axis at \
         all",
        measured.to_hex(),
    );
    Ok(())
}

/// The fixed non-connecting identity that owns every unrelated row.
///
/// Derived from a frozen constant rather than connected as, so it is knowable before any client
/// connects. That it lies *outside* the measured identity's sender-scoped read set is not a property
/// of this constant alone — the view filters on `user_identity == ctx.sender()`, so it holds exactly
/// when the connecting identity differs from this one. That is checked by
/// [`ensure_measured_is_not_the_unrelated_owner`], and only once it has been checked is a row this
/// identity owns appearing in the arm cache unambiguously a leak rather than an aliasing artefact.
pub(crate) fn unrelated_owner() -> Identity {
    let mut bytes = [0u8; 32];
    bytes[31] = UNRELATED_IDENTITY_BYTE;
    Identity::from_byte_array(bytes)
}

#[cfg(test)]
mod tests;
