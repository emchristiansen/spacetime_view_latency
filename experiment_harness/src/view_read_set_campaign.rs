//! Fresh-server campaign for the revised view read-set contract (spec c33f2e51).
//!
//! The completed seed-7 Pilot in [`crate::entity_owner_pilot`] measured a *cumulative* ladder walk
//! on one server, and its immutable ledger serializes that vocabulary — a flat axis inside
//! `AttemptKey`, and rung evidence carrying a per-rung increment. The revised protocol provisions a
//! fresh server per scale point and holds cardinality fixed while it measures, so its identities and
//! evidence are a different shape. Generalizing those types in place would reinterpret evidence that
//! is already on disk, so this module mints its own vocabulary instead — the same move
//! `entity_owner_pilot` itself made relative to the historical `Cell`/`SeedOp`/`CampaignDataset`
//! ontology, which remains untouched.
//!
//! Everything ontology-free underneath is reused verbatim rather than duplicated:
//! [`crate::provision`], [`crate::client::connected_client`],
//! [`crate::observation::raw_latencies::RawLatencies`],
//! [`crate::observation::latency_summary::LatencySummary`], the
//! [`crate::observation::durable_line_writer::DurableLineWriter`] /
//! [`crate::observation::file_line_writer::FileLineWriter`] durability seam, and the exact-rational
//! statistics in [`crate::analysis::stats`]. The ontology-*bearing* half of `crate::analysis` — the
//! dose ladder, `ResponseClass`, `BetaLabel`, and the fixed 95% thirty-block interval — cannot
//! implement this contract and is not reused.
//!
//! Enums here carry only variants this module can actually produce. A later candidate, axis, stage,
//! or reason arrives together with the driver that can execute it, and must not reinterpret evidence
//! already written under this vocabulary.
//!
//! One public entity per file; this entry file is declarative re-exports only.

pub(crate) mod attempt_inventory;
pub(crate) mod attempt_key;
pub(crate) mod attempt_outcome;
pub(crate) mod attempt_provenance;
pub(crate) mod axis_ladder;
pub(crate) mod campaign_driver;
pub(crate) mod campaign_ledger_line;
pub(crate) mod campaign_parameters;
pub(crate) mod campaign_params;
pub(crate) mod campaign_provenance;
pub(crate) mod campaign_record;
pub(crate) mod campaign_sink;
pub(crate) mod candidate_id;
pub(crate) mod candidate_version;
pub(crate) mod cell_statistic;
pub(crate) mod channel_evidence;
pub(crate) mod composition_validation;
pub(crate) mod diagnostic_artifact;
pub(crate) mod environment_gate_evidence;
pub(crate) mod environment_sample;
pub(crate) mod evidence_artifact;
pub(crate) mod experiment_axis;
pub(crate) mod failed_environment_gate;
pub(crate) mod failure_kind;
pub(crate) mod failure_stage;
pub(crate) mod infrastructure_phase;
pub(crate) mod measured_sample_boundary;
pub(crate) mod measurement_channel;
pub(crate) mod method_supersession;
pub(crate) mod method_validity;
pub(crate) mod mutation_schedule;
pub(crate) mod not_run_reason;
pub(crate) mod observed_runtime_pins;
pub(crate) mod partial_evidence;
pub(crate) mod passed_environment_gate;
pub(crate) mod pilot_block_index;
pub(crate) mod reconciled_campaign;
pub(crate) mod retry_eligibility;
pub(crate) mod retry_ordinal;
pub(crate) mod saturated_timing_batch;
pub(crate) mod saturated_write_timing;
pub(crate) mod scale_point;
pub(crate) mod scale_point_evidence;
pub(crate) mod stage_repetition;
pub(crate) mod superseded_scope;
pub(crate) mod supersession_justification;
pub(crate) mod terminal_attempt_record;
pub(crate) mod unrelated_global_rows_ladder_evidence;

pub(crate) use campaign_driver::view_read_set_campaign_pilot;
