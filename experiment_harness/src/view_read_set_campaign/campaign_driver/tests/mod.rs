//! Focused tests for the driver's pure and adapter stages. One test entity per file.
//!
//! Two kinds of stage are testable, and that is the honest scope of this tree. `schedule_retry` is
//! pure, taking a bound terminal record and returning an identity. The four recording adapters each
//! build one `CampaignRecord` and append it, so an injected writer is the whole seam they need — the
//! same `#[cfg(test)]` constructor the sink's own tests use, with no fake sink and no production
//! accessor added for their sake.
//!
//! What the adapter tests deliberately do not cover is *order*. Inventory-first,
//! clearance-before-provisioning, and post-attempt-immediately-after-terminal are properties of
//! `run_campaign`, `run_attempt`, and `settle`, which are still `todo!()`, and are independently
//! re-derived by reconciliation from the finished ledger. A single append cannot see its own
//! position, so nothing here stands in for those stages.
//!
//! Every remaining stage is still an explicit `todo!()`: the gate stages read `/proc` and wait on a
//! monotonic clock; provisioning and measurement need a live pinned server, a published module, and
//! a reducer the module does not yet have; and the four orchestration stages compose all of those.

mod capturing_writer;
mod fixture;
mod scripted_writer;

mod a_retry_is_scheduled_exactly_when_eligibility_grants_one;
mod a_terminal_ledger_refuses_every_recording_adapter;
mod each_recording_adapter_appends_its_own_record;
