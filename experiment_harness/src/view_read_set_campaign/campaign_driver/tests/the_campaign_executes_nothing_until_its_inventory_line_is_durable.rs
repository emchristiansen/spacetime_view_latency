//! A campaign whose opening inventory line does not persist runs no attempt at all.

use std::path::Path;

use crate::manifest::listen_address::ListenAddress;
use crate::view_read_set_campaign::campaign_driver::run_campaign;
use crate::view_read_set_campaign::campaign_sink::CampaignSink;

use super::fixture;
use super::scripted_writer::ScriptedWriter;

/// Coverage: the `?` between the two steps. The count is the assertion with teeth — the error text
/// reads the same whether the campaign stopped or carried on and reported the failure it kept, while
/// exactly one attempted write says nothing came after.
///
/// The execution inputs are inert because they are never reached: only
/// [`run_inventory`](super::super::run_inventory) receives them, and this run does not enter it.
/// Parsing an address touches no network, and neither path is opened.
#[test]
fn the_campaign_executes_nothing_until_its_inventory_line_is_durable() {
    let inventory = fixture::inventory();
    let provenance = fixture::campaign_provenance();
    let listen = match ListenAddress::parse("127.0.0.1:1") {
        Ok(listen) => listen,
        Err(error) => {
            panic!("an explicit host:port parses without touching the network: {error:#}")
        }
    };
    let module_wasm = Path::new("/nonexistent/experiment_module.wasm");
    let artifacts = Path::new("/nonexistent/artifacts");

    let (writer, calls) = ScriptedWriter::counted(0);
    let mut sink = CampaignSink::from_writer(Box::new(writer));

    let stopped = match run_campaign(
        &mut sink,
        &inventory,
        &provenance,
        listen,
        module_wasm,
        artifacts,
    ) {
        Ok(()) => panic!("a campaign that never recorded its inventory cannot report success"),
        Err(stopped) => format!("{stopped:#}"),
    };

    assert!(
        stopped.contains("recording the campaign inventory"),
        "the opening line is what failed, and the campaign must say so: {stopped}"
    );
    assert_eq!(
        *calls.borrow(),
        1,
        "the ledger must be offered exactly the inventory line and nothing after it"
    );
}
