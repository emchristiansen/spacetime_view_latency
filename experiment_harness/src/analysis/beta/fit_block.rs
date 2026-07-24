//! The frozen grid + golden-section exponent search and identifiability test over one block's ladder.

use crate::analysis::beta::basin_refinement::BasinRefinement;
use crate::analysis::beta::basin_selection::BasinSelection;
use crate::analysis::beta::basin_termination::BasinTermination;
use crate::analysis::beta::beta_candidate::BetaCandidate;
use crate::analysis::beta::block_convergence::BlockConvergence;
use crate::analysis::beta::block_fit::BlockFit;
use crate::analysis::beta::block_point::BlockPoint;
use crate::analysis::beta::block_search_outcome::BlockSearchOutcome;
use crate::analysis::beta::constrained_fit::constrained_fit;
use crate::analysis::beta::golden_stop::GoldenStop;
use crate::analysis::beta::grid_node_outcome::GridNodeOutcome;
use crate::analysis::beta::grid_outcome::GridOutcome;
use crate::analysis::finite_f64::FiniteF64;
use crate::params::NUM_DOSES_USIZE;

/// The inclusive lower bound of the exponent search domain (spec: `β ∈ [0.1, 4.0]`). `pub(super)` so the
/// population selector can assert its identified exponents lie in this single canonical domain rather
/// than duplicating the bound.
pub(super) const BETA_MIN: f64 = 0.1;
/// The inclusive upper bound of the exponent search domain. `pub(super)` for the same single-source reason
/// as [`BETA_MIN`].
pub(super) const BETA_MAX: f64 = 4.0;
/// The number of inclusive uniform grid nodes over `[0.1, 4.0]` spaced by `0.05` (`(4.0 − 0.1)/0.05 + 1`).
/// `pub(crate)` so both the telemetry types ([`GridOutcome`](super::grid_outcome::GridOutcome)) and the
/// report's fixed-cardinality node array
/// ([`GridNodesReport`](crate::analysis::report::grid_nodes_report::GridNodesReport)) size their fixed node
/// arrays from this single frozen source rather than re-typing the `79` literal — the search, its
/// convergence record, and its report projection cannot drift.
pub(crate) const GRID_NODES: usize = 79;
/// The golden-section bracket-width stopping threshold: refine until the bracket is at most this wide.
/// `pub(super)` so the convergence telemetry ([`BasinTermination`](super::basin_termination::BasinTermination))
/// asserts its recorded `BracketWidthReached`/final-bracket width against this single frozen source
/// rather than a re-typed literal — the search and its record cannot drift.
pub(super) const GOLDEN_BRACKET_TOL: f64 = 1e-4;
/// The golden-section iteration cap (spec: "or 100 iterations have executed"). `pub(super)` for the same
/// single-source reason as [`GOLDEN_BRACKET_TOL`]: the telemetry bounds its iteration count by this const.
pub(super) const GOLDEN_MAX_ITERS: usize = 100;
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
/// The search's candidate ordering and selection are unchanged; it additionally records the complete
/// [`BlockConvergence`] telemetry of every refined basin — the coarse 79-node grid outcome, one
/// [`BasinRefinement`] per located basin (its grid-node index, brackets, iterations, and stop cause), and
/// the selected basin — and returns both bound together as a [`BlockSearchOutcome`], so a fit can never be
/// paired with a different block's convergence record (spec: "retain a typed per-block search summary ...
/// Do not report only the winning basin").
///
/// `pub(super)` — the only caller is [`BetaDescriptor`](super::beta_descriptor::BetaDescriptor), which
/// builds each block's ladder from the trusted graph.
pub(super) fn fit_block(points: &[BlockPoint; NUM_DOSES_USIZE]) -> BlockSearchOutcome {
    // Grid RSS at every node; a node with no constrained fit is `+∞`, so it is never a basin.
    let mut grid_rss = [f64::INFINITY; GRID_NODES];
    for (k, slot) in grid_rss.iter_mut().enumerate() {
        *slot = eval_rss(points, grid_node(k));
    }

    // Each node's typed grid outcome, with no `+∞`/NaN sentinel crossing into telemetry: a finite RSS is a
    // Feasible node, a `+∞` node is NoPositiveScale. Heap-first into a boxed fixed array, mirroring the
    // crate's fixed-array idiom, so the node cardinality is a property of the type.
    let nodes: Vec<GridNodeOutcome> = grid_rss
        .iter()
        .map(|&rss| match FiniteF64::try_new(rss) {
            Some(rss) => GridNodeOutcome::feasible(rss),
            None => GridNodeOutcome::NoPositiveScale,
        })
        .collect();
    let nodes: Box<[GridNodeOutcome; GRID_NODES]> = nodes
        .into_boxed_slice()
        .try_into()
        .ok()
        .expect("exactly GRID_NODES grid nodes were evaluated");

    // Refine every grid-local basin in ascending grid-node order, recording each basin's grid node and
    // refinement telemetry, and track the globally least-RSS candidate together with the *positional* index
    // of its refinement in `refined_basins`. The winner is chosen by the single frozen [`challenger_wins`]
    // tie-rule predicate — the same predicate the per-basin [`select_better`] uses — carrying the winning
    // basin's vector index, so the recorded selection names a direct `refined_basins` slot (not a grid-node
    // index) and the winning candidate can never disagree with the winning index.
    let mut basin_node_indices: Vec<usize> = Vec::new();
    let mut refined_basins: Vec<BasinRefinement> = Vec::new();
    let mut best: Option<(BetaCandidate, usize)> = None;
    for k in 0..GRID_NODES {
        if grid_rss[k].is_finite() && is_local_basin(&grid_rss, k) {
            let seed = constrained_fit(points, grid_node(k))
                .expect("a finite grid RSS guarantees a constrained fit at this node");
            let lo = grid_node(if k == 0 { 0 } else { k - 1 });
            let hi = grid_node(if k == GRID_NODES - 1 { GRID_NODES - 1 } else { k + 1 });
            let refinement = golden_section(points, lo, hi, seed);

            let termination = BasinTermination::new(
                k,
                FiniteF64::new(lo),
                FiniteF64::new(hi),
                FiniteF64::new(refinement.final_lo),
                FiniteF64::new(refinement.final_hi),
                refinement.iterations,
                refinement.stop,
            );
            let candidate = refinement.candidate;
            let basin_vec_index = refined_basins.len();
            basin_node_indices.push(k);
            refined_basins.push(BasinRefinement::new(termination, candidate));

            best = Some(match best {
                None => (candidate, basin_vec_index),
                Some(current) => {
                    if challenger_wins(current.0, candidate) {
                        (candidate, basin_vec_index)
                    } else {
                        current
                    }
                }
            });
        }
    }

    let grid = GridOutcome::new(nodes, basin_node_indices);

    // The selection names the winning basin by its positional index in `refined_basins`, or the explicit
    // no-feasible outcome when the grid located no basin at all.
    let selection = match best {
        Some((_, basin_index)) => BasinSelection::Selected { basin_index },
        None => BasinSelection::NoFeasiblePositiveScale,
    };
    let convergence = BlockConvergence::new(grid, refined_basins, selection);

    let fit = classify_fit(points, best.map(|(candidate, _)| candidate));
    BlockSearchOutcome::mint(fit, convergence)
}

