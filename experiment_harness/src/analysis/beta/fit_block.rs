//! The frozen grid + golden-section exponent search and identifiability test over one block's ladder.

use crate::analysis::beta::beta_candidate::BetaCandidate;
use crate::analysis::beta::block_fit::BlockFit;
use crate::analysis::beta::block_point::BlockPoint;
use crate::analysis::beta::constrained_fit::constrained_fit;
use crate::params::NUM_DOSES_USIZE;

/// The inclusive lower bound of the exponent search domain (spec: `β ∈ [0.1, 4.0]`). `pub(super)` so the
/// population selector can assert its identified exponents lie in this single canonical domain rather
/// than duplicating the bound.
pub(super) const BETA_MIN: f64 = 0.1;
/// The inclusive upper bound of the exponent search domain. `pub(super)` for the same single-source reason
/// as [`BETA_MIN`].
pub(super) const BETA_MAX: f64 = 4.0;
/// The number of inclusive uniform grid nodes over `[0.1, 4.0]` spaced by `0.05` (`(4.0 − 0.1)/0.05 + 1`).
const GRID_NODES: usize = 79;
/// The golden-section bracket-width stopping threshold: refine until the bracket is at most this wide.
const GOLDEN_BRACKET_TOL: f64 = 1e-4;
/// The golden-section iteration cap (spec: "or 100 iterations have executed").
const GOLDEN_MAX_ITERS: usize = 100;
/// The relative RSS comparison tolerance: two residual sums tie within `RSS_REL_TOL · max(r1, r2, 1)`.
const RSS_REL_TOL: f64 = 1e-9;
/// The interior identifiability probe offset: an interior estimate must beat both `β ± PROBE_DELTA`.
const PROBE_DELTA: f64 = 0.001;
/// The bound-pinning threshold: a selected exponent within this of `0.1` or `4.0` is `PinnedAtBound`.
const BOUND_EPS: f64 = 1e-4;
/// The reciprocal golden ratio `1/φ = φ − 1`, the golden-section interior-point fraction.
const INV_GOLDEN: f64 = 0.618_033_988_749_894_9;

/// Fit the secondary exponent over one arm block's ten-dose ladder by the frozen dependency-free search
/// (spec: "Frozen dependency-free fit"): evaluate the 79 grid nodes, golden-section-refine every
/// grid-local RSS basin (including edge basins), pick the globally least-RSS refined candidate (ties to
/// the smaller β), then classify it as bound-pinned, interior-identifiable, or non-identifiable.
///
/// `pub(super)` — the only caller is [`BetaDescriptor`](super::beta_descriptor::BetaDescriptor), which
/// builds each block's ladder from the trusted graph.
pub(super) fn fit_block(points: &[BlockPoint; NUM_DOSES_USIZE]) -> BlockFit {
    // Grid RSS at every node; a node with no constrained fit is `+∞`, so it is never a basin.
    let mut grid_rss = [f64::INFINITY; GRID_NODES];
    for (k, slot) in grid_rss.iter_mut().enumerate() {
        *slot = eval_rss(points, grid_node(k));
    }

    // Refine every grid-local basin and keep the global least-RSS candidate (ties to the smaller β).
    let mut best: Option<BetaCandidate> = None;
    for k in 0..GRID_NODES {
        if grid_rss[k].is_finite() && is_local_basin(&grid_rss, k) {
            let seed = constrained_fit(points, grid_node(k))
                .expect("a finite grid RSS guarantees a constrained fit at this node");
            let lo = grid_node(if k == 0 { 0 } else { k - 1 });
            let hi = grid_node(if k == GRID_NODES - 1 { GRID_NODES - 1 } else { k + 1 });
            let refined = golden_section(points, lo, hi, seed);
            best = Some(match best {
                Some(current) => select_better(current, refined),
                None => refined,
            });
        }
    }

    // No basin has a positive-scale fit anywhere: the ladder carries no positive power-law scale.
    let candidate = match best {
        Some(candidate) => candidate,
        None => return BlockFit::NonPositiveScale,
    };

    // A candidate within `1e-4` of either global domain bound is bound-pinned, not identifiable.
    if (candidate.beta() - BETA_MIN).abs() <= BOUND_EPS
        || (candidate.beta() - BETA_MAX).abs() <= BOUND_EPS
    {
        return BlockFit::PinnedAtBound(candidate);
    }

    // Interior identifiability: RSS strictly below every *feasible* `β ± 0.001` probe by more than the
    // tolerance. A probe is feasible only when its exponent stays within the inclusive `[0.1, 4.0]` search
    // domain; an out-of-domain probe is omitted — never evaluated or clamped — so a non-pinned candidate
    // within `0.001` of one bound is tested only against its inward probe (the domain half-width `1.95`
    // exceeds `0.001`, so at least one probe is always feasible).
    if beats_probe(points, candidate, -PROBE_DELTA) && beats_probe(points, candidate, PROBE_DELTA) {
        BlockFit::Identifiable(candidate)
    } else {
        BlockFit::FlatObjective(candidate)
    }
}

