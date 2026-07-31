//! A ledger whose sequences do not start at zero and step by one in stream order is refused.

use crate::observation::record_seq::RecordSeq;
use crate::view_read_set_campaign::campaign_ledger_line::CampaignLedgerLine;
use crate::view_read_set_campaign::reconciled_campaign::ReconciledCampaign;

use super::fixture;

/// Coverage: the four ways a ledger's sequences can be wrong, exhausted in one place because they
/// are one defect seen from four sides — the position a line claims is not the position it
/// occupies.
///
/// A hole must be a rejection rather than a gap a reader interpolates over, because the missing
/// line could be exactly the clearance or terminal record an accounting rule turns on: every
/// per-attempt rule below is stated as "exactly one", and a rule that counted lines from a stream
/// with an unnoticed gap would read a lost line as an absent one and pass.
///
/// A reordering matters for a different reason: relative position within the stream is what "the
/// clearance precedes its terminal line" is checked against, so a stream whose order is not its
/// recorded order would make that check meaningless.
///
/// Each case is built by reassigning the sequences of an otherwise *valid* sixty-slot ledger, so
/// nothing but the sequencing differs from a ledger that is accepted.
#[test]
fn a_ledger_whose_sequences_are_not_contiguous_from_zero_is_refused() {
    let valid = fixture::full_campaign().lines();

    let cases: [(&str, Vec<CampaignLedgerLine>); 4] = [
        (
            "a ledger that starts at one",
            renumbered(&valid, |position| position + 1),
        ),
        (
            "a ledger with a hole",
            renumbered(
                &valid,
                |position| if position < 3 { position } else { position + 1 },
            ),
        ),
        (
            "a ledger with a repeated sequence",
            renumbered(&valid, |position| if position == 3 { 2 } else { position }),
        ),
        ("a ledger whose lines are out of order", reordered(&valid)),
    ];

    for (description, lines) in cases {
        let error = ReconciledCampaign::reconciled(lines)
            .expect_err(&format!("{description} must be refused"));
        let rendered = format!("{error:#}");
        assert!(
            rendered.contains("contiguous"),
            "{description} must be refused for its sequencing, got {rendered}"
        );
    }
}

/// The same records, with each line's sequence replaced by `sequence_at(position)`.
fn renumbered(
    lines: &[CampaignLedgerLine],
    sequence_at: impl Fn(u64) -> u64,
) -> Vec<CampaignLedgerLine> {
    lines
        .iter()
        .enumerate()
        .map(|(position, line)| {
            let position = u64::try_from(position).expect("a campaign ledger's length fits u64");
            CampaignLedgerLine::at(RecordSeq::new(sequence_at(position)), line.body().clone())
        })
        .collect()
}

/// The same lines, sequences untouched, with two adjacent ones swapped — so the stream is a
/// permutation of a valid ledger rather than a renumbering of one.
fn reordered(lines: &[CampaignLedgerLine]) -> Vec<CampaignLedgerLine> {
    let mut reordered = lines.to_vec();
    reordered.swap(2, 3);
    reordered
}
