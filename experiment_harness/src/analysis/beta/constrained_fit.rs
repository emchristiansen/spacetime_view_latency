//! The constrained least-squares fit over `a ≥ 0, b > 0` at one fixed exponent `β`.

use crate::analysis::beta::beta_candidate::BetaCandidate;
use crate::analysis::beta::block_point::BlockPoint;
use crate::params::NUM_DOSES_USIZE;

/// Solve the convex least-squares problem `minimize Σ(y_i − a − b x_i)²` subject to `a ≥ 0, b > 0` at a
/// fixed exponent `β`, where `x_i = N_i^β` over the block's ten-dose ladder (spec: "Frozen
/// dependency-free fit"). Returns the fitted candidate, or `None` when no admissible fit has strictly
/// positive finite scale.
///
/// The frozen rule is exactly: use the unconstrained ordinary-least-squares solution when it is feasible
/// (`a ≥ 0`, `b > 0`, finite); otherwise evaluate the `a = 0` edge with `b = Σx_i y_i / Σx_i²`; if
/// neither yields a strictly positive finite `b`, no fit exists. Because every `y_i ≥ 0` and every
/// `x_i > 0`, the `a = 0` edge scale is `≥ 0`, so the only no-fit case is a ladder whose positive
/// scale collapses to zero (an all-zero-latency ladder).
///
/// `pub(super)` — the only caller is the [`fit_block`](super::fit_block::fit_block) search.
pub(super) fn constrained_fit(
    points: &[BlockPoint; NUM_DOSES_USIZE],
    beta: f64,
) -> Option<BetaCandidate> {
    let count = NUM_DOSES_USIZE as f64;

    // x_i = N_i^β. A non-finite power (overflow at an extreme β) has no usable fit.
    let mut xs = [0.0f64; NUM_DOSES_USIZE];
    for (slot, point) in xs.iter_mut().zip(points.iter()) {
        let x = point.n().powf(beta);
        if !x.is_finite() {
            return None;
        }
        *slot = x;
    }

    let sum_x: f64 = xs.iter().sum();
    let sum_y: f64 = points.iter().map(|point| point.y()).sum();
    let mean_x = sum_x / count;
    let mean_y = sum_y / count;

    // Unconstrained OLS-with-intercept: b = Σ(x−x̄)(y−ȳ) / Σ(x−x̄)², a = ȳ − b·x̄. Feasible only when
    // the centred x-spread is positive and the solution respects a ≥ 0, b > 0 with finite residuals.
    let mut centred_xx = 0.0f64;
    let mut centred_xy = 0.0f64;
    for (x, point) in xs.iter().zip(points.iter()) {
        let dx = x - mean_x;
        centred_xx += dx * dx;
        centred_xy += dx * (point.y() - mean_y);
    }
    if centred_xx > 0.0 {
        let b = centred_xy / centred_xx;
        let a = mean_y - b * mean_x;
        if a.is_finite() && b.is_finite() && a >= 0.0 && b > 0.0 {
            let rss = residual_sum_of_squares(&xs, points, a, b);
            if rss.is_finite() {
                return Some(BetaCandidate::new(beta, a, b, rss));
            }
        }
    }

    // a = 0 edge: minimize Σ(y_i − b x_i)² over b alone, giving b = Σx_i y_i / Σx_i².
    let sum_xy: f64 = xs.iter().zip(points.iter()).map(|(x, p)| x * p.y()).sum();
    let sum_xx: f64 = xs.iter().map(|x| x * x).sum();
    if sum_xx > 0.0 {
        let b = sum_xy / sum_xx;
        if b.is_finite() && b > 0.0 {
            let rss = residual_sum_of_squares(&xs, points, 0.0, b);
            if rss.is_finite() {
                return Some(BetaCandidate::new(beta, 0.0, b, rss));
            }
        }
    }

    None
}

/// The residual sum of squares `Σ(y_i − a − b x_i)²` for the given fit over the ladder's `x_i = N_i^β`.
fn residual_sum_of_squares(
    xs: &[f64; NUM_DOSES_USIZE],
    points: &[BlockPoint; NUM_DOSES_USIZE],
    a: f64,
    b: f64,
) -> f64 {
    xs.iter()
        .zip(points.iter())
        .map(|(x, point)| {
            let residual = point.y() - a - b * x;
            residual * residual
        })
        .sum()
}
