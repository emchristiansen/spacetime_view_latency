//! Each role selects its own measured target, and the two roles never select the same one.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::measured_target::MeasuredTarget;

/// Coverage: this choice of table *is* the Arm/Control distinction, so a crossed mapping would
/// measure the direct base table and record it as the candidate's own read set. The inequality is
/// asserted because both match arms could return the same target and each equality still pass.
#[test]
fn each_role_measures_its_own_target() {
    assert_eq!(
        MeasuredTarget::of(RunRole::Arm),
        MeasuredTarget::EntityOwnerSenderView,
        "the Arm measures the module's sender-scoped view, which is the candidate under test"
    );
    assert_eq!(
        MeasuredTarget::of(RunRole::Control),
        MeasuredTarget::EntityOwner,
        "the Control measures the direct public base table it is matched against"
    );
    assert_ne!(
        MeasuredTarget::of(RunRole::Arm),
        MeasuredTarget::of(RunRole::Control),
        "the two roles differ in exactly this choice of table, so they can never select the same one"
    );
}
