//! Every line of a whole ledger, sorted into what was admitted and what was refused.

use crate::indexed_sender_view_calibration_analysis::calibration_ledger::CalibrationLedger;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::pair_refusal::PairRefusal;
use crate::indexed_sender_view_calibration_analysis::refused_line::RefusedLine;

/// The admission outcome for a **whole** ledger file: every line offered, none skipped.
///
/// **This type exists so that "the ledger as a whole" is a thing the code holds, not a thing a
/// caller is trusted to have checked.** The frozen inventory is exactly two original records. A
/// ledger that also contains a failed attempt, a foreign-role line, or a retry-shaped duplicate is
/// therefore not that inventory — even when two of its lines are individually perfect. An earlier
/// draft passed only the admitted vector onward, so those extra lines were computed, retained, and
/// then silently dropped the moment the two originals happened to admit; the report it produced said
/// nothing about them at all.
///
/// **The whole file, not merely a whole slice.** [`Self::of`] consumes a
/// [`CalibrationLedger`] by value, and that type can only be obtained by reading a path, so
/// "every line" means every line of an actual artifact rather than of whatever text a caller
/// assembled. Source positions arrive already minted by the decoder instead of being re-derived here
/// from a vector index, which was only correct while that vector happened to be the entire file in
/// original order.
///
/// **Every line is offered before any refusal is acted on.** Stopping at the first refusal would
/// report one lost slot and hide the other, and "line 1 was verified against 999 appends" versus
/// "both lines were" call for different redesigns.
#[derive(Debug, Clone)]
pub(crate) struct LedgerAdmission {
    admitted: Vec<CompleteReplicate>,
    refused: Vec<RefusedLine>,
}

impl LedgerAdmission {
    /// Offer every line of the ledger for admission, in file order.
    pub(crate) fn of(ledger: CalibrationLedger) -> Self {
        let mut admitted = Vec::new();
        let mut refused = Vec::new();
        for line in ledger.into_lines() {
            match CompleteReplicate::admit(&line.record) {
                Ok(replicate) => admitted.push(replicate),
                Err(reason) => refused.push(RefusedLine {
                    line_number: line.line_number,
                    reason,
                }),
            }
        }
        Self { admitted, refused }
    }

    /// Consume the outcome, yielding the complete replicates **only** if nothing was refused.
    ///
    /// **The single exit, and it discharges the refusals rather than handing them over.** An earlier
    /// shape yielded both halves as a tuple, on the reasoning that a caller taking the pair could not
    /// drop one of them. It could: nothing stopped it binding the refusals to `_` and proceeding, so
    /// "the ledger had other lines" remained a fact a caller had to remember to act on. Here the
    /// refusals are never handed out at all — they are either the reason this fails, or they were
    /// empty. No sequence of calls on a `LedgerAdmission` reaches *its* admitted replicates while
    /// any of its own lines was refused: this is its only exit, and it consumes `self`.
    ///
    /// **That is a guarantee about this value, deliberately not about the crate.**
    /// [`CompleteReplicate::admit`] is `pub(crate)`, so a caller holding lines can always run
    /// admission itself and keep both halves side by side. What this type removes is the accidental
    /// separation on the path the analyzer actually takes — not the possibility of someone writing a
    /// second, unchecked path beside it.
    ///
    /// The refusal is a [`PairRefusal`] because that is what the condition means to the only caller:
    /// a ledger with any refused line is not the frozen inventory, so no pair exists. Every refused
    /// line travels inside it with its position and reason.
    pub(crate) fn into_complete_inventory(self) -> Result<Vec<CompleteReplicate>, PairRefusal> {
        let LedgerAdmission { admitted, refused } = self;
        if !refused.is_empty() {
            return Err(PairRefusal::LedgerHasRefusedLines { refused });
        }
        Ok(admitted)
    }
}
