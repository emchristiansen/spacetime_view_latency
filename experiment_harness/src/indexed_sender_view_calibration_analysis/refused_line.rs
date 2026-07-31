//! One ledger line that was offered for admission and refused, with where it was.

use std::fmt;

use crate::indexed_sender_view_calibration_analysis::admission_refusal::AdmissionRefusal;

/// A refused line, paired with its 1-based position in the ledger.
///
/// **The position is half the diagnostic.** "A record was verified against 999 appends" does not
/// tell an operator which of the two originals is affected; "line 2 was verified against 999
/// appends" does, and the ledger is written one attempt per line in issue order. Keeping the pair in
/// a named type rather than a bare tuple means neither half can be dropped on the way to the report,
/// and no caller has to remember which element of a `(u64, _)` is which.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RefusedLine {
    /// Where the refused record sits in the ledger, counting from one.
    pub(crate) line_number: u64,
    /// Why it is not a complete replicate.
    pub(crate) reason: AdmissionRefusal,
}

impl fmt::Display for RefusedLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let RefusedLine {
            line_number,
            reason,
        } = self;
        write!(f, "line {line_number}: {reason}")
    }
}
