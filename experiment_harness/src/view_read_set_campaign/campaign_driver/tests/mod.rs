//! Focused tests for the driver's pure stages. One test entity per file.
//!
//! Only one driver stage is testable at all today, and that is the honest scope of this tree:
//! `schedule_retry` is pure, taking a bound terminal record and returning an identity, with no
//! filesystem, clock, server, sink, or live-provenance dependency. It therefore leaves no
//! direct-inspection-only residue — every disposition it can produce is exercised here.
//!
//! Every other stage in this file is still an explicit `todo!()`, and the reasons differ: the
//! recording stages wait on `CampaignSink::append`; the gate stages read `/proc` and wait on a
//! monotonic clock; provisioning and measurement need a live pinned server, a published module, and
//! a reducer the module does not yet have; and the four orchestration stages compose all of those.
//! None of them is blocked on anything this tree could substitute, so nothing here stands in for
//! them.

mod a_retry_is_scheduled_exactly_when_eligibility_grants_one;
mod fixture;
