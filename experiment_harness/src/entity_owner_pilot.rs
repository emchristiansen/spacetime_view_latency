//! Pilot stage for the `EntityOwnerSenderView` candidate (spec c33f2e51).
//!
//! This candidate has no counterpart in the historical arm machinery. That ontology (`Cell`,
//! `SeedOp`, `SubscribedTable`, `CampaignDataset`, `RecordKind`, `DoseObservation`,
//! `ObservationSink`) is fixed to the `Message`/`ChronicleMessage` mechanism study and is preserved,
//! not reinterpreted — see [`crate::entity_owner_smoke`]. So this module mints its own attempt
//! vocabulary and durable ledger, while reusing every ontology-free primitive underneath verbatim:
//! [`crate::provision`], [`crate::client::connected_client`],
//! [`crate::observation::raw_latencies::RawLatencies`],
//! [`crate::observation::latency_summary::LatencySummary`], and the
//! [`crate::observation::durable_line_writer::DurableLineWriter`] /
//! [`crate::observation::file_line_writer::FileLineWriter`] durability seam.
//!
//! Enums here carry only variants this module can actually produce. A variant added later must not
//! reinterpret evidence already written under this vocabulary.
//!
//! One public entity per file; this entry file is declarative re-exports only.

pub(crate) mod attempt_inventory;
pub(crate) mod attempt_key;
pub(crate) mod attempt_outcome;
pub(crate) mod attempt_provenance;
pub(crate) mod campaign_provenance;
pub(crate) mod candidate_id;
pub(crate) mod candidate_version;
pub(crate) mod development_build;
pub(crate) mod diagnostic_artifact;
pub(crate) mod evidence_artifact;
pub(crate) mod experiment_axis;
pub(crate) mod failure_kind;
pub(crate) mod global_row_rung;
pub(crate) mod harness_build_provenance;
pub(crate) mod method_validity;
pub(crate) mod not_run_reason;
pub(crate) mod partial_evidence;
pub(crate) mod pilot_block_index;
pub(crate) mod pilot_driver;
pub(crate) mod pilot_parameters;
pub(crate) mod pilot_params;
pub(crate) mod pilot_record;
pub(crate) mod pilot_sink;
pub(crate) mod retry_ordinal;
pub(crate) mod rung_evidence;
pub(crate) mod stage_repetition;

pub(crate) use pilot_driver::entity_owner_sender_view_pilot;
