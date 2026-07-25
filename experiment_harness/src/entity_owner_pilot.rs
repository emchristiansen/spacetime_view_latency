//! Pilot stage for the `EntityOwnerSenderView` candidate (spec c33f2e51).
//!
//! The spec's Pilot is "five randomized matched blocks for plumbing, controls, and data adequacy
//! only"; it "never authorizes a performance conclusion". This module implements that stage for one
//! candidate: the frozen `N_global` ladder walked progressively on a fresh isolated server per
//! attempt, sender-view Arm against direct-public-table Control, with every attempt's terminal
//! disposition appended to a durable NDJSON ledger.
//!
//! **This candidate has no counterpart in the historical arm machinery.** That ontology (`Cell`,
//! `SeedOp`, `SubscribedTable`, `CampaignDataset`, `RecordKind`, `DoseObservation`,
//! `ObservationSink`) is fixed to the `Message`/`ChronicleMessage` mechanism study and is preserved,
//! not reinterpreted — see the spec's "Preserve the historical campaign model" and
//! [`crate::entity_owner_smoke`]. So this module mints the spec's own "Minimal type design" attempt
//! vocabulary ([`attempt_key::AttemptKey`], [`attempt_outcome::AttemptOutcome`]) and its own durable
//! sink, while reusing every ontology-free primitive underneath verbatim: provisioning
//! ([`crate::provision`]), the connected client, [`crate::observation::raw_latencies::RawLatencies`],
//! [`crate::observation::latency_summary::LatencySummary`], and the
//! [`crate::observation::durable_line_writer::DurableLineWriter`] /
//! [`crate::observation::file_line_writer::FileLineWriter`] durability seam.
//!
//! One public entity per file; this entry file is declarative re-exports only.

pub(crate) mod attempt_inventory;
pub(crate) mod attempt_key;
pub(crate) mod attempt_outcome;
pub(crate) mod candidate_id;
pub(crate) mod candidate_version;
pub(crate) mod confirmatory_block_index;
pub(crate) mod diagnostic_artifact;
pub(crate) mod evidence_artifact;
pub(crate) mod experiment_axis;
pub(crate) mod failure_kind;
pub(crate) mod global_row_rung;
pub(crate) mod method_validity;
pub(crate) mod not_run_reason;
pub(crate) mod partial_evidence;
pub(crate) mod pilot_block_index;
pub(crate) mod pilot_driver;
pub(crate) mod pilot_params;
pub(crate) mod pilot_record;
pub(crate) mod pilot_record_id;
pub(crate) mod pilot_record_kind;
pub(crate) mod pilot_sink;
pub(crate) mod retry_ordinal;
pub(crate) mod rung_evidence;
pub(crate) mod stage_repetition;

pub(crate) use pilot_driver::entity_owner_sender_view_pilot;
