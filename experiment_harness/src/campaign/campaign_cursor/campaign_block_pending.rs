//! The campaign's paired block carrier: drive the block bound by a predecessor-minted [`CampaignBlockDraw`].

use std::path::Path;

use crate::campaign::block_cursor::BlockReady;
use crate::campaign::block_cursor::BlockRunOutcome;
use crate::campaign::block_cursor::BlockStep;
use crate::campaign::campaign_aborted::CampaignAborted;
use crate::manifest::listen_address::ListenAddress;

use super::campaign_ready::CampaignBlockDraw;
use super::CampaignBlockOutcome;

/// The paired carrier a campaign hands out for its next block. It owns exactly one opaque
/// [`CampaignBlockDraw`] — the predecessor-minted binding of the block's owned campaign sink, the drawn
/// block, and the campaign's private remainder (seed, remaining block iterator, and progress). It accepts
/// **no** identity-bearing loose arguments: [`Self::from_draw`] takes only the whole draw, so no caller can
/// pair this block with a foreign sink, seed, remainder, or progress. [`Self::drive`] decomposes the draw,
/// starts the block cursor *itself* (via [`BlockReady::begin`] under the continuation's own seed), loops the
/// block's own [`BlockStep`] transitions driving each concrete
/// [`BlockRunPending`](crate::campaign::block_cursor::BlockRunPending) the block hands out, and folds the
/// resulting block terminal back into the campaign through the draw's own continuation.
///
/// Because the draw is minted only inside [`CampaignReady::next_block`](super::CampaignReady) from that
/// campaign's own fields, the block cursor started here and the block terminal folded in are always this
/// campaign's own — the block/continuation match is structural, not a runtime `assert_eq!`. The carrier
/// accepts only non-identity server/Wasm configuration ([`ListenAddress`] and the module WASM path) at
/// [`Self::drive`]; it never accepts a seed, a block cursor, a block outcome, an effect closure, or a
/// terminal, so no foreign block or terminal can be injected.
pub(crate) struct CampaignBlockPending {
    draw: CampaignBlockDraw,
}

impl CampaignBlockPending {
    /// Wrap the predecessor-minted draw. Accepts only the opaque [`CampaignBlockDraw`] and nothing else, so
    /// there is no decomposed-piece seam here. `pub(in crate::campaign::campaign_cursor)` so only the
    /// campaign cursor can build a carrier; the sole implemented caller is
    /// [`CampaignReady::next_block`](super::CampaignReady), which hands the draw it just minted.
    pub(in crate::campaign::campaign_cursor) fn from_draw(draw: CampaignBlockDraw) -> Self {
        Self { draw }
    }

    /// Decompose the draw, start its exact block cursor, drive the block's `[Run; 2]` to a block terminal,
    /// and fold it into the campaign. Starts the block at [`BlockReady::begin`] under the continuation's own
    /// seed (so the block orders its runs under the same canonical seed the continuation will resume the
    /// campaign under), then loops the block's [`BlockStep`] transitions, driving each concrete
    /// [`BlockRunPending`](crate::campaign::block_cursor::BlockRunPending) the block emits; a completed run
    /// resumes the block on its own iterator, and a stopped run aborts the block — which folds through the
    /// draw's continuation into a concluded campaign. Block exhaustion folds the completed block into the
    /// campaign, resuming it. A pre-cleanup acquisition failure of any run short-circuits on the Err channel
    /// as a [`CampaignAborted`] whose sink is already finalized. `pub(in crate::campaign)` confines this to
    /// the campaign module subtree — not one caller, which the visibility does not single out; the sole
    /// implemented caller is the campaign orchestrator ([`run_campaign`](crate::campaign::run_campaign)).
    pub(in crate::campaign) fn drive(
        self,
        listen: ListenAddress,
        module_wasm: &Path,
    ) -> Result<CampaignBlockOutcome, CampaignAborted> {
        let (sink, block_run, continuation) = self.draw.into_parts();
        let mut block = BlockReady::begin(sink, block_run, continuation.seed());
        loop {
            match block.next_run() {
                BlockStep::Exhausted(done) => {
                    return Ok(CampaignBlockOutcome::Resumed(continuation.finish(done)));
                }
                BlockStep::Running(pending) => match pending.drive(listen, module_wasm)? {
                    BlockRunOutcome::Resumed(ready) => block = ready,
                    BlockRunOutcome::Aborted(incomplete) => {
                        return Ok(CampaignBlockOutcome::Concluded(
                            continuation.abort(incomplete),
                        ));
                    }
                },
            }
        }
    }
}
