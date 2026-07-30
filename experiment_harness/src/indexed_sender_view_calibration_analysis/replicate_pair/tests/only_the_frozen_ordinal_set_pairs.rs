//! Pairing requires exactly the frozen ordinal set — not merely two distinct ordinals.

use crate::indexed_sender_view_calibration_analysis::calibration_ledger::CalibrationLedger;
use crate::indexed_sender_view_calibration_analysis::frozen_replicate_ordinals::frozen_replicate_ordinals;
use crate::indexed_sender_view_calibration_analysis::ledger_admission::LedgerAdmission;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_analysis::pair_refusal::PairRefusal;
use crate::indexed_sender_view_calibration_analysis::replicate_pair::ReplicatePair;

/// Coverage: every way an all-admissible ledger can still fail to be the frozen inventory.
///
/// **Foreign ordinals are the case a distinctness check would miss.** Records at replicates 2 and 3
/// are two perfectly distinct attempts; pairing them would silently promote slots the inventory never
/// declared into "the two originals", and every diagnostic downstream would then describe a
/// comparison the freeze does not contain. The requirement is therefore set equality against
/// [`frozen_replicate_ordinals`], not a count plus an inequality.
///
/// **A duplicate is reported as itself**, not as a missing original. `{0, 0}` and `{0}` are the same
/// set, so without its own check one attempt counted twice would be reported as replicate 1 being
/// absent — a different diagnosis leading to a different redesign.
///
/// Every case here is built from fully admissible lines, so each refusal is attributable to the
/// ledger's *composition* rather than to any line being bad. The complementary case — good lines plus
/// a refused one — is `a_ledger_with_an_extra_refused_line_yields_no_report`.
#[test]
fn only_the_frozen_ordinal_set_pairs() {
    let frozen = frozen_replicate_ordinals();

    pair(&[0, 1]).expect("the frozen inventory's two originals pair");

    assert_eq!(
        pair(&[0]).unwrap_err(),
        PairRefusal::OrdinalsNotFrozenInventory {
            found: [0].into_iter().collect(),
            expected: frozen.clone(),
        },
        "one series is not the pair the rule is stated over; the method is redesigned or deferred"
    );

    assert_eq!(
        pair(&[2, 3]).unwrap_err(),
        PairRefusal::OrdinalsNotFrozenInventory {
            found: [2, 3].into_iter().collect(),
            expected: frozen.clone(),
        },
        "two distinct but undeclared ordinals are not the frozen inventory's originals"
    );

    assert_eq!(
        pair(&[0, 1, 0]).unwrap_err(),
        PairRefusal::DuplicateOrdinal { replicate: 0 },
        "one attempt counted twice is reported as the duplicate it is, not as a missing original"
    );

    // The overcomplete case: the frozen set is present *and* something else is. A membership test
    // written as "both originals are here" rather than as set equality would accept this, and the
    // report would then describe a pair drawn from a file that also contains an undeclared third
    // attempt. It is a distinct fault from `[2, 3]` — nothing is missing here, only extra — and from
    // `[0, 1, 0]`, where the extra line is a repeat rather than a new slot.
    assert_eq!(
        pair(&[0, 1, 2]).unwrap_err(),
        PairRefusal::OrdinalsNotFrozenInventory {
            found: [0, 1, 2].into_iter().collect(),
            expected: frozen,
        },
        "a superset of the frozen inventory is not the frozen inventory; containing both originals \
         is not the same as being exactly them"
    );
}

/// Build a ledger of fully valid lines at `replicates` and offer it for pairing.
fn pair(replicates: &[u32]) -> Result<ReplicatePair, PairRefusal> {
    let lines: Vec<String> = replicates
        .iter()
        .map(|replicate| LedgerFixture::complete(*replicate).line())
        .collect();
    let ledger_of_fixtures = CalibrationLedger::from_complete_contents_for_tests(&lines.join("\n"))
        .expect("the fixture lines decode");
    ReplicatePair::of(LedgerAdmission::of(ledger_of_fixtures))
}
