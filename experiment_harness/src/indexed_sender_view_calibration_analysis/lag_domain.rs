//! Which lags the autocorrelation diagnostic is computed at.

use crate::indexed_sender_view_calibration_analysis::candidate_window::CandidateWindow;

/// Every lag `k` the autocorrelation diagnostic is computed at, ascending: `1 ..= (largest candidate
/// window − 1)`.
///
/// # Why a set of lags rather than lag-1
///
/// §568 names the admissible output as "lag dependence/**effective information**", and effective
/// information under serial dependence is a statement about dependence *across* lags — a single lag
/// cannot express it. §569 says the decision considers "**autocorrelation**" without naming a lag,
/// and §615 says "exact **lag** numerator/autocovariance plus left/right energy terms", again
/// without a `k`. (§565 does mention "lag-1 autocorrelation 0.51", but that is a retained-evidence
/// observation about a different historical Site 2 probe, not a requirement on this analyzer.)
/// Reporting only `k = 1` would answer a narrower question than the rule asks.
///
/// # Why this domain
///
/// Derived from [`CandidateWindow::ALL`] rather than preregistered as a new constant, so it moves
/// with the candidate set instead of drifting from it.
///
/// Autocorrelation bears on this decision because it determines how much independent information a
/// width-`W` window's median actually carries, and only lags strictly below `W` can induce
/// dependence *among the samples inside* one such window. The domain must therefore reach
/// `W − 1` for the **largest** candidate, not the smallest: stopping at the smallest candidate's
/// `W − 1` — nine, here — would leave within-window dependence unreported for every candidate from
/// twenty upward, which is six of the seven.
///
/// **Sparse tails are retained rather than censored.** At `k = 999` a thousand-sample series has
/// exactly one pair, and a coefficient from one pair carries almost no information. That is itself
/// evidence, and each element publishes its own `pairs` count so the support is explicit — dropping
/// those lags would silently substitute this module's judgement about which evidence is worth
/// looking at for Control's.
///
/// **No summation, no truncation, no effective sample size.** Each lag is reported with its exact
/// components and stops there. Combining them into an `n_eff` would require choosing a truncation
/// rule and a summation convention, and the result would be one number that reads as an answer —
/// which is Control's to derive in SSOT, not this module's to assert.
pub(crate) fn lag_domain() -> Vec<usize> {
    let widest = CandidateWindow::ALL
        .iter()
        .map(|window| window.get())
        .max()
        .expect("the candidate set is never empty");
    (1..widest).collect()
}

#[cfg(test)]
mod tests;
