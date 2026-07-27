//! Focused tests for the driver's pure, adapter, and settlement stages. One test entity per file.
//!
//! Three kinds of stage are testable, and that is the honest scope of this tree. `schedule_retry` is
//! pure, taking a bound terminal record and returning an identity. The four recording adapters each
//! build one `CampaignRecord` and append it. `settle` composes two of those adapters with an injected
//! release closure. All three need only an injected writer — the same `#[cfg(test)]` constructor the
//! sink's own tests use, with no fake sink and no production accessor added for their sake.
//!
//! Order is covered exactly where it is owned. Terminal-immediately-followed-by-post-attempt is
//! `settle`'s, and is proved here; Inventory-first and clearance-before-provisioning belong to
//! `run_campaign` and `run_attempt`, which are still `todo!()`. Reconciliation re-derives all three
//! independently from the finished ledger.
//!
//! Every remaining stage is still an explicit `todo!()`: the gate stages read `/proc` and wait on a
//! monotonic clock; provisioning and measurement need a live pinned server and a published module;
//! and the three remaining orchestration stages compose all of those.

mod capturing_writer;
mod fixture;
mod scripted_writer;

mod a_measureless_attempt_records_no_post_line_and_reports_its_release_failures;
mod a_retry_is_scheduled_exactly_when_eligibility_grants_one;
mod a_settled_attempt_records_its_terminal_line_then_its_post_attempt_line;
mod a_terminal_ledger_refuses_every_recording_adapter;
mod each_recording_adapter_appends_its_own_record;
mod every_primary_failure_leads_while_release_still_runs;
