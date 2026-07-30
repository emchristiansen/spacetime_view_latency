//! Every decoded ledger line, sorted into what was admitted and what was refused.

use crate::indexed_sender_view_calibration_analysis::calibration_record_dto::CalibrationRecordDto;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::refused_line::RefusedLine;

/// The admission outcome for a **whole** ledger: every line offered, none skipped.
///
/// **This type exists so that "the ledger as a whole" is a thing the code holds, not a thing a
/// caller is trusted to have checked.** The frozen inventory is exactly two original records. A
/// ledger that also contains a failed attempt, a foreign-role line, or a retry-shaped duplicate is
/// therefore not that inventory — even when two of its lines are individually perfect. An earlier
/// draft passed only the admitted vector onward, so those extra lines were computed, retained, and
/// then silently dropped the moment the two originals happened to admit; the report it produced said
/// nothing about them at all.
///
/// Carrying both halves together closes that by construction: [`ReplicatePair::of`] takes this type
/// and can see the refusals, so it cannot succeed on a ledger that had any.
///
/// **Every line is offered before any refusal is acted on.** Stopping at the first refusal would
/// report one lost slot and hide the other, and "line 1 was verified against 999 appends" versus
/// "both lines were" call for different redesigns.
///
/// [`ReplicatePair::of`]: super::replicate_pair::ReplicatePair::of
#[derive(Debug, Clone)]
pub(crate) struct LedgerAdmission {
    admitted: Vec<CompleteReplicate>,
    refused: Vec<RefusedLine>,
}

impl LedgerAdmission {
    /// Offer every decoded record for admission, in ledger order.
    pub(crate) fn of(records: &[CalibrationRecordDto]) -> Self {
        let mut admitted = Vec::new();
        let mut refused = Vec::new();
        for (index, record) in records.iter().enumerate() {
            let line_number = u64::try_from(index + 1).expect("a 1-based line number fits u64");
            match CompleteReplicate::admit(record) {
                Ok(replicate) => admitted.push(replicate),
                Err(reason) => refused.push(RefusedLine {
                    line_number,
                    reason,
                }),
            }
        }
        Self { admitted, refused }
    }

    /// Consume the outcome, yielding the admitted replicates and every refused line.
    ///
    /// Consuming rather than lending, and yielding **both** halves at once, so a caller cannot take
    /// the admitted replicates while leaving the refusals behind — which is exactly the drop this
    /// type exists to prevent.
    pub(crate) fn into_parts(self) -> (Vec<CompleteReplicate>, Vec<RefusedLine>) {
        (self.admitted, self.refused)
    }
}
