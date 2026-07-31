//! The canonical text is primary-key ordered, one terminated line per row, whatever order the rows
//! arrived in.

use super::row::{owner, row};

/// Coverage: the content address is only a *content* address if two subscribers that received the
/// same rows in different orders produce the same bytes. Delivery order is not something the harness
/// controls, so the ordering has to be recomputed here rather than inherited — and if it were not,
/// two artifacts of one identical state would carry different digests, and a reader comparing them
/// would conclude the states differed.
///
/// The rows are therefore supplied deliberately out of order, and the whole expected text is
/// asserted exactly rather than parsed back: the claim is about bytes, so a round-trip through a
/// parser could hide a separator or terminator change that a reader's `wc -l` would not.
///
/// Termination is part of the same assertion. Every line ends in `\n` *including the last*, which is
/// what makes `wc -l` equal the recorded row count, and the payload with an embedded colon is
/// present because real measured payloads contain them — the reader's rule is a left split into
/// three fields, not a split on every colon.
#[test]
fn rows_are_ordered_by_key_and_every_line_is_terminated() {
    let owner_hex = owner().to_hex().to_string();

    let unordered = vec![
        row(
            9,
            "view-read-set-campaign-mutation:e1-saturated-queue-growth:999",
        ),
        row(0, "view-read-set-campaign-seeded-payload"),
        row(1_000_000_000, "view-read-set-campaign-seeded-payload"),
        row(2, "view-read-set-campaign-seeded-payload"),
    ];

    let text = super::super::canonical_encoding(&unordered)
        .expect("distinct keys and line-break-free records canonicalize");

    assert_eq!(
        text,
        format!(
            "0:{owner_hex}:view-read-set-campaign-seeded-payload\n\
             2:{owner_hex}:view-read-set-campaign-seeded-payload\n\
             9:{owner_hex}:view-read-set-campaign-mutation:e1-saturated-queue-growth:999\n\
             1000000000:{owner_hex}:view-read-set-campaign-seeded-payload\n"
        ),
        "rows must be emitted in ascending primary-key order, one terminated line each, \
         regardless of the order they arrived in"
    );

    assert_eq!(
        text.lines().count(),
        unordered.len(),
        "the artifact's line count is the row count, which is what makes wc -l a valid check"
    );
}
