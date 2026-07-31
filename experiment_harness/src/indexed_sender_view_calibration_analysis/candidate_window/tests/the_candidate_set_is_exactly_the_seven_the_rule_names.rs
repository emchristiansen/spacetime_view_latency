//! `CandidateWindow::ALL` is exactly §569's seven-element candidate set, ascending.

use super::super::CandidateWindow;

/// Coverage: the enumeration a reader would have to check by eye against the spec.
///
/// Asserted as a whole list rather than by length plus spot checks, because the failure this guards
/// against is a *quiet* one: an eighth candidate, or a changed value, would still be a well-typed
/// enum and would still produce a plausible-looking report. Nothing else in this namespace can
/// notice, since every downstream type takes whatever `ALL` contains.
///
/// Ascending order is asserted too. The report emits candidates in this order, and §569 discusses
/// them smallest-first — a reader comparing rows against the rule should not have to re-sort them.
#[test]
fn the_candidate_set_is_exactly_the_seven_the_rule_names() {
    let counts: Vec<usize> = CandidateWindow::ALL
        .iter()
        .map(|window| window.get())
        .collect();
    assert_eq!(
        counts,
        vec![10, 20, 50, 100, 200, 500, 1_000],
        "the candidate set is exactly the seven counts SSOT §569 names, in ascending order"
    );
}
