//! Every appended record becomes exactly one line, at the next sequence, tagged with its kind.

use std::time::Duration;

use anyhow::anyhow;

use crate::entity_owner_pilot::attempt_inventory::AttemptInventory;
use crate::entity_owner_pilot::attempt_outcome::AttemptOutcome;
use crate::entity_owner_pilot::campaign_provenance::PilotCampaignProvenance;
use crate::entity_owner_pilot::diagnostic_artifact::DiagnosticArtifact;
use crate::entity_owner_pilot::global_row_rung::GlobalRowRung;
use crate::entity_owner_pilot::not_run_reason::NotRunReason;
use crate::entity_owner_pilot::pilot_record::PilotRecord;
use crate::entity_owner_pilot::pilot_sink::PilotSink;
use crate::entity_owner_pilot::rung_evidence::RungEvidence;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::observation::latency_sample::LatencySample;
use crate::observation::raw_latencies::RawLatencies;
use crate::params::BATCH_SIZE_USIZE;

use super::capturing_writer::CapturingWriter;

/// The three facts a ledger reader needs before it can enforce any of the ordering and accounting
/// invariants `PilotRecord` documents — asserted over the real serialization, not assumed:
///
/// - **one line per record**, so a write is never split or coalesced;
/// - **ascending sequences from zero**, so "first line" and "before" are readable from the file
///   rather than from write-time knowledge;
/// - **a self-describing kind tag** on every body, so `Inventory` / `Provisioned` / `Rung` /
///   `Terminal` can be told apart without replaying the run.
///
/// `Provisioned` is deliberately absent: its provenance is snapshotted out of live provisioning
/// capabilities, which a unit test cannot own. Its ordering is covered by the driver's statement
/// order, not here.
#[test]
fn each_record_is_one_line_carrying_its_sequence_and_kind() {
    const SEED: u64 = 7;

    let seed = ScheduleSeed::new(SEED);
    let inventory = AttemptInventory::frozen(seed).expect("the seed freezes");
    let provenance = PilotCampaignProvenance::resolved().expect("the campaign pins resolve");
    let attempt = inventory.attempts()[0];

    let samples = (0..BATCH_SIZE_USIZE)
        .map(|i| {
            let nanos = u64::try_from(i).expect("test sample index fits u64");
            LatencySample::from_elapsed(Duration::from_nanos(nanos))
        })
        .collect();
    let latencies = RawLatencies::sealed(samples).expect("a full batch of latency samples seals");
    let evidence = RungEvidence::observed(GlobalRowRung::ALL[0], 10, latencies);

    let outcome = AttemptOutcome::NotRun {
        reason: NotRunReason::PriorAttemptReleaseFailed {
            diagnostic: DiagnosticArtifact::of_error(&anyhow!("a prior attempt's release failed")),
        },
    };

    let (writer, lines) = CapturingWriter::new();
    let mut sink = PilotSink::from_writer(Box::new(writer));

    let records = [
        PilotRecord::Inventory {
            seed,
            inventory: &inventory,
            provenance: &provenance,
        },
        PilotRecord::Rung {
            attempt,
            evidence: &evidence,
        },
        PilotRecord::Terminal {
            attempt,
            outcome: &outcome,
        },
    ];
    let expected_kinds = ["Inventory", "Rung", "Terminal"];

    for (index, record) in records.iter().enumerate() {
        let seq = sink.write(record).expect("the capturing writer accepts the line");
        let expected = u64::try_from(index).expect("a record index fits u64");
        assert_eq!(
            seq.get(),
            expected,
            "record {index} must be assigned the next sequence"
        );
    }

    let lines = lines.borrow();
    assert_eq!(
        lines.len(),
        records.len(),
        "each appended record must produce exactly one line"
    );

    for (index, (line, expected_kind)) in lines.iter().zip(expected_kinds).enumerate() {
        assert_eq!(
            line.last(),
            Some(&b'\n'),
            "line {index} must be newline-terminated NDJSON"
        );
        let parsed: serde_json::Value =
            serde_json::from_slice(line).expect("each line must be one JSON object");

        let expected_seq = u64::try_from(index).expect("a line index fits u64");
        assert_eq!(
            parsed["seq"].as_u64(),
            Some(expected_seq),
            "line {index} must carry its assigned sequence"
        );

        let body = parsed["body"]
            .as_object()
            .expect("the body is serde-externally-tagged, so it is a one-key object");
        let kind: Vec<&String> = body.keys().collect();
        assert_eq!(
            kind,
            vec![&expected_kind.to_string()],
            "line {index} must be tagged with exactly its record kind"
        );
    }
}
