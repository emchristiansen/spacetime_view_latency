//! One lag's dependence diagnostic, with its exact components retained.

use serde::Serialize;

use crate::analysis::stats::rational::Rational;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::correlation_coefficient::CorrelationCoefficient;
use crate::indexed_sender_view_calibration_analysis::exact_rational_report::ExactRationalReport;
use crate::indexed_sender_view_calibration_analysis::normalized_autocorrelation::NormalizedAutocorrelation;

/// One replicate's dependence at **one** lag `k`, reported as its exact components plus a normalized
/// coefficient — never as the coefficient alone.
///
/// §569 weighs autocorrelation, and a bare `r = 0.31` is not reviewable: it hides which normalization
/// produced it, and a reader who disagrees with that choice has no way to recompute. So the
/// definition is laid out in the artifact itself. With `x̄` the mean of the whole series, `n` its
/// length, and `m = n − k` the number of lag-`k` pairs:
///
/// - `numerator = Σ_{i=0}^{m-1} (xᵢ − x̄)(xᵢ₊ₖ − x̄)` — the lag-`k` autocovariance numerator;
/// - `left_energy = Σ_{i=0}^{m-1} (xᵢ − x̄)²` — the `m` terms that pair as `xᵢ`;
/// - `right_energy = Σ_{i=k}^{n-1} (xᵢ − x̄)²` — the `m` terms that pair as `xᵢ₊ₖ`;
/// - `normalized = numerator / √(left_energy · right_energy)`.
///
/// All three components are **exact rationals**, and the mean is exact too — the samples are
/// integers, so `x̄` is a rational and every centred term stays rational. Only the final ratio is
/// floating point, because the square root has no exact rational form.
///
/// **The two energies are retained separately rather than collapsed**, which is what makes the
/// definition checkable. This is the Cauchy–Schwarz normalization, so `|normalized| ≤ 1` always
/// holds; the more common same-series convention divides by the *full* `Σ(xᵢ − x̄)²` instead and would
/// give a slightly different number. Publishing both energies lets a reader see which was used and
/// recompute the other, rather than having to trust a label.
///
/// **This type is one lag, not the whole diagnostic.** §568 asks for "lag dependence/effective
/// information" and §569 for "autocorrelation" — neither names a lag, and lag-1 alone cannot speak to
/// effective information, which is a statement about dependence across lags. Which lags are computed
/// is a separate, explicitly derived decision recorded by
/// [`lag_domain`](super::lag_domain::lag_domain).
///
/// The `pairs` count travels with each element for exactly that reason: at the far end of the domain
/// a coefficient rests on very few pairs, and a reader must be able to see that rather than infer it.
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) struct LagAutocorrelation {
    /// Which lag `k` this element is for.
    lag: usize,
    /// How many lag-`k` pairs the series contains — `n − k`.
    pairs: usize,
    /// The exact lag-`k` autocovariance numerator.
    numerator: ExactRationalReport,
    /// The exact energy of the `m` leading terms.
    left_energy: ExactRationalReport,
    /// The exact energy of the `m` trailing terms.
    right_energy: ExactRationalReport,
    /// The normalized coefficient, or the explicit reason it is undefined.
    normalized: NormalizedAutocorrelation,
}

