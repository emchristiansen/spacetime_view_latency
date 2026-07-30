//! The §569 analyzer for the `IndexedControlActivitySenderView` append-only E2 calibration pilot
//! (spec c33f2e51).
//!
//! This module reads a calibration ledger and emits **threshold-free diagnostics** for Control to
//! freeze `W`. It provisions nothing, measures nothing, and mutates nothing: it opens one file for
//! reading and writes one JSON document to stdout.
//!
//! **The ceiling is structural, not documentary.** No type in this namespace has a field for a
//! verdict, a recommendation, a chosen `W`, a pass/fail, a threshold, a score, or a cross-candidate
//! comparison. §569's rule — prefer the smallest count whose position-aware window medians are
//! stable enough in both attempts, weighing autocorrelation, early/late bias, worst-window
//! behaviour, and between-attempt disagreement together — is applied by **Control in SSOT**, not
//! here. This module's whole job is to state exactly what the two series did.
//!
//! **The semantic boundary, stated as what may and may not be imported.** The pure generic numeric
//! primitives [`Rational`](crate::analysis::stats::rational::Rational),
//! [`median`](crate::analysis::stats::median::median), and
//! [`FiniteF64`](crate::analysis::finite_f64::FiniteF64) are reused rather than duplicated —
//! reimplementing exact rational arithmetic for this analyzer would be a second copy of a checked,
//! overflow-loud contract. Campaign **ingest**, **report**, and **domain** types are not imported at
//! all: `CellEvidence`, the `*Dto` wire records, the `*Report` projections, and the cell/dose/block
//! vocabulary carry campaign semantics this pilot must not inherit. Every DTO, admission type, and
//! report type here is its own.
//!
//! **The sealed pilot API is preserved.** The ledger is parsed through this module's own DTOs. No
//! accessor was added to [`PacedSampleNanos`](crate::indexed_sender_view_calibration_pilot::paced_sample_nanos::PacedSampleNanos),
//! [`CalibrationSeries`](crate::indexed_sender_view_calibration_pilot::calibration_series::CalibrationSeries),
//! or any other pilot type. This is the deliberate serde route the pilot documents as remaining open,
//! taken now for the stated §569 purpose that module says such parsing code would otherwise lack.
//!
//! **Re-admission reconstructs the pilot's whole predicate, because the token did not survive the
//! wire.** [`VerifiedPopulation`](crate::indexed_sender_view_calibration_pilot::verified_population::VerifiedPopulation)
//! is a capability minted only by the row-by-row verifier, and what reaches the ledger is three
//! plain integers. So [`CompleteReplicate`](complete_replicate::CompleteReplicate) re-derives every
//! serialized fact a complete series depends on — frozen method, frozen attempt identity including
//! the `Arm` role, `Attempted`/`CalibrationRecorded` shape, exact sample count, strict positivity,
//! and all three population counts. A field this analyzer declines to decode is a field nothing
//! checks, so every key component is mirrored and matched.
//!
//! **The unit of admission is the whole ledger, not the line.**
//! [`ReplicatePair`](replicate_pair::ReplicatePair) is constructed from a
//! [`LedgerAdmission`](ledger_admission::LedgerAdmission), which carries the refusals together with
//! the admissions, and it refuses outright if any line was refused before requiring exactly the
//! frozen inventory's ordinal set. So two perfect originals accompanied by a third line — a failed
//! attempt, a `Control` record, a retry-shaped duplicate — produce no report rather than a report
//! that quietly ignores the third.
//!
//! **The lag set is derived, not preregistered.** §568/§569/§615 name "lag dependence/effective
//! information" and "autocorrelation" without fixing a `k`, so [`lag_domain`] derives the domain
//! `1 ..= (largest candidate window − 1)` from [`CandidateWindow::ALL`] — the smallest domain that
//! reaches within-window dependence for *every* candidate, not just the smallest one. Each element
//! publishes its own pair count, so sparse tails are visible rather than censored, and nothing is
//! summed into an effective sample size, which would read as an answer.
//!
//! [`CandidateWindow::ALL`]: candidate_window::CandidateWindow::ALL
//!
//! One public entity per file; this entry file is declarative re-exports only.

pub(crate) mod absolute_value;
pub(crate) mod admission_refusal;
pub(crate) mod analyze_calibration;
pub(crate) mod attempt_key_dto;
pub(crate) mod attempt_ordinal_dto;
pub(crate) mod attempted_outcome_dto;
pub(crate) mod between_replicate_report;
pub(crate) mod calibration_analysis_error;
pub(crate) mod calibration_diagnostics_report;
pub(crate) mod calibration_ingest_error;
pub(crate) mod calibration_record_dto;
pub(crate) mod calibration_rung_dto;
pub(crate) mod calibration_series_dto;
pub(crate) mod candidate_id_dto;
pub(crate) mod candidate_window;
pub(crate) mod complete_replicate;
pub(crate) mod correlation_coefficient;
pub(crate) mod exact_rational_report;
pub(crate) mod exact_samples;
pub(crate) mod experiment_axis_dto;
pub(crate) mod frozen_candidate_version;
pub(crate) mod frozen_population;
pub(crate) mod frozen_replicate_ordinals;
pub(crate) mod lag_autocorrelation;
pub(crate) mod lag_domain;
pub(crate) mod ledger_admission;
/// Hand-built wire lines shared by this namespace's pure tests. Test-only, so nothing here widens
/// the analyzer's real API.
#[cfg(test)]
pub(crate) mod ledger_fixture;
pub(crate) mod measurement_channel_dto;
pub(crate) mod method_facts_dto;
pub(crate) mod normalized_autocorrelation;
pub(crate) mod outcome_ceiling_dto;
pub(crate) mod pair_refusal;
pub(crate) mod parse_calibration_ndjson;
pub(crate) mod positioned_difference_report;
pub(crate) mod positioned_median_report;
pub(crate) mod refused_line;
pub(crate) mod replicate_diagnostics_report;
pub(crate) mod replicate_pair;
pub(crate) mod run_role_dto;
pub(crate) mod stage_repetition_dto;
pub(crate) mod trend_report;
pub(crate) mod verified_population_dto;
pub(crate) mod window_diagnostics_report;
pub(crate) mod window_median_series;
pub(crate) mod window_stability_report;

pub(crate) use analyze_calibration::analyze_calibration;
