//! Parameterizing the timed-subscription helper did not re-word the campaign's own diagnostics.

use crate::view_read_set_campaign::measured_target::MeasuredTarget;

use super::super::measured_target_description;

/// Coverage: the exact substring the E3 and E4 adapters interpolate into their two failure
/// diagnostics, for every campaign target.
///
/// **Why this is worth a test at all.** `subscribe_retained_sql` used to name its subscription by
/// formatting a [`MeasuredTarget`] inline, and was parameterized so the `ControlRegistry` discovery
/// screen — whose four targets that enum does not name — could reach the same clock. A shared helper
/// that derived its own description would have silently re-worded the campaign's long-standing
/// "the subscription to `EntityOwnerSenderView` was not applied within …" into whatever suited the
/// newer caller. Supplying the description prevents that; this pins that the campaign's supplied one
/// is still the value it always was.
///
/// The expectations are literal strings, deliberately. Asserting against `format!("{target:?}")`
/// would be asserting the function equals itself, and would pass just as happily after a variant
/// rename that changed every recorded diagnostic.
///
/// The surrounding message templates are not reachable without a live server — they fire only on a
/// subscription timeout or a dropped applied channel — so they are covered by inspection, as this
/// module's other network-bound branches are. What a rename can silently change is the target name,
/// and that is exactly what this holds still.
#[test]
fn the_campaign_subscription_diagnostic_keeps_its_target_wording() {
    for (target, expected) in [
        (
            MeasuredTarget::EntityOwnerSenderView,
            "EntityOwnerSenderView",
        ),
        (MeasuredTarget::EntityOwner, "EntityOwner"),
    ] {
        assert_eq!(
            measured_target_description(target),
            expected,
            "the campaign names {target:?} in its subscription diagnostics as {expected:?}, and \
             renaming the variant would rewrite every diagnostic it has ever reported"
        );
    }
}