impl LagAutocorrelation {
    /// Compute one replicate's dependence at lag `k`, over its whole series in issue order.
    ///
    /// Over the *whole* series and never per candidate window: dependence is a property of the run.
    pub(crate) fn of(replicate: &CompleteReplicate, lag: usize) -> Self {
        let samples: Vec<i128> = replicate
            .samples()
            .iter()
            .map(|sample| {
                i128::try_from(*sample).expect(OUTSIDE_THE_EXACT_DOMAIN)
            })
            .collect();
        let length = samples.len();
        assert!(
            (1..length).contains(&lag),
            "lag {lag} has no pairs in a series of {length} samples"
        );
        let pairs = length - lag;

        // Centring is done on an integer scale rather than through `Rational`, and the two agree
        // exactly. With `S = Σxᵢ` and `n` the length, `scaledᵢ = n·xᵢ − S` is exactly `n·(xᵢ − x̄)`,
        // so every product below is `n²` times the corresponding centred product. That common factor
        // is divided out once, at the end, when each sum becomes a rational — instead of dragging a
        // thousand-denominator rational through several million multiplications and gcd reductions.
        let count = i128::try_from(length).expect("the frozen sample count fits i128");
        // Every step below is checked and fails loud rather than wrapping. No step claims a ceiling
        // derived from the measurement method — see `OUTSIDE_THE_EXACT_DOMAIN`.
        let total = samples
            .iter()
            .try_fold(0i128, |sum, sample| sum.checked_add(*sample))
            .expect(OUTSIDE_THE_EXACT_DOMAIN);
        let scaled: Vec<i128> = samples
            .iter()
            .map(|sample| {
                count
                    .checked_mul(*sample)
                    .and_then(|scaled| scaled.checked_sub(total))
                    .expect(OUTSIDE_THE_EXACT_DOMAIN)
            })
            .collect();

        let numerator_scaled = sum_of_products(&scaled, &scaled[lag..], pairs);
        let left_scaled = sum_of_products(&scaled, &scaled, pairs);
        let right_scaled = sum_of_products(&scaled[lag..], &scaled[lag..], pairs);

        // Undo the `n²` scaling exactly, so the published components are the real centred sums.
        let square = count
            .checked_mul(count)
            .expect("the squared frozen sample count fits i128");
        let numerator = Rational::new(numerator_scaled, square);
        let left = Rational::new(left_scaled, square);
        let right = Rational::new(right_scaled, square);

        // A zero energy on either side is a genuine `0/0`, not a zero coefficient. The common `n²`
        // factor cancels in the ratio, so it is computed from the scaled sums directly.
        let normalized = match left_scaled == 0 || right_scaled == 0 {
            true => NormalizedAutocorrelation::UndefinedZeroVariance,
            false => {
                let denominator = (left_scaled as f64 * right_scaled as f64).sqrt();
                NormalizedAutocorrelation::Defined {
                    coefficient: CorrelationCoefficient::from_rounded_ratio(
                        numerator_scaled as f64 / denominator,
                    ),
                }
            }
        };

        Self {
            lag,
            pairs,
            numerator: ExactRationalReport::of(numerator),
            left_energy: ExactRationalReport::of(left),
            right_energy: ExactRationalReport::of(right),
            normalized,
        }
    }
}

/// `Σ_{i<count} left[i] · right[i]`, checked at every step.
///
/// Both slices are integer-scaled centred values, so each product is `n²` times a centred product
/// and the whole sum carries that one common factor out to the caller.
fn sum_of_products(left: &[i128], right: &[i128], count: usize) -> i128 {
    (0..count)
        .try_fold(0i128, |sum, index| {
            left[index]
                .checked_mul(right[index])
                .and_then(|product| sum.checked_add(product))
        })
        .expect(OUTSIDE_THE_EXACT_DOMAIN)
}

/// Why a checked step in the centring arithmetic failing is a crash rather than a recoverable case.
///
/// **The domain of this exact report is what `i128` can represent, and nothing narrower.** No
/// ceiling on a sample's magnitude is claimed here, and none is derived from the measurement method.
/// If a sample, or any centred sum built from it, leaves `i128`, that series is outside what this
/// report can state exactly, and the only honest response is to fail loudly rather than wrap a
/// meaningless value into a plausible-looking coefficient.
///
/// One message serves every checked step, because they all report the same fact: the value or the
/// arithmetic left `i128`.
const OUTSIDE_THE_EXACT_DOMAIN: &str =
    "a sample or centred sum left i128, so this series is outside the domain this report can state \
     exactly; wrapping would yield a plausible-looking but meaningless coefficient";

#[cfg(test)]
mod tests;
