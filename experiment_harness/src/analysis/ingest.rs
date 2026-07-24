//! The untrusted `Deserialize` boundary: DTOs mirroring the exact NDJSON wire shape.
//!
//! This is the crate's *only* `Deserialize` surface. The production record types
//! ([`ValidatedRunManifest`](crate::manifest::validated_run_manifest::ValidatedRunManifest),
//! [`DoseObservation`](crate::observation::dose_observation::DoseObservation), and every value they
//! embed) are `Serialize`-only and are never round-tripped, so an arbitrary artifact can never
//! masquerade as a value proven by the live typestate path. Each DTO here is a *purely syntactic*
//! mirror of one wire node:
//!
//! - Every field is the most permissive faithful primitive — `String` for every hex digest, version,
//!   path, socket address, and URL; integer primitives for counts; `Vec<u128>` for the raw latency
//!   array; a closed `enum` for each externally-tagged tag set. No hex decoding, semver parsing,
//!   range check, or nonzero check happens here.
//! - **All** semantic validation — hex/semver/identity parsing, cardinality/summary recomputation,
//!   completeness and cross-reference integrity — happens in exactly one downstream [`validate`] pass
//!   (`super::validate`), never scattered across the DTO layer.
//! - Every struct DTO is `#[serde(deny_unknown_fields)]`, and serde's derived struct visitor already
//!   rejects *duplicate* fields, so the closed wire contract is enforced structurally with no
//!   `serde_json::Value` staging that could silently collapse a duplicate or unknown field.
//!
//! The two line kinds are distinguished by the record's kind, not by an external tag on the line, so
//! the whole-line envelope [`WireRecordDto`] dispatches its body by [`RecordKindDto`] rather than by a
//! derived `Deserialize`.
//!
//! One public entity per file; this entry file is declarative re-exports only.

pub(crate) mod cell_dto;
pub(crate) mod chronicle_physical_rows_dto;
pub(crate) mod distribution_facts_dto;
pub(crate) mod dose_coordinate_dto;
pub(crate) mod dose_observation_dto;
pub(crate) mod event_evidence_dto;
pub(crate) mod ingest_error;
pub(crate) mod key_scoped_arm_dto;
pub(crate) mod latency_summary_dto;
pub(crate) mod manifest_body_dto;
pub(crate) mod manifest_reference_dto;
pub(crate) mod message_physical_rows_dto;
pub(crate) mod module_facts_dto;
pub(crate) mod observation_body_dto;
pub(crate) mod parse_ndjson;
pub(crate) mod physical_cardinalities_dto;
pub(crate) mod preregistered_parameters_dto;
pub(crate) mod record_id_dto;
pub(crate) mod record_kind_dto;
pub(crate) mod role_identities_dto;
pub(crate) mod run_coordinate_dto;
pub(crate) mod run_role_dto;
pub(crate) mod server_facts_dto;
pub(crate) mod table_scoped_arm_dto;
pub(crate) mod validated_run_manifest_dto;
pub(crate) mod wire_record_dto;
