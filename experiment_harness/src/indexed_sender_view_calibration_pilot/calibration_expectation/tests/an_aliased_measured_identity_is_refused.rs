//! The measured identity may not be the frozen unrelated owner.

use spacetimedb_sdk::Identity;

use crate::indexed_sender_view_calibration_pilot::calibration_expectation::{
    ensure_measured_is_not_the_unrelated_owner, unrelated_owner, CalibrationExpectation,
};
use crate::indexed_sender_view_calibration_pilot::calibration_rung::CalibrationRung;

/// Coverage: the precondition the entire unrelated axis rests on.
///
/// The arm is a pure sender-equality filter — `indexed_control_activity_sender_view` is
/// `where user_identity == ctx.sender()`. So if the connecting identity were the unrelated owner, the
/// unrelated rows would fall *inside* the measured read set and the unrelated axis SSOT §562/§564
/// authorize would simply be absent.
///
/// The consequence is not false evidence. The arm would hold 2,010 rows against a 1,010-row
/// expectation, so `VerifiedPopulation::verify` fails closed on the 1,000 unrelated-range rows and
/// records nothing. The harm is that it fails *for the wrong stated reason*: those faults read as a
/// sender-scope leak — the gravest charge against this candidate — when the real fault is an identity
/// collision. This guard makes the attempt refuse before it is spent, and refuse with the truth.
///
/// Nothing in the type system excludes the alias — an `Identity` is 32 bytes and `unrelated_owner` is
/// an ordinary one — so the state is structurally representable however unlikely ordinary derivation
/// makes it. Negligible probability is not a proof, which is why this is checked rather than
/// asserted in prose.
///
/// Both entry points are exercised: the standalone guard Phase 2 must call before seeding, and
/// `after`, so an aliased expectation is unconstructible even if that call were ever skipped.
#[test]
fn an_aliased_measured_identity_is_refused() {
    let aliased = unrelated_owner();

    let error = ensure_measured_is_not_the_unrelated_owner(aliased)
        .expect_err("the measured identity must not be the unrelated owner");
    let rendered = format!("{error:#}");
    assert!(
        rendered.contains("unrelated axis"),
        "the refusal must say what is lost, not merely that two values matched, got {rendered:?}"
    );

    CalibrationExpectation::after(CalibrationRung::Baseline, 0, aliased)
        .expect_err("an aliased expectation must not be constructible");

    // A distinct identity is accepted, so the guard rejects the collision and not every identity.
    let distinct = Identity::from_byte_array([0u8; 32]);
    assert_ne!(distinct, aliased, "the fixture identities must differ");
    ensure_measured_is_not_the_unrelated_owner(distinct)
        .expect("a distinct measured identity is admissible");
    CalibrationExpectation::after(CalibrationRung::Baseline, 0, distinct)
        .expect("a distinct measured identity yields an expectation");
}
