//! A ledger that does not open with exactly one inventory line is refused.

use crate::view_read_set_campaign::campaign_record::CampaignRecord;
use crate::view_read_set_campaign::reconciled_campaign::ReconciledCampaign;

use super::fixture;

/// Coverage: the three ways the opening preregistration can be wrong, exhausted together because
/// all three break the same guarantee — that a ledger is accounted against the inventory and pins
/// it was actually written under.
///
/// A ledger with no lines at all has no preregistration to check against, and silently accounting
/// it as an empty campaign would report "no missing slots" for a campaign that recorded nothing.
///
/// A ledger whose first line is not the inventory records evidence written before its own
/// preregistration existed, which is the ordering the protocol exists to prevent: the frozen order
/// and pins must precede the evidence they govern.
///
/// A second inventory line is the subtle one. It cannot be caught by "is there an inventory?", and
/// a reader that took the first would silently ignore a second that disagreed with it — so a ledger
/// could name two different frozen orders, or two different sets of pins, and be accounted against
/// whichever the reconciler happened to reach first.
#[test]
fn a_ledger_without_exactly_one_opening_inventory_is_refused() {
    let mut with_second_inventory = fixture::full_campaign();
    let second = with_second_inventory.append(CampaignRecord::Inventory {
        inventory: fixture::inventory(),
        provenance: fixture::provenance(),
    });

    let without_inventory = fixture::resequenced(
        fixture::full_campaign()
            .lines()
            .get(1..)
            .expect("a full campaign ledger has lines after its inventory"),
    );

    let cases = [
        ("a ledger with no lines at all", Vec::new(), "no lines"),
        (
            "a ledger whose first line is not its inventory",
            without_inventory,
            "first line is a",
        ),
        (
            "a ledger recording a second inventory",
            with_second_inventory.lines(),
            "a second one",
        ),
    ];

    for (description, lines, expected) in cases {
        let error = ReconciledCampaign::reconciled(lines)
            .expect_err(&format!("{description} must be refused"));
        let rendered = format!("{error:#}");
        assert!(
            rendered.contains(expected),
            "{description} must be refused for its inventory line, got {rendered}"
        );
    }

    assert!(
        second.get() > 0,
        "the second inventory was appended after the campaign, not at the opening line"
    );
}
