//! An SDK internal error reported for a seeding write is classified with the reducer's own refusal,
//! not as a transport fault.

use std::sync::mpsc;

use crate::client::measured_step_failure::MeasuredStepFailure;

use super::super::{await_reducer_completion, ReducerCompletion};

/// Coverage: the second callback-delivered failure, asserted separately because it arrives by a
/// different route — the SDK failed to run the call on this connection's behalf rather than the
/// module refusing it — and an implementation that classified only the refusal would leave this one
/// unclassified.
///
/// It lands on the same side as every other channel in this client: a callback that reported a
/// failure did reach the callback, so the transport carried it. The two are distinguished by their
/// wording alone, which is what keeps the diagnostic honest without splitting the retry rule on a
/// distinction it does not make.
#[test]
fn a_seeding_internal_error_is_an_application_failure() {
    let (tx, rx) = mpsc::channel::<ReducerCompletion>();

    match tx.send(ReducerCompletion::Internal(
        "InternalError { cause: parse failure }".to_string(),
    )) {
        Ok(()) => {}
        Err(error) => panic!("the receiver is alive at this point: {error}"),
    }
    drop(tx);

    let failure = match await_reducer_completion(rx) {
        Ok(()) => panic!("an internal error must not report a confirmed completion"),
        Err(failure) => failure,
    };
    assert!(
        matches!(failure, MeasuredStepFailure::Application(_)),
        "an internal error reported through the callback is not a transport fault"
    );
    let message = format!("{:#}", failure.into_error());
    assert!(
        message.contains("internal error awaiting reducer")
            && message.contains("InternalError { cause: parse failure }"),
        "the failure is distinguishable from a reducer refusal and keeps the SDK's own rendering; \
         got {message}"
    );
}
