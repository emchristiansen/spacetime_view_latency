//! The typed evidence locating a schedule-order grammar violation.

use crate::analysis::validate::record_slot::RecordSlot;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::observation::record_seq::RecordSeq;
use crate::plan::run_role::RunRole;
use crate::plan::schedule::BlockCoordinate;

/// Which schedule-order obligation a record's sequence position violated, with typed evidence at the
/// narrowest structure that differs first. The stage compares each record, in ascending sequence order,
/// against the record the seed-derived grammar requires at that position and reports the first mismatch,
/// classified by a fixed field priority: block coordinate, then role, then the within-run record slot.
///
/// `position` is the [`RecordSeq`] — the campaign sequence number — at which the grammar and the
/// artifact first disagree.
#[derive(Debug)]
pub(crate) enum ScheduleOrderFault {
    /// At `position` the block whose run should appear — its `(cell, repetition block)` — is not the block
    /// the seed-derived global permutation places there. This is the fault a swapped block *or* a pair of
    /// non-adjacent arm/control runs surfaces: separating a pair forces a foreign block's record into the
    /// pair's second-run position.
    BlockOutOfScheduleOrder {
        position: RecordSeq,
        expected: BlockCoordinate,
        observed: BlockCoordinate,
    },
    /// Within the correct block, the run role at `position` is not the seed-selected adjacent role order,
    /// so the two matched runs execute in the wrong order.
    RoleOutOfScheduleOrder {
        position: RecordSeq,
        block: BlockCoordinate,
        expected: RunRole,
        observed: RunRole,
    },
    /// Within the correct run, the record at `position` is not the expected slot — a manifest not at the
    /// run's first position, or a dose observation out of canonical ladder order.
    RunRecordOutOfScheduleOrder {
        position: RecordSeq,
        run: RunCoordinate,
        expected: RecordSlot,
        observed: RecordSlot,
    },
}