/// Classify the globally least-RSS candidate into the authoritative per-block fit outcome — the exact
/// post-search rule the search has always applied, so the telemetry-building search and the classification
/// read one identical decision. `None` (the grid located no basin) is `NonPositiveScale`; otherwise a
/// candidate within `1e-4` of either domain bound is bound-pinned, an interior candidate whose RSS strictly
/// beats every feasible `β ± 0.001` probe is identifiable, and any other interior candidate is a flat
/// objective.
fn classify_fit(points: &[BlockPoint; NUM_DOSES_USIZE], candidate: Option<BetaCandidate>) -> BlockFit {
    let candidate = match candidate {
        Some(candidate) => candidate,
        // No basin has a positive-scale fit anywhere: the ladder carries no positive power-law scale.
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

/// The outcome of refining one grid-local basin: the least-RSS candidate the golden-section refinement kept
/// (ties to smaller β) and the exact bracket/iteration/stop telemetry of the refinement that produced it.
/// The candidate is unchanged from the previous search; the extra fields are the telemetry the convergence
/// record retains.
struct GoldenRefinement {
    /// The least-RSS candidate the refinement kept (ties to the smaller β).
    candidate: BetaCandidate,
    /// The lower endpoint of the bracket at termination.
    final_lo: f64,
    /// The upper endpoint of the bracket at termination.
    final_hi: f64,
    /// The number of golden-section iterations actually executed (one per completed loop body).
    iterations: usize,
    /// Why the refinement stopped, read from the actual terminating condition.
    stop: GoldenStop,
}

/// Golden-section-refine the RSS objective over the bracket `[lo, hi]`, seeded with a known finite
/// candidate (the basin's grid node), until the bracket is at most `GOLDEN_BRACKET_TOL` wide or
/// `GOLDEN_MAX_ITERS` iterations run. Returns the least-RSS candidate evaluated (ties to smaller β) plus
/// the exact bracket/iteration/stop telemetry. The interval choice keeps the smaller-β side on a tie, per
/// the frozen tie rule; the candidate selection is identical to the previous search.
fn golden_section(
    points: &[BlockPoint; NUM_DOSES_USIZE],
    lo0: f64,
    hi0: f64,
    seed: BetaCandidate,
) -> GoldenRefinement {
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

    // Read the stop cause from the loop's actual terminating condition. The guard is
    // `(hi - lo) > GOLDEN_BRACKET_TOL && iters < GOLDEN_MAX_ITERS`, so the loop exits once the bracket has
    // contracted to the tolerance or the cap is hit. When both coincide on the final step the width has been
    // reached, which is the convergence cause, so it is recorded as `BracketWidthReached`. `iters` counts one
    // per completed loop body, so it is the exact executed count with no off-by-one.
    let stop = if (hi - lo) <= GOLDEN_BRACKET_TOL {
        GoldenStop::BracketWidthReached
    } else {
        GoldenStop::IterationCapReached
    };
    GoldenRefinement {
        candidate: best,
        final_lo: lo,
        final_hi: hi,
        iterations: iters,
        stop,
    }
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

/// Whether `challenger` displaces `current` as the running best under the single frozen selection rule:
/// strictly lower RSS wins; on an RSS tie (within the relative tolerance) the strictly smaller β wins; a
/// full tie keeps the incumbent. This is the *sole* tie-rule decision — both the per-candidate
/// [`select_better`] and the index-carrying global selection in [`fit_block`] route through it, so the
/// winning candidate and the recorded winning basin index are decided by one predicate and can never drift
/// apart.
fn challenger_wins(current: BetaCandidate, challenger: BetaCandidate) -> bool {
    if rss_strictly_lower(challenger.rss(), current.rss()) {
        true
    } else if rss_strictly_lower(current.rss(), challenger.rss()) {
        false
    } else {
        challenger.beta() < current.beta()
    }
}

/// The better of two candidates under the frozen [`challenger_wins`] rule: strictly lower RSS wins; on an
/// RSS tie the smaller β wins; a full tie keeps `current`.
fn select_better(current: BetaCandidate, challenger: BetaCandidate) -> BetaCandidate {
    if challenger_wins(current, challenger) {
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
