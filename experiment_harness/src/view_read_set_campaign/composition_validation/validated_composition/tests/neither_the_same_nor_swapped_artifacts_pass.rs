//! Passing one artifact as both phases, or the two in the wrong order, fails.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;
use crate::view_read_set_campaign::composition_validation::validated_composition::ValidatedComposition;

use super::fixture;

/// Coverage: `validate` takes two [`ObservedRowSet`](crate::view_read_set_campaign::composition_validation::observed_row_set::ObservedRowSet)s
/// positionally, so the two ways a caller can misuse it are handing over one artifact twice and
/// handing them over in the wrong order. Neither is caught by comparing paths or digests — nothing
/// in the check does that — so this test pins the mechanism that *does* reject them: the phase
/// expectations are mutually exclusive over a nonempty owned slice, since the seeded payload and the
/// batch's derived final payloads are distinct frozen literals.
///
/// The distinction matters for what may be claimed. This establishes that no single artifact can
/// satisfy both sides, and that the sides are not interchangeable — it does **not** establish that
/// either artifact was taken at the phase it is filed under. That is the driver's measurement path,
/// and remains a provenance question these types do not answer.
///
/// Both misuses are checked in one test because they are one claim about the same mechanism, and
/// checking only the duplicate would leave the ordering silently free.
#[test]
fn neither_the_same_nor_swapped_artifacts_pass() {
    let directory = fixture::attempt_directory("artifact-misuse");
    let before = fixture::persist(
        &directory,
        ObservedRowSetLabel::SeededBeforeMeasurement,
        fixture::seeded_rows(RunRole::Arm),
    );
    let after = fixture::persist(
        &directory,
        ObservedRowSetLabel::AfterSaturatedBatch,
        fixture::final_rows(RunRole::Arm),
    );

    // The same artifact as both phases: it satisfies whichever census matches its payloads and
    // fails the other.
    let duplicated = ValidatedComposition::validate(
        fixture::transition(RunRole::Arm),
        before.clone(),
        before.clone(),
    )
    .expect_err("one observation cannot be both phases of a transition");
    assert!(
        format!("{duplicated:#}").contains("saturated batch confirmed"),
        "the seeded artifact passed twice must fail the *after* census, got {duplicated:#}"
    );

    // The two artifacts in the wrong order: each fails the phase it was not taken at, and the
    // before-census is reached first.
    let swapped = ValidatedComposition::validate(fixture::transition(RunRole::Arm), after, before)
        .expect_err("the two observations are not interchangeable");
    assert!(
        format!("{swapped:#}").contains("before the first measured write"),
        "the swapped pair must fail at the before-phase census, got {swapped:#}"
    );
}
