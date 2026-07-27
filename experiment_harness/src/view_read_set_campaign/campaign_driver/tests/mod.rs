//! Focused tests for the driver's pure, adapter, settlement, and composition stages. One test entity
//! per file.
//!
//! Four kinds of stage are testable, and that is the honest scope of this tree. `schedule_retry` is
//! pure, taking a bound terminal record and returning an identity. The four recording adapters each
//! build one `CampaignRecord` and append it. `settle` composes two of those adapters with an injected
//! release closure. Those three need only an injected writer — the same `#[cfg(test)]` constructor
//! the sink's own tests use, with no fake sink and no production accessor added for their sake.
//! `validate_final_composition` needs no seam at all: it derives its expectation from the attempt key
//! and reads two retained artifacts, so a scratch directory and the real persistence path are the
//! whole of its input.
//!
//! Order is covered exactly where it is owned. Terminal-immediately-followed-by-post-attempt is
//! `settle`'s, and is proved here; Inventory-first and clearance-before-provisioning belong to
//! `run_campaign` and `run_attempt`, which are still `todo!()`. Reconciliation re-derives all three
//! independently from the finished ledger.
//!
//! The measurement stage is live and mostly untestable from here: `measure_attempt` and
//! `measure_channel` take a connected client, subscribe, reconnect, and issue writes against a real
//! server. What can be proved without one was extracted rather than faked — `failure_stage` is
//! covered here, the two batch barriers and the paced stop condition beside the client, and the
//! role→target coupling beside that type. No seam was added to reach the rest.
//!
//! `observe_environment`'s four `/proc` parsers are covered here over verbatim kernel content; the
//! reads themselves and the page-size and CPU-count queries are host facts, covered by inspection.
//! `preflight_gate` composes those with a sixty-second wait, so it has no pure factor of its own; the
//! decision it produces is proved beside `EnvironmentGateEvidence`.
//!
//! Every remaining stage is still an explicit `todo!()`: provisioning needs a live pinned server and
//! a published module, and the four remaining orchestration stages compose all of those.

mod capturing_writer;
mod composition_fixture;
mod fixture;
mod scratch_dir;
mod scripted_writer;

mod a_control_attempt_validates_the_whole_swept_slice;
mod a_leaked_foreign_row_fails_as_semantics_or_security;
mod a_load_average_reads_its_first_field_in_hundredths;
mod a_measureless_attempt_records_no_post_line_and_reports_its_release_failures;
mod a_retry_is_scheduled_exactly_when_eligibility_grants_one;
mod a_settled_attempt_records_its_terminal_line_then_its_post_attempt_line;
mod a_terminal_ledger_refuses_every_recording_adapter;
mod an_arm_attempt_validates_its_own_read_set_and_keeps_its_channels;
mod an_observed_sample_makes_a_failure_measured_with_no_evidence_to_show_for_it;
mod available_memory_is_the_kernels_kibibyte_line_in_bytes;
mod each_recording_adapter_appends_its_own_record;
mod every_primary_failure_leads_while_release_still_runs;
mod memory_pressure_reads_full_avg60_and_not_some;
mod swap_out_reads_the_cumulative_pswpout_counter;
