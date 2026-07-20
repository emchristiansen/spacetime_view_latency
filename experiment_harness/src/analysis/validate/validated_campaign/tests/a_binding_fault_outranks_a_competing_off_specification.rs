//! First-error-order proof: the stage-3 manifest-binding pass strictly precedes the stage-4 stable-fact
//! pass *across all records*, not per record.
//!
//! Stage 3 (manifest↔reference binding) runs to completion over every manifest before stage 4
//! (campaign-stable / off-spec / provenance) begins over any. This proof separates the two faults onto
//! *different* records in sequence order: an off-specification fault on the earlier manifest and a binding
//! fault on a later one. A correct staged pipeline reaches the later record's binding fault (stage 3) before
//! it would ever run stage 4 on the earlier record, so it returns the binding fault. An accidental per-record
//! pipeline — stage 4 on the earlier record before stage 3 on the later one — would instead surface the
//! earlier off-specification, so this fixture detects that inversion where a same-record fixture could not.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::analysis::validate::manifest_reference_fault::ManifestReferenceFault;

/// The earlier-sequence manifest is given a stage-4 off-specification fault (an off-spec CLI version); a
/// later-sequence manifest is given a stage-3 binding fault (its embedded self-reference names a divergent
/// schedule seed). Because the whole stage-3 binding pass completes before the stage-4 stable-fact pass
/// begins, the fold reaches the later record's `EmbeddedDisagreement` before it would ever evaluate the
/// earlier record's version — so the binding fault is returned, proving stage 3 precedes stage 4 globally.
#[test]
fn a_binding_fault_outranks_a_competing_off_specification() {
    // A schedule seed distinct from the fixture's shared seed, so the later manifest's embedded reference
    // disagrees with its own (unchanged) manifest identity.
    let divergent_seed = 1_234_567u64;
    assert_ne!(
        divergent_seed,
        CampaignFixture::schedule_seed().get(),
        "the divergent embedded seed must differ from the fixture's shared seed"
    );

    let mut fixture = CampaignFixture::valid();

    // The stage-4 off-spec fault goes on the first manifest in sequence order; the stage-3 binding fault on
    // the second — two distinct runs, the off-spec one strictly earlier in the durable sequence.
    let off_spec_index = fixture.nth_manifest_index(0);
    let binding_index = fixture.nth_manifest_index(1);
    let off_spec_seq = manifest_seq(&fixture.records()[off_spec_index]);
    let binding_seq = manifest_seq(&fixture.records()[binding_index]);
    assert!(
        off_spec_seq < binding_seq,
        "the off-specification manifest (seq {off_spec_seq}) must sit strictly earlier in sequence than \
         the binding-fault manifest (seq {binding_seq})"
    );

    // Stage-4 fault on the earlier manifest: an off-specification CLI version.
    match &mut fixture.records_mut()[off_spec_index] {
        WireRecordDto::Manifest { body, .. } => {
            body.manifest.distribution.cli_version = "9.9.9".to_string();
        }
        WireRecordDto::Dose { .. } => panic!("the located record must be a manifest"),
    }
    // Stage-3 fault on the later manifest: its embedded self-reference names a divergent seed, disagreeing
    // with its own unchanged manifest identity.
    match &mut fixture.records_mut()[binding_index] {
        WireRecordDto::Manifest { body, .. } => {
            body.reference.schedule_seed = divergent_seed;
        }
        WireRecordDto::Dose { .. } => panic!("the located record must be a manifest"),
    }

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!(
            "a campaign with an earlier off-specification manifest and a later binding-fault manifest must \
             fail validation"
        );
    };

    match error {
        IntegrityError::ManifestReferenceBinding {
            fault: ManifestReferenceFault::EmbeddedDisagreement { .. },
            ..
        } => {}
        other => panic!(
            "expected the later record's stage-3 EmbeddedDisagreement to outrank the earlier record's \
             stage-4 off-specification, got {other:?}"
        ),
    }
}

/// The durable sequence number of a manifest record.
fn manifest_seq(record: &WireRecordDto) -> u64 {
    match record {
        WireRecordDto::Manifest { record, .. } => record.seq,
        WireRecordDto::Dose { .. } => panic!("the located record must be a manifest"),
    }
}
