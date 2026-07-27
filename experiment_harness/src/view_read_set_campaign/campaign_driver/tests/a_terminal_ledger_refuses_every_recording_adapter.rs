//! A ledger made terminal by a persist failure refuses every recording adapter, and each says so in
//! its own words.

use anyhow::Result;

use crate::view_read_set_campaign::campaign_driver::{
    record_inventory, record_post_attempt, record_preflight_cleared, record_terminal,
};
use crate::view_read_set_campaign::campaign_sink::CampaignSink;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;

use super::fixture;
use super::scripted_writer::ScriptedWriter;

/// Coverage: the other half of what an adapter does — propagate. Each gets its own fresh sink over a
/// writer that fails every line, so the first call is that sink's first persist failure and the
/// second meets a ledger already terminal.
///
/// Both calls are asserted, because they fail for different reasons and an adapter could report one
/// while losing the other. What matters is that neither is *replaced*: the sink's diagnostic survives
/// under the adapter's own context, so a reader learns both which line kind was being recorded and
/// that the ledger had already refused writes. An adapter that mapped the error to its own text would
/// hide the refusal; one that dropped its context would leave the persist failure unattributed.
///
/// A fresh sink per adapter rather than one shared: sharing would let the first adapter's failure be
/// the poison every later one reports, which proves nothing about those later ones.
#[test]
fn a_terminal_ledger_refuses_every_recording_adapter() {
    let attempt = fixture::key(RetryOrdinal::ORIGINAL);
    let inventory = fixture::inventory();
    let provenance = fixture::campaign_provenance();
    let terminal = fixture::record(RetryOrdinal::ORIGINAL, fixture::preflight_rejected());
    let gate = fixture::passed_gate();
    let sample = fixture::post_attempt_sample();

    let inventory_call = |sink: &mut CampaignSink| record_inventory(sink, &inventory, &provenance);
    let clearance_call = |sink: &mut CampaignSink| record_preflight_cleared(sink, attempt, gate);
    let terminal_call = |sink: &mut CampaignSink| record_terminal(sink, &terminal);
    let post_attempt_call = |sink: &mut CampaignSink| record_post_attempt(sink, attempt, sample);

    let adapters: [(&str, &dyn Fn(&mut CampaignSink) -> Result<()>); 4] = [
        ("recording the campaign inventory", &inventory_call),
        ("recording the preflight clearance", &clearance_call),
        ("recording the attempt terminal outcome", &terminal_call),
        ("recording the post-attempt environment", &post_attempt_call),
    ];

    for (context, append) in adapters {
        let mut sink = CampaignSink::from_writer(Box::new(ScriptedWriter::ok_for(0)));

        let first = append(&mut sink)
            .expect_err("a writer that fails every line makes the very first append fail");
        assert!(
            format!("{first:#}").contains(context),
            "the first failure of `{context}` must be reported under that context"
        );

        let refused = append(&mut sink).expect_err("the ledger is terminal after that failure");
        let rendered = format!("{refused:#}");
        assert!(
            rendered.contains(context),
            "the refusal of `{context}` must be reported under that context"
        );
        assert!(
            rendered.contains("refuses further writes"),
            "`{context}` must surface the ledger's own refusal rather than replacing it: {rendered}"
        );
    }
}
