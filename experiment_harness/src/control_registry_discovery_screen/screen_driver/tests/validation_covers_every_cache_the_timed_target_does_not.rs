//! Whichever target is timed, the untimed three are exactly the rest — no cache unsubscribed, none
//! twice.

use std::collections::BTreeSet;

use crate::control_registry_discovery_screen::screen_target::ScreenTarget;

use super::super::{validation_targets, VALIDATION_TARGET_COUNT};

/// Which three targets each timed target leaves to validate, **written out** rather than computed.
///
/// An independent oracle is the whole point, and the same standard the failure mappings are held to.
/// Deriving the expectation as `ALL.filter(|t| *t != timed)` would be re-running the function under
/// test: a change of mind about what "the others" means — or a shared off-by-one — would agree with
/// itself and pass. Sixteen entries transcribed by hand cannot.
///
/// Verified total against [`ScreenTarget::ALL`] below, so a target cannot be dropped from the oracle
/// either.
const FROZEN_VALIDATION_TARGETS: [(ScreenTarget, [ScreenTarget; VALIDATION_TARGET_COUNT]); 4] = [
    (
        ScreenTarget::ArmA,
        [
            ScreenTarget::ControlA,
            ScreenTarget::ArmB,
            ScreenTarget::ControlB,
        ],
    ),
    (
        ScreenTarget::ControlA,
        [
            ScreenTarget::ArmA,
            ScreenTarget::ArmB,
            ScreenTarget::ControlB,
        ],
    ),
    (
        ScreenTarget::ArmB,
        [
            ScreenTarget::ArmA,
            ScreenTarget::ControlA,
            ScreenTarget::ControlB,
        ],
    ),
    (
        ScreenTarget::ControlB,
        [
            ScreenTarget::ArmA,
            ScreenTarget::ControlA,
            ScreenTarget::ArmB,
        ],
    ),
];

/// Coverage: the complement property the four-way composition rests on, for all four timed targets.
///
/// **What this half proves, and what the type system proves instead.** That all four subscriptions
/// are live *simultaneously* when the caches are read is structural and not testable here:
/// `LiveSubscriptions` owns the timed handle plus a fixed `[SubscriptionHandle;
/// VALIDATION_TARGET_COUNT]`, and the reads happen through a method borrowing that value, so a read
/// cannot be moved past a drop or reached with a short count without failing to compile. Building
/// one needs live subscriptions and therefore a server.
///
/// What *is* pure, and what a compiler cannot catch, is which targets get subscribed. A filter that
/// dropped the wrong one would leave a cache never subscribed — read as zero rows, recorded as a
/// `Semantics` mismatch, and reported as a composition failure that never happened. The array's
/// length would still be three, so the type would not notice.
#[test]
fn validation_covers_every_cache_the_timed_target_does_not() {
    // Totality first: an oracle missing a row would leave that timed target's selection unchecked.
    for target in ScreenTarget::ALL {
        assert_eq!(
            FROZEN_VALIDATION_TARGETS
                .iter()
                .filter(|(timed, _)| *timed == target)
                .count(),
            1,
            "{target:?} must appear exactly once in the frozen validation table"
        );
    }

    for (timed, expected) in FROZEN_VALIDATION_TARGETS {
        let validation = validation_targets(timed);

        // Ordered, because the driver subscribes in this order and the array is fixed-size: an
        // unordered comparison would accept a permutation the frozen table does not name.
        assert_eq!(
            validation, expected,
            "timing {timed:?} must validate exactly {expected:?}, in that order"
        );

        let subscribed: BTreeSet<&'static str> = validation
            .iter()
            .map(|target| target.canonical_tag())
            .collect();
        assert_eq!(
            subscribed.len(),
            VALIDATION_TARGET_COUNT,
            "timing {timed:?} must not subscribe one target twice while leaving another unread"
        );

        // The timed target is already subscribed; re-issuing it would be a second subscription on a
        // connection whose first one is the measurement.
        assert!(
            !subscribed.contains(timed.canonical_tag()),
            "{timed:?} is the timed target and must not be subscribed a second time"
        );

        // Together with the timed one, every cache the composition reads is covered exactly once.
        let mut all = subscribed;
        assert!(
            all.insert(timed.canonical_tag()),
            "the timed target must be the one the validation set omits"
        );
        assert_eq!(
            all.len(),
            ScreenTarget::ALL.len(),
            "all four caches must be subscribed before the composition is read"
        );
    }
}
