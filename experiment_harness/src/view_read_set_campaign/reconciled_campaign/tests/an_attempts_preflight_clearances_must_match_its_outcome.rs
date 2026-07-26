//! Preflight clearances must be exactly what an attempt's terminal outcome calls for, before it.

use crate::view_read_set_campaign::campaign_ledger_line::CampaignLedgerLine;
use crate::view_read_set_campaign::campaign_record::CampaignRecord;
use crate::view_read_set_campaign::reconciled_campaign::ReconciledCampaign;

use super::fixture;

/// Coverage: the whole clearance family — both directions of the cardinality rule, a duplicate, and
/// a misplacement — exhausted in one place because they are the four ways one rule can be broken.
///
/// **What the rule is for.** "Launch only if the host is quiet" is unauditable for exactly the
/// attempts whose evidence gets used, unless a launched attempt carries the readings that cleared
/// it. A missing clearance means an attempt ran ungated, and its measurement may have been taken on
/// a contended host — which is not detectable from the measurement itself.
///
/// **Why the extra-clearance direction matters just as much.** A clearance for a
/// preflight-*rejected* attempt is a flat contradiction: the gate refused it, and that refusal is
/// already recorded inside the terminal record. A ledger asserting both would say the same gate both
/// passed and failed, and a reader would have no way to tell which copy was true. That is why the
/// rule is total over the outcome rather than an at-most-one bound.
///
/// **Why position is checked and not only count.** A clearance written *after* its terminal line
/// cannot have cleared anything — it postdates the attempt it claims to have authorized. Recording
/// it afterwards would be indistinguishable from writing the gate readings to justify a launch that
/// had already happened.
///
/// The requirement is only that it *precede*, not that it be adjacent, which is what its frozen
/// clause says: the gate readings are written as soon as the gate passes, and other attempts' lines
/// may legitimately fall between them and this attempt's terminal record. The post-attempt reading
/// is the stricter case — the spec requires it immediately after the measured attempt — and that
/// clause is not reachable from here.
#[test]
fn an_attempts_preflight_clearances_must_match_its_outcome() {
    let launched = fixture::failed_slot();
    let rejected = fixture::eligible_slot();

    let missing = fixture::resequenced(&without_clearance(&fixture::full_campaign().lines()));

    let mut duplicated = fixture::full_campaign();
    duplicated.append(CampaignRecord::PreflightCleared {
        attempt: launched,
        gate: fixture::passed_gate(),
    });

    let mut contradictory = fixture::full_campaign();
    contradictory.append(CampaignRecord::PreflightCleared {
        attempt: rejected,
        gate: fixture::passed_gate(),
    });

    let mut misplaced_lines = without_clearance(&fixture::full_campaign().lines());
    misplaced_lines.push(CampaignLedgerLine::at(
        crate::observation::record_seq::RecordSeq::zero(),
        CampaignRecord::PreflightCleared {
            attempt: launched,
            gate: fixture::passed_gate(),
        },
    ));
    let misplaced = fixture::resequenced(&misplaced_lines);

    let cases = [
        (
            "a launched attempt with no clearance",
            missing,
            launched,
            "has 0 preflight clearance",
        ),
        (
            "a launched attempt cleared twice",
            duplicated.lines(),
            launched,
            "has 2 preflight clearance",
        ),
        (
            "a preflight-rejected attempt with a clearance",
            contradictory.lines(),
            rejected,
            "has 1 preflight clearance",
        ),
        (
            "a clearance written after the terminal line it claims to authorize",
            misplaced,
            launched,
            "must be written at some sequence before",
        ),
    ];

    for (description, lines, attempt, expected) in cases {
        let error = ReconciledCampaign::reconciled(lines)
            .expect_err(&format!("{description} must be refused"));
        let rendered = format!("{error:#}");
        assert!(
            rendered.contains(&attempt.canonical_tag()),
            "{description} must be refused by naming the attempt, got {rendered}"
        );
        assert!(
            rendered.contains(expected),
            "{description} must be refused for its clearance accounting, got {rendered}"
        );
    }
}

/// The same ledger with its single preflight clearance removed.
///
/// The fixture's campaign launches exactly one attempt, so there is exactly one clearance to drop
/// and no ambiguity about which.
fn without_clearance(lines: &[CampaignLedgerLine]) -> Vec<CampaignLedgerLine> {
    lines
        .iter()
        .filter(|line| !matches!(line.body(), CampaignRecord::PreflightCleared { .. }))
        .cloned()
        .collect()
}
