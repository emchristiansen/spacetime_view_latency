//! One attempt's recorded paced series — the pilot's entire admissible output.

use anyhow::{ensure, Result};
use serde::Serialize;

use crate::indexed_sender_view_calibration_pilot::calibration_params::MAX_PACED_SAMPLES_USIZE;
use crate::indexed_sender_view_calibration_pilot::paced_sample_nanos::PacedSampleNanos;
use crate::indexed_sender_view_calibration_pilot::verified_population::VerifiedPopulation;

/// The complete ordered raw latencies of one calibration attempt, sealed only if every sample is
/// usable *and* the composition it was measured against held.
///
/// **This type deliberately exposes no reduction.** There is no median, no mean, no minimum, no
/// maximum, no percentile, no slice accessor, and no per-sample numeric accessor — only a count and
/// `Serialize`. A reduction over these samples would be a cell statistic, a cell statistic is a
/// performance claim, and the spec authorizes no performance claim from this pilot.
///
/// That absence removes the accidental and the convenient path, **not every path**: `Serialize` is
/// required so the evidence reaches the ledger, and a serde round trip inside this crate would
/// recover the numbers. The honest invariant is narrower than "impossible" — no reduction API exists
/// or can be added inattentively, and no outcome or ledger field is shaped like a result for one to
/// be reported through. See
/// [`PacedSampleNanos`](super::paced_sample_nanos::PacedSampleNanos) for the full statement.
///
/// Contrast [`ColdApplyEvidence`](crate::control_registry_discovery_screen::cold_apply_evidence::ColdApplyEvidence),
/// which seals exactly the duration that *is* its cell statistic. The two types face opposite ways
/// on purpose: that one exists so a measurement can become evidence, this one exists so a
/// measurement cannot become a result.
///
/// **Order is the evidence.** The samples are retained in issue order because every diagnostic the
/// spec's decision rule names — append-series stationarity and trend, lag dependence,
/// position-aware contiguous-window median stability — is a statement about position in the series.
/// A set of latencies would answer none of them.
///
/// **No `Debug`**, inherited from [`PacedSampleNanos`]: a derived rendering here would recover
/// through this type exactly what that one withholds. The same omission carries up through every
/// containing type.
#[derive(Clone, Serialize)]
pub(crate) struct CalibrationSeries {
    samples: Vec<PacedSampleNanos>,
    population: VerifiedPopulation,
}

impl CalibrationSeries {
    /// Seal one attempt's series, failing loud unless it is **complete**, every sample is usable,
    /// and the composition it was measured against matched its frozen expectation.
    ///
    /// Four refusals:
    ///
    /// - **Not exactly the frozen count.** The spec's post-pilot decision rule evaluates every
    ///   candidate `W` in `{10, 20, 50, 100, 200, 500, 1000}` against **both complete retained
    ///   series**, so a series of any other length cannot serve as a calibration replicate at all: a
    ///   short one cannot answer the 1,000 case, and a long one is evidence from a method other than
    ///   the one that was frozen. `W_max` means an attempt may *fail* before reaching a thousand
    ///   without fabricating samples — never that a shorter attempt succeeded.
    /// - **Nonpositive sample.** The spec invalidates a cell on a missing, non-finite, or
    ///   nonpositive statistic rather than counting it as flat.
    /// - **Append count disagreement.** The verified population must have been proven against
    ///   exactly as many appends as there are samples, and that must be the frozen count.
    ///
    /// **There is no composition parameter to get wrong.** The `population` argument is a
    /// [`VerifiedPopulation`], a token only
    /// [`VerifiedPopulation::verify`](super::verified_population::VerifiedPopulation::verify) can
    /// mint, and it mints one only when every row in both caches matched its preregistered key
    /// range, owner, control, and derived timestamp. So a series cannot be sealed against an
    /// unvalidated — or same-cardinality-substituted — population at all: the caller has nothing to
    /// pass. That is why this constructor checks counts rather than re-checking composition.
    ///
    /// Every rejected case keeps its samples in issue order as a
    /// [`RejectedSeries`](super::rejected_series::RejectedSeries) on a `Sample` or `Semantics`
    /// failure. Nothing is discarded — a partial series is exactly as informative about stationarity
    /// and lag as a complete one, up to where it stops. What it cannot do is stand in for a
    /// replicate.
    ///
    /// **This is what stops the pilot being called complete on a short series.** The only success
    /// shape, [`AttemptedOutcome::CalibrationRecorded`](super::attempted_outcome::AttemptedOutcome),
    /// holds a `CalibrationSeries` and nothing else, so an attempt that stopped early can reach the
    /// ledger only as a failure.
    pub(crate) fn recorded(
        samples: Vec<PacedSampleNanos>,
        population: VerifiedPopulation,
    ) -> Result<Self> {
        ensure!(
            samples.len() == MAX_PACED_SAMPLES_USIZE,
            "a calibration replicate is exactly the frozen {MAX_PACED_SAMPLES_USIZE} samples, got \
             {}; a shorter run is a failed attempt whose samples are retained as a rejected series, \
             not a successful replicate",
            samples.len(),
        );
        if let Some(position) = samples.iter().position(|sample| !sample.is_positive()) {
            anyhow::bail!(
                "sample {position} is not strictly positive, so this series invalidates its own \
                 attempt rather than counting as flat"
            );
        }
        let verified_appends = population.verified_appends();
        ensure!(
            verified_appends == MAX_PACED_SAMPLES_USIZE as u64,
            "this series is complete, so its population must have been verified against all \
             {MAX_PACED_SAMPLES_USIZE} appends, not {verified_appends}",
        );
        Ok(Self {
            samples,
            population,
        })
    }

    /// How many samples this attempt recorded — always exactly the frozen count.
    ///
    /// A count, never a statistic: it says how much evidence exists without saying anything about
    /// what it measured. Invariant by the constructor above, and retained so a ledger-only reader
    /// can confirm completeness without counting the serialized array.
    pub(crate) fn len(&self) -> usize {
        self.samples.len()
    }
}
