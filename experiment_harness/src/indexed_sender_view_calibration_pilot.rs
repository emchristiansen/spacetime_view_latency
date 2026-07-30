//! The `IndexedControlActivitySenderView` append-only E2 **method-calibration pilot** (spec
//! c33f2e51, "Control decision: candidate-specific calibration pilot").
//!
//! This module exists to answer one methodological question and nothing else: **what is the
//! smallest within-cell paced sample count `W` that a later 16-attempt matched-index screen can
//! honestly use?** It runs exactly two independent replicates at the baseline global rung, appends
//! at most [`MAX_PACED_SAMPLES`](calibration_params::MAX_PACED_SAMPLES) production-shaped single-row
//! rows to the measured identity's own slice, and retains every ordered raw nanosecond.
//!
//! **The calibration ceiling is structural, not documentary.** The spec forbids this pilot from
//! producing any performance, scaling, candidate, or site conclusion and any Arm/Control
//! comparison, and three properties of this vocabulary enforce that rather than asking a reader to
//! remember it:
//!
//! - [`CalibrationSeries`](calibration_series::CalibrationSeries) is the only type that can hold
//!   measured durations, and it exposes **no reduction whatsoever** — no median, mean, minimum, or
//!   maximum, and no per-sample numeric accessor — so no cell statistic can be computed through the
//!   ordinary API or added to it inattentively. It seals only from a
//!   [`VerifiedPopulation`](verified_population::VerifiedPopulation), a token minted solely by the
//!   row-by-row composition verifier, so a series measured against a same-cardinality substitution
//!   cannot be sealed at all.
//! - [`AttemptedOutcome`](attempted_outcome::AttemptedOutcome) has no `Complete` variant. Its
//!   success shape is `CalibrationRecorded`, so a ledger line cannot spell a completed measurement
//!   as a completed *result*.
//! - [`OutcomeCeiling`](outcome_ceiling::OutcomeCeiling) has exactly one variant and is written onto
//!   every record from a frozen constant, so no attempt can report running under a wider ceiling
//!   than the one the spec authorized.
//!
//! **The limit of that enforcement, stated plainly.** These are not capability boundaries. The
//! samples must serialize — retaining them is the pilot's entire purpose — and a serde round trip
//! inside this crate would recover the numbers, so deliberate serialization-and-parse arithmetic
//! remains possible and no type here can forbid it. What the vocabulary does guarantee is that the
//! easy path does not exist and cannot be added by accident, and that a reduction obtained by the
//! deliberate route would have no outcome variant, evidence type, or ledger field shaped to carry
//! it. The ceiling is ultimately held by what the record vocabulary can say, backed by review — not
//! by the type system alone.
//!
//! There is likewise **no Control role and no second candidate** in this vocabulary:
//! [`CandidateId`](candidate_id::CandidateId) has one variant and every identity is minted at
//! [`RunRole::Arm`](crate::plan::run_role::RunRole), so the forbidden Arm/Control comparison has no
//! values to be formed from.
//!
//! Structurally this is the [`crate::control_registry_discovery_screen`] record model applied to a
//! paced-append channel: a frozen inventory sealed before the first server is provisioned, one fresh
//! isolated instance per attempt behind the typed host gate, four closed failure stages onto five
//! honest terminal record shapes, and terminal-record-before-release settlement. Nothing here
//! extends the dormant `view_read_set_campaign` namespace; the axis ladder is not re-minted but read
//! from the Pilot's frozen [`GlobalRowRung`](crate::entity_owner_pilot::global_row_rung::GlobalRowRung).
//!
//! **The one genuine departure from the E3 record model is partial series retention.** E3's measured
//! unit is a single cold apply, so a failed measurement has no sample by construction and its
//! kind-to-sample mapping is a biconditional. A paced batch is up to a thousand samples, so a batch
//! that fails partway may genuinely hold some — see
//! [`FailureKind::requires_series`](failure_kind::FailureKind::requires_series) and
//! [`permits_series`](failure_kind::FailureKind::permits_series), which weaken that biconditional to
//! the pair of implications a many-sample channel can actually satisfy.
//!
//! Enums carry only variants this module can produce. A variant added later must not reinterpret
//! evidence already written under this vocabulary.
//!
//! One public entity per file; this entry file is declarative re-exports only.

pub(crate) mod attempt_failure;
pub(crate) mod attempt_inventory;
pub(crate) mod attempt_key;
pub(crate) mod attempt_ordinal;
pub(crate) mod attempt_stage;
pub(crate) mod attempted_outcome;
pub(crate) mod calibration_driver;
pub(crate) mod calibration_expectation;
pub(crate) mod calibration_params;
pub(crate) mod calibration_record;
pub(crate) mod calibration_replicate;
pub(crate) mod calibration_rung;
pub(crate) mod calibration_series;
pub(crate) mod calibration_target;
pub(crate) mod candidate_id;
pub(crate) mod candidate_version;
pub(crate) mod composition_mismatch;
pub(crate) mod diagnostic_artifact;
pub(crate) mod expected_population;
pub(crate) mod experiment_axis;
pub(crate) mod failure_kind;
pub(crate) mod failure_phase;
pub(crate) mod gate_outcome;
pub(crate) mod generated_tree_digest;
pub(crate) mod host_observations;
pub(crate) mod method_facts;
pub(crate) mod not_run_reason;
pub(crate) mod observed_row;
pub(crate) mod outcome_ceiling;
pub(crate) mod paced_sample_nanos;
pub(crate) mod partial_evidence;
pub(crate) mod partial_provision;
pub(crate) mod pinned_artifact_identity;
pub(crate) mod provision_depth;
pub(crate) mod rejected_series;
pub(crate) mod resource_disposition;
pub(crate) mod retry_eligibility;
pub(crate) mod sampling_progress;
pub(crate) mod stage_repetition;
pub(crate) mod verified_population;

pub(crate) use calibration_driver::indexed_sender_view_calibration_pilot;
