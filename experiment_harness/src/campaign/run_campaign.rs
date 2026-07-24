//! The real campaign orchestrator: draw each block from the consuming campaign cursor and drive it.

use std::path::Path;

use crate::campaign::campaign_aborted::CampaignAborted;
use crate::campaign::campaign_cursor::CampaignBlockOutcome;
use crate::campaign::campaign_cursor::CampaignReady;
use crate::campaign::campaign_cursor::CampaignStep;
use crate::campaign::campaign_outcome::CampaignOutcome;
use crate::manifest::listen_address::ListenAddress;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::observation::observation_sink::ObservationSink;

/// Drive the whole preregistered campaign under `seed`, streaming each run's records to the campaign's one
/// owned `sink`, and yield the single terminal [`CampaignOutcome`] — or a typed [`CampaignAborted`] on the
/// Err channel if acquiring any run failed before that run's cleanup owner existed.
///
/// The orchestrator holds no run/block/coordinate state of its own: it only draws the next block from the
/// [`CampaignReady`] cursor and, for each drawn block, hands the non-identity server/Wasm configuration to
/// the paired [`CampaignBlockPending`](crate::campaign::campaign_cursor::CampaignBlockPending) carrier, which
/// owns the block cursor and the campaign's private continuation and loops the concrete run acquisition
/// itself. A completed block resumes the campaign on its own iterator; a block that stopped short concludes
/// the campaign at its terminal; block-iterator exhaustion drives the mandatory finalize via
/// [`CampaignExhausted::finalize`](crate::campaign::campaign_cursor::CampaignExhausted::finalize).
///
/// `seed` is the single [`ScheduleSeed`]: it drives both the block-order/arm-first permutation the campaign
/// cursor derives internally *and* the deterministic dataset each run provisions and seeds, so the two
/// cannot diverge. `pub(crate)` because this is the harness's sole production campaign entry point.
pub(crate) fn run_campaign(
    listen: ListenAddress,
    module_wasm: &Path,
    seed: ScheduleSeed,
    sink: ObservationSink,
    // Signature-only for Phase 1: requiring a clean worktree here, comparing it against
    // `BuildProvenance`'s `EmbeddedHarnessCommit`, and repeating that recheck before each of the 540
    // runs are deferred to Phase 2.
    harness_checkout_root: &Path,
) -> Result<CampaignOutcome, CampaignAborted> {
    let _ = harness_checkout_root;
    let mut campaign = CampaignReady::preregistered(seed, sink);
    loop {
        match campaign.next_block() {
            CampaignStep::Exhausted(exhausted) => return Ok(exhausted.finalize()),
            CampaignStep::Running(pending) => match pending.drive(listen, module_wasm)? {
                CampaignBlockOutcome::Resumed(ready) => campaign = ready,
                CampaignBlockOutcome::Concluded(outcome) => return Ok(outcome),
            },
        }
    }
}
