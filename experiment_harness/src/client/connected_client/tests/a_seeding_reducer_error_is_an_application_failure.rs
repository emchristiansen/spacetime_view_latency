//! A seeding write the module refused is classified as the application failure it is, not as a fault
//! of the infrastructure it ran on.

use std::sync::mpsc;

use crate::client::measured_step_failure::MeasuredStepFailure;

use super::super::{await_reducer_completion, ReducerCompletion};

/// Coverage: the classification is the point. The campaign seeds inside an attempt whose retry rule
/// turns on this distinction — an infrastructure failure before the first sample leaves the slot
/// retryable, while a module refusal is the application's own answer and would be refused again by a
/// server that is working correctly.
///
/// The reducer's own message is preserved verbatim under the composed wording, so the refusal a
/// reader sees is the module's, not a paraphrase of it.
#[test]
fn a_seeding_reducer_error_is_an_application_failure() {
    let (tx, rx) = mpsc::channel::<ReducerCompletion>();

    match tx.send(ReducerCompletion::ReducerFailed(
        "entity_owner uuid already present".to_string(),
    )) {
        Ok(()) => {}
        Err(error) => panic!("the receiver is alive at this point: {error}"),
    }
    drop(tx);

    let failure = match await_reducer_completion(rx) {
        Ok(()) => panic!("a refused reducer must not report a confirmed completion"),
        Err(failure) => failure,
    };
    assert!(
        matches!(failure, MeasuredStepFailure::Application(_)),
        "a reducer-returned error is the module's answer, never an infrastructure fault"
    );
    let message = format!("{:#}", failure.into_error());
    assert!(
        message.contains("reducer returned an error")
            && message.contains("entity_owner uuid already present"),
        "the failure names the kind of answer and preserves the reducer's own message; got {message}"
    );
}
