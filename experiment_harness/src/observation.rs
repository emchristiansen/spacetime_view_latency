//! Lossless, machine-readable per-dose observation records and their required durable NDJSON sink.
//!
//! An observation is a per-dose record ([`dose_observation::DoseObservation`]) carrying a reference
//! to the immutable run manifest, the schedule/ladder coordinate, both physical cardinalities, the
//! lossless raw latency vector and its derived summary, and the SDK logical event evidence. Records
//! are appended to one required output file ([`output_path::OutputPath`]) through a durable sink
//! ([`observation_sink::ObservationSink`]) that assigns each a global [`record_seq::RecordSeq`],
//! drives a [`durable_line_writer::DurableLineWriter`] for the actual I/O, and becomes terminal on
//! any persist failure so a torn or ambiguous tail can never be silently extended.
//!
//! These types prove *structure*: record schema, cardinality, sequence identity, and the sink's
//! state transitions. They do not — and no Phase-1 type can — establish that samples were actually
//! measured, that events were truly delivered, or that a line is physically crash-durable; those are
//! effect provenance for the measurement milestone.
//!
//! One public entity per file; this entry file is declarative re-exports only.

pub(crate) mod dose_coordinate;
pub(crate) mod dose_evidence;
pub(crate) mod dose_latency_accumulator;
pub(crate) mod dose_observation;
pub(crate) mod durable_line_writer;
pub(crate) mod event_evidence;
pub(crate) mod file_line_writer;
pub(crate) mod final_sync_failures;
pub(crate) mod finalize_error;
pub(crate) mod latency_sample;
pub(crate) mod latency_summary;
pub(crate) mod manifest_reference;
pub(crate) mod observation_sink;
pub(crate) mod output_path;
pub(crate) mod persist_error;
pub(crate) mod poisoned_tail;
pub(crate) mod raw_latencies;
pub(crate) mod record_id;
pub(crate) mod record_kind;
pub(crate) mod record_seq;
pub(crate) mod sink_create_error;
pub(crate) mod tail_state;
