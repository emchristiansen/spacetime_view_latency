//! Total integrity validation: fold the untrusted wire records into the trusted campaign graph.
//!
//! Exactly one pass turns the full `Vec<`[`WireRecordDto`](crate::analysis::ingest::wire_record_dto::WireRecordDto)`>`
//! produced by [`ingest`](crate::analysis::ingest) into a trusted
//! [`ValidatedCampaign`](validated_campaign::ValidatedCampaign), or fails loud with a typed
//! [`IntegrityError`](integrity_error::IntegrityError). Nothing partially valid escapes: the graph is
//! constructed only after every completeness and integrity obligation holds, so statistical code can
//! never observe a structurally invalid campaign.
//!
//! The trusted graph is a fixed-cardinality tree whose shape encodes the preregistered structure as
//! far as Rust allows:
//!
//! ```text
//! ValidatedCampaign            one seed + preregistered parameters, the nine preregistered cells
//! └── CellDataset              one Cell, exactly REPETITION_BLOCKS (30) matched blocks
//!     └── MatchedBlock         one block index, its arm run and its matched control run
//!         └── TrustedRun       one RunCoordinate, exactly NUM_DOSES (10) doses
//!             └── TrustedDose  one dose: raw latencies, summary, cardinalities, event evidence
//! ```
//!
//! Every leaf reuses the production trusted content types verbatim
//! ([`RunCoordinate`](crate::manifest::run_coordinate::RunCoordinate),
//! [`RawLatencies`](crate::observation::raw_latencies::RawLatencies),
//! [`LatencySummary`](crate::observation::latency_summary::LatencySummary),
//! [`PhysicalCardinalities`](crate::dataset::physical_cardinalities::PhysicalCardinalities),
//! [`EventEvidence`](crate::observation::event_evidence::EventEvidence)) — never their untrusted DTO
//! mirrors — so a value that reaches the graph carries analysis-domain meaning, not merely wire syntax.
//!
//! One public entity per file; this entry file is declarative re-exports only.

pub(crate) mod arm_run;
pub(crate) mod cell_dataset;
pub(crate) mod control_run;
pub(crate) mod integrity_error;
pub(crate) mod integrity_error_category;
pub(crate) mod matched_block;
pub(crate) mod run_kind;
pub(crate) mod trusted_dose;
pub(crate) mod trusted_run;
pub(crate) mod validated_campaign;
