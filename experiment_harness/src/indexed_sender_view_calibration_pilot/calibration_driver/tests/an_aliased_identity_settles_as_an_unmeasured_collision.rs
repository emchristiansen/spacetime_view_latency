//! An aliased measured identity ends the attempt as an unmeasured collision, and a distinct one
//! proceeds.

use spacetimedb_sdk::Identity;

use crate::indexed_sender_view_calibration_pilot::attempt_stage::AttemptStage;
use crate::indexed_sender_view_calibration_pilot::calibration_expectation::unrelated_owner;
use crate::indexed_sender_view_calibration_pilot::failure_kind::FailureKind;
use crate::indexed_sender_view_calibration_pilot::retry_eligibility::RetryEligibility;
use crate::indexed_sender_view_calibration_pilot::sampling_progress::SamplingProgress;

use super::super::{guard_unrelated_axis, MeasuredSettlement};

/// Coverage: the precondition the whole unrelated axis rests on, from the guard through to the
/// terminal record shape it produces.
///
/// The measured identity is issued by the server at the handshake, so an alias with the frozen
/// unrelated owner is structurally representable however unlikely ordinary derivation makes it. Left
/// unchecked the arm would hold both populations against a one-population expectation, verification
/// would fail closed on the unrelated-range rows, and the attempt would settle as a `Semantics`
/// failure — filing a sender-scope leak, the gravest charge against this candidate, where the real
/// fault is an identity collision.
///
/// **Pure in the identity, which is the point.** `guard_unrelated_axis` takes a 32-byte value and no
/// client, so the classification is provable without provisioning a server. A guard against a
/// vanishingly rare configuration is checked here or nowhere.
///
/// The four derived facts are asserted rather than the kind alone, because the kind is only half the
/// claim — a truthful kind filed into a shape that promised host observations would be just as
/// wrong:
///
/// - **`Unmeasured`**, so the record shape carries provenance and no bracket. The connection existed;
///   the measurement window never opened.
/// - **`BeforeFirstSample`**, so nothing claims a sample had begun.
/// - **`NotRetryable`**. Half the infrastructure case is provable — nothing here reuses a credential
///   — but that a *fresh* attempt would establish a *different* identity is a property of the pinned
///   server's anonymous-identity minting, whose source this repository does not carry. Retryability
///   asserts a later attempt would prospectively differ, so it is not claimed.
/// - **No retained samples.** `NothingObserved` is the only evidence variant whose `retained_samples`
///   is `None`, so this single assertion establishes both that no series was invented and that no
///   composition mismatch is carried — the constructor admits a mismatch only alongside a series.
#[test]
fn an_aliased_identity_settles_as_an_unmeasured_collision() {
    let aliased = unrelated_owner();

    let settlement = guard_unrelated_axis(aliased)
        .expect("an aliased identity is a recordable outcome, never a harness bug")
        .expect("an aliased identity must not be allowed to proceed");

    let failure = match settlement {
        MeasuredSettlement::NotMeasured { failure } => failure,
        // Asserted by match rather than by a discriminant accessor: `MeasuredSettlement` has no
        // `Debug`, inherited from the retained samples its bracketed variant can hold.
        MeasuredSettlement::Unbracketed { .. } | MeasuredSettlement::Bracketed { .. } => panic!(
            "a collision refused before the first append has no host observation, so it cannot \
             inhabit a bracketed shape"
        ),
    };

    assert_eq!(
        failure.kind(),
        FailureKind::IdentityCollision,
        "the refusal must name the collision rather than a connection or a reducer that did not fail"
    );
    assert_eq!(
        failure.stage(),
        AttemptStage::Unmeasured,
        "the connection existed and the measurement window never opened"
    );
    assert_eq!(
        failure.progress(),
        SamplingProgress::BeforeFirstSample,
        "the guard fires before any append is issued"
    );
    assert_eq!(
        failure.retry_eligibility(),
        RetryEligibility::NotRetryable,
        "a fresh attempt establishing a different identity is not proven from any source this \
         repository carries, so no prospective retry is claimed"
    );
    assert_eq!(
        failure.retained_samples(),
        None,
        "nothing was measured, so no series exists — and NothingObserved is the only evidence that \
         reports None, which is also what excludes a carried composition mismatch"
    );

    // A distinct identity proceeds, so the guard rejects the collision and not every identity.
    let distinct = Identity::from_byte_array([0u8; 32]);
    assert_ne!(distinct, aliased, "the fixture identities must differ");
    assert!(
        guard_unrelated_axis(distinct)
            .expect("a distinct identity is not a harness bug")
            .is_none(),
        "a distinct measured identity has an unrelated axis and must not be refused"
    );
}
