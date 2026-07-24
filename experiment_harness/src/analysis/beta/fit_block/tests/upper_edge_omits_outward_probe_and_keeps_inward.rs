//! Upper-edge feasible-probe proof: the mirror of
//! [`lower_edge_omits_outward_probe_and_keeps_inward`](super::lower_edge_omits_outward_probe_and_keeps_inward)
//! at the *upper* domain bound `4.0`. An interior candidate within `PROBE_DELTA = 0.001` of `4.0` — but
//! farther than the `BOUND_EPS = 1e-4` pin threshold — is tested only against its single feasible *inward*
//! probe. Its outward `β + 0.001` probe rises above `4.0`, outside the inclusive `[0.1, 4.0]` domain, and
//! is omitted rather than evaluated (spec: "an out-of-domain probe is omitted rather than evaluated or
//! clamped").
//!
//! The omission is load-bearing: the generating exponent is [`GENERATING_BETA`] = `4.0005`, *above* the
//! domain, so the objective's exact minimum sits at the out-of-domain outward probe — its RSS is zero
//! there, strictly below the interior candidate's — and evaluating it would reject the candidate as flat.
//! Omitting it leaves the feasible inward probe, which the candidate genuinely beats, to certify it. A
//! second, deliberately non-beating candidate confirms the inward probe genuinely decides.

use crate::analysis::beta::beta_candidate::BetaCandidate;
use crate::analysis::beta::constrained_fit::constrained_fit;
use crate::analysis::beta::fit_block::tests::ladder_from;
use crate::analysis::beta::fit_block::{beats_probe, eval_rss, BETA_MAX, BETA_MIN, BOUND_EPS, PROBE_DELTA};

/// An interior exponent `5e-4` below the upper bound: farther than the `1e-4` pin threshold (so not
/// bound-pinned) yet within `1e-3` of `4.0` (so its outward `β + 0.001` probe is out of domain).
const EDGE_BETA: f64 = 3.9995;
/// The generating exponent, just *above* the domain, so the exact RSS minimum lies at the out-of-domain
/// outward probe — making its omission the thing that preserves identifiability.
const GENERATING_BETA: f64 = 4.0005;

#[test]
fn upper_edge_omits_outward_probe_and_keeps_inward() {
    // The edge condition on the constants: EDGE_BETA is a genuine interior, non-pinned exponent within one
    // probe-delta of the upper bound; its outward probe is strictly out of domain while its inward probe
    // stays inside.
    assert!(
        BETA_MAX - EDGE_BETA > BOUND_EPS,
        "EDGE_BETA is farther than the pin threshold from the upper bound"
    );
    assert!(
        BETA_MAX - EDGE_BETA < PROBE_DELTA,
        "EDGE_BETA is within one probe-delta of the upper bound"
    );
    assert!(
        EDGE_BETA + PROBE_DELTA > BETA_MAX,
        "the outward probe β + 0.001 rises above the domain and is infeasible"
    );
    assert!(
        (BETA_MIN..=BETA_MAX).contains(&(EDGE_BETA - PROBE_DELTA)),
        "the inward probe β − 0.001 stays inside the domain and is feasible"
    );

    // y = N^4.0005: at β = 4.0005 the fit's x = N^4.0005 equals y exactly, so RSS is zero there and grows
    // as β moves inward. The candidate is the genuine constrained fit at the interior EDGE_BETA.
    let points = ladder_from(|n| n.powf(GENERATING_BETA));
    let candidate = constrained_fit(&points, EDGE_BETA)
        .expect("an interior exponent near the upper bound admits a constrained fit");
    assert_eq!(
        candidate.beta(),
        EDGE_BETA,
        "constrained_fit stamps the interior exponent it solved"
    );

    // Outward probe (β + 0.001 = 4.0005) is out of domain and *decisive if evaluated*: its RSS is zero,
    // strictly below the candidate's, so evaluating it would reject the candidate. beats_probe omits it.
    let outward_rss = eval_rss(&points, EDGE_BETA + PROBE_DELTA);
    assert!(
        outward_rss < candidate.rss(),
        "the out-of-domain outward probe has strictly lower RSS ({outward_rss}) than the candidate ({}), \
         so evaluating it would reject the candidate",
        candidate.rss()
    );
    assert!(
        beats_probe(&points, candidate, PROBE_DELTA),
        "the outward probe is omitted (out of domain), so beats_probe returns true despite its lower RSS"
    );

    // Inward probe (β − 0.001 = 3.9985) is in domain and load-bearing: the candidate's RSS is strictly
    // below it, so this single feasible probe certifies the candidate.
    let inward_rss = eval_rss(&points, EDGE_BETA - PROBE_DELTA);
    assert!(
        candidate.rss() < inward_rss,
        "the candidate's RSS ({}) is strictly below the feasible inward probe's ({inward_rss})",
        candidate.rss()
    );
    assert!(
        beats_probe(&points, candidate, -PROBE_DELTA),
        "the feasible inward probe is evaluated and the candidate beats it"
    );

    // The inward probe genuinely decides: a candidate whose RSS is exactly the inward probe's does not beat
    // it, so beats_probe returns false — the inward test is load-bearing, not vacuously true.
    let non_beating = BetaCandidate::new(EDGE_BETA, candidate.a(), candidate.b(), inward_rss);
    assert!(
        !beats_probe(&points, non_beating, -PROBE_DELTA),
        "a candidate no better than the feasible inward probe fails it"
    );
}
