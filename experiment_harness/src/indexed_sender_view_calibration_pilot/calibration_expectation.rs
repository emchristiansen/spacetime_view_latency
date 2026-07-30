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
    /// as, which is not knowable until runtime; the unrelated owner is derived from a frozen
    /// constant so it is knowable in advance and no client ever authenticates as it.
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

/// The fixed non-connecting identity that owns every unrelated row.
///
/// Derived from a frozen constant rather than connected as, so the unrelated population is provably
/// outside the measured identity's sender-scoped read set: no client ever authenticates as it, so a
/// row it owns appearing in the arm cache is unambiguously a leak rather than a seeding mistake.
pub(crate) fn unrelated_owner() -> Identity {
    let mut bytes = [0u8; 32];
    bytes[31] = UNRELATED_IDENTITY_BYTE;
    Identity::from_byte_array(bytes)
}
