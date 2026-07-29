//! The `ControlRegistry` discovery E3 screen (spec c33f2e51, "Site 4 ControlRegistry discovery E3
//! attempt-inventory freeze").
//!
//! A **descriptive screen**, never a disposition experiment: it freezes no decision boundary and no
//! block aggregate, and its only permitted performance outcome is `Indeterminate(ScreenOnly)`. It
//! cannot close `ControlRegistry` and authorizes no E2.
//!
//! Structurally this is the [`crate::entity_owner_pilot`] sealed-inventory vocabulary applied to the
//! [`crate::entity_owner_visible_rows_probe`] screen shape: attempts are frozen and sealed before
//! the first server is provisioned, then each runs on its own fresh isolated instance behind the
//! host gate. Nothing here extends the dormant `view_read_set_campaign` namespace — the campaign's
//! orchestrator, ladder aggregates, and ledger machinery are neither imported nor generalized, which
//! is what the spec's "a narrow site-specific probe module satisfies the rule without touching the
//! dormant namespace" requires.
//!
//! The one type this module deliberately does *not* mint is the axis ladder. Rungs are addressed
//! only through [`crate::entity_owner_pilot::global_row_rung::GlobalRowRung`], whose sole
//! constructor is its frozen `ALL` table, so a rung off the existing unrelated/global ladder is
//! unrepresentable here rather than merely unwritten.
//!
//! Enums carry only variants this module can actually produce. A variant added later must not
//! reinterpret evidence already written under this vocabulary.
//!
//! One public entity per file; this entry file is declarative re-exports only.

pub(crate) mod attempt_failure;
pub(crate) mod attempt_inventory;
pub(crate) mod attempt_key;
pub(crate) mod attempt_stage;
pub(crate) mod attempted_outcome;
pub(crate) mod candidate_id;
pub(crate) mod candidate_version;
pub(crate) mod cold_apply_evidence;
pub(crate) mod diagnostic_artifact;
pub(crate) mod experiment_axis;
pub(crate) mod failure_kind;
pub(crate) mod failure_phase;
pub(crate) mod four_way_composition;
pub(crate) mod four_way_expectation;
pub(crate) mod four_way_observation;
pub(crate) mod generated_tree_digest;
pub(crate) mod host_observations;
pub(crate) mod method_facts;
pub(crate) mod not_run_reason;
pub(crate) mod partial_evidence;
pub(crate) mod partial_provision;
pub(crate) mod pinned_artifact_identity;
pub(crate) mod provision_depth;
pub(crate) mod rejected_apply_nanos;
pub(crate) mod resource_disposition;
pub(crate) mod retry_eligibility;
pub(crate) mod retry_ordinal;
pub(crate) mod sampling_progress;
pub(crate) mod screen_block_index;
pub(crate) mod screen_composition;
pub(crate) mod screen_driver;
pub(crate) mod screen_params;
pub(crate) mod screen_record;
pub(crate) mod screen_rung;
pub(crate) mod screen_target;
pub(crate) mod stage_repetition;
pub(crate) mod supersession;

pub(crate) use screen_driver::control_registry_discovery_screen;
