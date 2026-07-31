//! Focused tests for the campaign-local row counter. One test entity per file.
//!
//! The counter is the one piece of the delivery pipeline that is both live-facing and fully pure:
//! the SDK's callbacks call `record_*`, but nothing about the tallying needs a connection. What the
//! callbacks are registered against is the driver's measurement stage, and is not tested here.

mod a_snapshot_reports_the_window_without_clearing_it;
mod each_event_kind_accumulates_on_its_own_tally;