/// Whether the `candidate` satisfies its `β + offset` identifiability probe: either the probe is
/// infeasible — its exponent falls outside the inclusive `[0.1, 4.0]` search domain, so it is omitted and
/// imposes no constraint (spec: "test against every feasible `±0.001` probe; feasible iff the exponent
/// remains in inclusive `[0.1, 4.0]`; omit out-of-domain probes — never evaluate or clamp") — or it is
/// feasible and the candidate's RSS is strictly below the probe's by more than the relative tolerance. An
/// infeasible probe is never evaluated, so `constrained_fit` is never called outside the search domain.
fn beats_probe(points: &[BlockPoint; NUM_DOSES_USIZE], candidate: BetaCandidate, offset: f64) -> bool {
    let probe = candidate.beta() + offset;
    if !(BETA_MIN..=BETA_MAX).contains(&probe) {
        return true;
    }
    rss_strictly_lower(candidate.rss(), eval_rss(points, probe))
}

/// The `k`-th inclusive grid node `β_k = (2 + k)/20`, giving `β_0 = 0.1` and `β_78 = 4.0` exactly at the
/// domain endpoints with the frozen `0.05` spacing.
fn grid_node(k: usize) -> f64 {
    (2.0 + k as f64) / 20.0
}

/// Whether grid node `k` is a local RSS basin: its RSS is no greater than each present neighbour's
/// (edge nodes compare against their single neighbour). The `≤` comparison makes a flat plateau a run of
/// basins, which the global tie-to-smaller-β selection then resolves deterministically.
fn is_local_basin(grid_rss: &[f64; GRID_NODES], k: usize) -> bool {
    let left_ok = k == 0 || grid_rss[k] <= grid_rss[k - 1];
    let right_ok = k == GRID_NODES - 1 || grid_rss[k] <= grid_rss[k + 1];
    left_ok && right_ok
}

/// Golden-section-refine the RSS objective over the bracket `[lo, hi]`, seeded with a known finite
/// candidate (the basin's grid node), until the bracket is at most `GOLDEN_BRACKET_TOL` wide or
/// `GOLDEN_MAX_ITERS` iterations run. Returns the least-RSS candidate evaluated (ties to smaller β). The
/// interval choice keeps the smaller-β side on a tie, per the frozen tie rule.
fn golden_section(
    points: &[BlockPoint; NUM_DOSES_USIZE],
    lo0: f64,
    hi0: f64,
    seed: BetaCandidate,
) -> BetaCandidate {
    let mut lo = lo0;
    let mut hi = hi0;
    let mut best = seed;
    best = consider(points, best, lo);
    best = consider(points, best, hi);

    let mut c = hi - INV_GOLDEN * (hi - lo);
    let mut d = lo + INV_GOLDEN * (hi - lo);
    let mut fc = eval_rss(points, c);
    let mut fd = eval_rss(points, d);
    best = consider(points, best, c);
    best = consider(points, best, d);

    let mut iters = 0;
    while (hi - lo) > GOLDEN_BRACKET_TOL && iters < GOLDEN_MAX_ITERS {
        if rss_strictly_lower(fd, fc) {
            // The minimum lies toward larger β: keep `[c, hi]`.
            lo = c;
            c = d;
            fc = fd;
            d = lo + INV_GOLDEN * (hi - lo);
            fd = eval_rss(points, d);
            best = consider(points, best, d);
        } else {
            // Tie or fc strictly lower: keep `[lo, d]`, the smaller-β side (frozen tie rule).
            hi = d;
            d = c;
            fd = fc;
            c = hi - INV_GOLDEN * (hi - lo);
            fc = eval_rss(points, c);
            best = consider(points, best, c);
        }
        iters += 1;
    }
    best
}

/// Fold the exponent `beta` into the running best candidate: if it admits a constrained fit, keep the
/// better of the two; otherwise the running best is unchanged.
fn consider(
    points: &[BlockPoint; NUM_DOSES_USIZE],
    best: BetaCandidate,
    beta: f64,
) -> BetaCandidate {
    match constrained_fit(points, beta) {
        Some(candidate) => select_better(best, candidate),
        None => best,
    }
}

/// The better of two candidates: strictly lower RSS wins; on an RSS tie (within the relative tolerance)
/// the smaller β wins (frozen tie rule).
fn select_better(current: BetaCandidate, challenger: BetaCandidate) -> BetaCandidate {
    if rss_strictly_lower(challenger.rss(), current.rss()) {
        challenger
    } else if rss_strictly_lower(current.rss(), challenger.rss()) {
        current
    } else if challenger.beta() < current.beta() {
        challenger
    } else {
        current
    }
}

/// The RSS at `beta`, or `+∞` when no constrained fit exists there — so a missing fit is unambiguously
/// worse than any real one in every comparison.
fn eval_rss(points: &[BlockPoint; NUM_DOSES_USIZE], beta: f64) -> f64 {
    constrained_fit(points, beta)
        .map(BetaCandidate::rss)
        .unwrap_or(f64::INFINITY)
}

/// Whether `lower` is strictly below `higher` by more than the relative RSS tolerance
/// `RSS_REL_TOL · max(lower, higher, 1)` (spec: relative tolerance `1e-9 · max(RSS₁, RSS₂, 1)`). A
/// non-finite `higher` (no fit) counts as strictly greater than any finite `lower`; a non-finite `lower`
/// is never strictly lower.
fn rss_strictly_lower(lower: f64, higher: f64) -> bool {
    if !higher.is_finite() {
        return lower.is_finite();
    }
    if !lower.is_finite() {
        return false;
    }
    higher - lower > RSS_REL_TOL * higher.max(lower).max(1.0)
}

#[cfg(test)]
mod tests;
