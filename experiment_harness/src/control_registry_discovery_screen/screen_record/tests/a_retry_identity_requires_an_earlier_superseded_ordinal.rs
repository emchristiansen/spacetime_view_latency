//! The record model represents a retry identity, and refuses a link that contradicts the ordinal.

use crate::control_registry_discovery_screen::attempt_key::AttemptKey;
use crate::control_registry_discovery_screen::candidate_version::CONTROL_REGISTRY_DISCOVERY_VERSION;
use crate::control_registry_discovery_screen::experiment_axis::ExperimentAxis;
use crate::control_registry_discovery_screen::not_run_reason::NotRunReason;
use crate::control_registry_discovery_screen::retry_ordinal::RetryOrdinal;
use crate::control_registry_discovery_screen::screen_record::ScreenRecord;
use crate::control_registry_discovery_screen::stage_repetition::StageRepetition;

use super::fixture::{attempt_key, pinned, seed};

/// Coverage: originals and retries are each refused the other's supersession shape.
///
/// **This screen mints no retry.** It executes exactly the sixteen frozen original identities and
/// schedules nothing on `RetryEligibility::Retryable`, which is a prospective authorization for a
/// separately frozen retry inventory rather than an instruction for this run to loop. The model must
/// nonetheless be *able* to express a retry — the spec requires supersession links, and a model that
/// silently accepted an original carrying one, or a retry carrying none, would record a relationship
/// nobody could trust.
///
/// The earlier version of these constructors hard-coded the link to `None`, which made every retry
/// identity unrecordable; that is what this pins against returning.
#[test]
fn a_retry_identity_requires_an_earlier_superseded_ordinal() {
    let original = attempt_key();
    let pinned = pinned();
    let seed = seed();

    // An original is exactly what this screen writes, and it supersedes nothing.
    ScreenRecord::not_run(
        original,
        &pinned,
        seed,
        None,
        NotRunReason::EnvironmentRefused,
    )
    .expect("an original with no supersession link is the case this screen always takes");

    assert!(
        ScreenRecord::not_run(
            original,
            &pinned,
            seed,
            Some(RetryOrdinal::ORIGINAL),
            NotRunReason::EnvironmentRefused,
        )
        .is_err(),
        "an original supersedes nothing, so naming a link must be refused"
    );

    // A retry identity at the same logical slot: every component equal but the ordinal.
    let retry = AttemptKey::new(
        original.candidate(),
        ExperimentAxis::UnrelatedGlobalRows,
        original.rung(),
        original.role(),
        StageRepetition::Screen(original.stage_block()),
        RetryOrdinal::ORIGINAL.next(),
        CONTROL_REGISTRY_DISCOVERY_VERSION,
    );
    assert!(
        retry.same_logical_slot(original),
        "the retry must address the same logical slot as the original it replaces"
    );

    ScreenRecord::not_run(
        retry,
        &pinned,
        seed,
        Some(RetryOrdinal::ORIGINAL),
        NotRunReason::EnvironmentRefused,
    )
    .expect("a retry naming the earlier ordinal it supersedes is representable");

    assert!(
        ScreenRecord::not_run(retry, &pinned, seed, None, NotRunReason::EnvironmentRefused)
            .is_err(),
        "a retry must name the attempt it supersedes"
    );

    assert!(
        ScreenRecord::not_run(
            retry,
            &pinned,
            seed,
            Some(RetryOrdinal::ORIGINAL.next()),
            NotRunReason::EnvironmentRefused,
        )
        .is_err(),
        "a retry may only supersede an attempt earlier than itself, never itself"
    );
}
