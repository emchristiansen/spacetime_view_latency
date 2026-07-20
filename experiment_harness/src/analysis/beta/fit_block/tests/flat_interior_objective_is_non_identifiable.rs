//! A shallow interior basin is retained but reported [`BlockFit::FlatObjective`] /
//! [`NonIdentifiableReason::FlatObjective`], not an identified estimate (spec: an interior candidate is
//! identifiable only when its RSS is strictly below every feasible `β ± 0.001` probe by more than the
//! relative tolerance; otherwise `FlatObjective`). The candidate here is strictly interior (`β ≈ 1.962`),
//! so both probes are feasible and both must be beaten. This is distinct from non-positive-scale rejection
//! and bound pinning.
//!
//! The fixture is a narrow `N ∈ [500, 1040]` ladder carrying a faint `N²` trend plus a large
//! high-frequency alternating component (`±1.5·10⁶`) that no smooth power can absorb. The alternating
//! energy inflates the irreducible RSS to ≈ 2.18·10¹³ — and hence the `1e-9·max(…)` tolerance to
//! ≈ 2.18·10⁴ — while the trend still bends the objective into an interior minimum near `β ≈ 1.962`. The
//! broad `0.05` grid resolves that basin (its neighbours' RSS are `~10⁶` higher, far beyond the
//! tolerance), yet across the fine `±0.001` probes the RSS moves by under `±8·10³` — within the
//! tolerance — so the exponent is not distinguishable. The inequalities that force `FlatObjective` are
//! re-derived here from the same frozen constants rather than snapshotted.

use crate::analysis::beta::block_fit::BlockFit;
use crate::analysis::beta::block_point::BlockPoint;
use crate::analysis::beta::constrained_fit::constrained_fit;
use crate::analysis::beta::fit_block::fit_block;
use crate::analysis::beta::non_identifiable_reason::NonIdentifiableReason;
use crate::params::NUM_DOSES_USIZE;

/// The narrow ten-dose `N` ladder over which the alternating component is nearly, but not exactly,
/// unfittable — narrow enough that the objective is shallow, wide enough that the `N²` trend still
/// carves an interior minimum rather than pinning at the lower bound.
const NARROW_N: [f64; NUM_DOSES_USIZE] =
    [500.0, 560.0, 620.0, 680.0, 740.0, 800.0, 860.0, 920.0, 980.0, 1040.0];
/// The alternating half-amplitude. Chosen at the centre of the FlatObjective band: below it the basin
/// sharpens to Identifiable, above it the inflated flatness pulls the global minimum to the pinned lower
/// bound.
const ALTERNATING: f64 = 1_500_000.0;
/// The frozen relative-RSS tolerance coefficient, re-stated here to re-derive the identifiability
/// inequality the search engine applies (`fit_block`'s private `RSS_REL_TOL`).
const RSS_REL_TOL: f64 = 1e-9;
/// The frozen interior identifiability probe offset (`fit_block`'s private `PROBE_DELTA`).
const PROBE_DELTA: f64 = 0.001;

#[test]
fn flat_interior_objective_is_non_identifiable() {
    // y = N² + (±ALTERNATING) + ALTERNATING: the constant offset keeps every latency nonnegative (the
    // free intercept absorbs it); the alternating sign is the near-unfittable component.
    let ladder: [BlockPoint; NUM_DOSES_USIZE] = std::array::from_fn(|dose_index| {
        let n = NARROW_N[dose_index];
        let alternating = if dose_index % 2 == 0 { ALTERNATING } else { -ALTERNATING };
        BlockPoint::from_synthetic(n, n * n + alternating + ALTERNATING)
    });

    let fit = fit_block(&ladder).fit();
    let candidate = match fit {
        BlockFit::FlatObjective(candidate) => candidate,
        other => panic!("a shallow interior basin must be a retained FlatObjective, got {other:?}"),
    };

    // Retained but non-identifiable via the flat-objective taxonomy — never Identifiable/Pinned/NonPos.
    assert!(fit.candidate().is_some(), "FlatObjective retains its rejected interior candidate");
    assert_eq!(
        fit.non_identifiable_reason(),
        Some(NonIdentifiableReason::FlatObjective),
        "a flat interior objective is non-identifiable via FlatObjective"
    );
    assert!(!fit.is_identifiable(), "a flat-objective outcome is not an identified estimate");
    assert!(!fit.is_pinned_at_bound(), "the selected exponent is interior, not bound-pinned");

    let beta = candidate.beta();
    assert!(
        beta > 0.1 + 1e-4 && beta < 4.0 - 1e-4,
        "the retained candidate is a strictly interior exponent, got {beta}"
    );

    // Re-derive the frozen strict-improvement predicate and the RSS at the exact probe offsets the engine
    // uses, so this proof shows *why* the outcome is FlatObjective.
    let rss_at = |offset: f64| {
        constrained_fit(&ladder, beta + offset)
            .expect("every probe near the interior basin admits a constrained fit")
            .rss()
    };
    let strictly_lower = |lower: f64, higher: f64| higher - lower > RSS_REL_TOL * higher.max(lower).max(1.0);

    let rss = candidate.rss();
    let below = rss_at(-PROBE_DELTA);
    let above = rss_at(PROBE_DELTA);
    let far_below = rss_at(-0.05);
    let far_above = rss_at(0.05);

    // Broad basin: the 0.05 grid neighbours are strictly higher, so the search genuinely locates an
    // interior minimum rather than sitting on a domain-wide plateau.
    assert!(
        strictly_lower(rss, far_below) && strictly_lower(rss, far_above),
        "the broad 0.05 grid resolves the interior basin: RSS {rss} strictly below neighbours \
         {far_below} and {far_above}"
    );
    // Fine flatness: *neither* ±0.001 probe is a strict improvement over the candidate — each strict-lower
    // comparison is individually false — so the frozen identifiability test (strictly lower than BOTH
    // probes) fails on both sides, not merely on one. The exponent is indistinguishable within tolerance.
    assert!(
        !strictly_lower(rss, below),
        "the −0.001 probe RSS {below} is within tolerance of the candidate RSS {rss}, not strictly higher"
    );
    assert!(
        !strictly_lower(rss, above),
        "the +0.001 probe RSS {above} is within tolerance of the candidate RSS {rss}, not strictly higher"
    );
}
