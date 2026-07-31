//! The lag domain is `1 ..= (widest candidate − 1)`, contiguous and derived from the candidate set.

use crate::indexed_sender_view_calibration_analysis::candidate_window::CandidateWindow;
use crate::indexed_sender_view_calibration_analysis::lag_domain::lag_domain;

/// Coverage: the exact domain, and the property that fixes its upper end.
///
/// The endpoint is the load-bearing part. Only lags strictly below `W` can induce dependence among
/// the samples inside a width-`W` window, so the domain has to reach `W − 1` for the **widest**
/// candidate — stopping at the narrowest candidate's `W − 1` would leave within-window dependence
/// unreported for six of the seven candidates. Asserting against `CandidateWindow::ALL` rather than
/// against `999` keeps the two moving together if the candidate set ever changes.
#[test]
fn the_lag_domain_reaches_one_below_the_widest_candidate() {
    let widest = CandidateWindow::ALL
        .iter()
        .map(|window| window.get())
        .max()
        .expect("the candidate set is never empty");
    let domain = lag_domain();

    assert_eq!(
        domain.first().copied(),
        Some(1),
        "lag 0 is the series against itself and carries no dependence information"
    );
    assert_eq!(
        domain.last().copied(),
        Some(widest - 1),
        "the domain reaches one below the widest candidate, so every candidate's within-window \
         dependence is covered"
    );
    assert_eq!(
        domain.len(),
        widest - 1,
        "the domain is contiguous, with no lag silently dropped from the middle"
    );
    assert!(
        domain.windows(2).all(|pair| pair[1] == pair[0] + 1),
        "the domain is ascending and contiguous"
    );
}
