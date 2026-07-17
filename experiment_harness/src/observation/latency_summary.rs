//! The per-dose median and IQR, derived from the lossless raw latencies.

use serde::Serialize;

use crate::observation::raw_latencies::RawLatencies;

/// The per-dose summary statistics reported alongside the raw vector: the median and interquartile
/// range of the confirmed round-trip latencies, in nanoseconds.
///
/// The sole constructor is [`Self::from_raw`], so construction is centralized and the summary is
/// derived from its dose's [`RawLatencies`] at record assembly rather than supplied independently.
///
/// **Quantile convention — R-1 (nearest-rank / inverse empirical CDF).** For the ascending sorted
/// one-based samples `x_(1..n)` and a probability `p ∈ (0,1)`, the quantile is
/// `Q(p) = x_(⌈n·p⌉)` (the rank clamped to `[1, n]`), and `IQR = Q(0.75) − Q(0.25)`. This is the
/// convention recorded in the spec's Implementation-Time Decision "Use R-1 nearest-rank per-dose
/// summaries": every dose holds exactly [`BATCH_SIZE`](crate::params::BATCH_SIZE) integer-nanosecond
/// samples, so R-1 is **sample-valued and integer-exact** — it introduces neither interpolation nor
/// any nanosecond rounding rule nor an IQR order-of-operations choice. This doc comment is the
/// schema documentation of that convention; the lossless [`RawLatencies`] vector remains the
/// authoritative evidence, so any later convention can be recomputed from it. The derivation is
/// pinned by hand-computed odd-, even-, and 1,000-sample tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct LatencySummary {
    median_nanos: u128,
    iqr_nanos: u128,
}

impl LatencySummary {
    /// Derive the median and IQR from the raw latencies under the R-1 nearest-rank convention (see
    /// the type docs) — the only way to build a summary. The samples are copied out and sorted
    /// ascending (the raw vector itself stays in issue order), then `Q(0.5)`, `Q(0.25)`, and
    /// `Q(0.75)` are read at their nearest ranks.
    pub(crate) fn from_raw(raw: &RawLatencies) -> Self {
        let mut sorted: Vec<u128> = raw.samples().iter().map(|s| s.nanos()).collect();
        sorted.sort_unstable();

        let median_nanos = nearest_rank(&sorted, 1, 2);
        let q1 = nearest_rank(&sorted, 1, 4);
        let q3 = nearest_rank(&sorted, 3, 4);
        // Q3 is at rank ⌈3n/4⌉ ≥ ⌈n/4⌉ over the same ascending sort, so Q3 ≥ Q1 and the difference
        // never underflows; a checked subtraction makes that invariant loud rather than assumed.
        let iqr_nanos = q3.checked_sub(q1).expect(
            "R-1 Q(0.75) is at a rank ≥ Q(0.25)'s over the same ascending sort, so Q3 ≥ Q1",
        );
        Self {
            median_nanos,
            iqr_nanos,
        }
    }

    /// The median round-trip latency in nanoseconds.
    pub(crate) fn median_nanos(self) -> u128 {
        self.median_nanos
    }

    /// The interquartile range of round-trip latencies in nanoseconds.
    pub(crate) fn iqr_nanos(self) -> u128 {
        self.iqr_nanos
    }

    /// The summary of a dose whose samples are all exactly 1 nanosecond: median 1, IQR 0. Used to
    /// assemble a whole coherent [`DoseObservation`] fixture — the value is chosen to agree with
    /// that fixture's identical 1 ns samples (and equals what [`Self::from_raw`] derives from them),
    /// not to exercise any real derivation. Test-only.
    #[cfg(test)]
    pub(crate) fn fixture() -> Self {
        Self {
            median_nanos: 1,
            iqr_nanos: 0,
        }
    }
}

/// R-1 nearest-rank quantile: for ascending sorted one-based samples `x_(1..n)` and probability
/// `p = num/den ∈ (0,1)`, `Q(p) = x_(⌈n·p⌉)` with the rank clamped to `[1, n]`. Integer-exact and
/// sample-valued — no interpolation, no rounding. `⌈n·num/den⌉` is computed with integer
/// arithmetic as `(n·num + den − 1) / den`. `sorted` must be nonempty and sorted ascending.
fn nearest_rank(sorted: &[u128], num: u128, den: u128) -> u128 {
    let n = u128::try_from(sorted.len()).expect("a dose's sample count fits u128");
    let rank = (n * num + den - 1) / den;
    let rank = rank.clamp(1, n);
    sorted[usize::try_from(rank - 1).expect("a clamped 1..=n rank fits usize")]
}

#[cfg(test)]
mod tests;
