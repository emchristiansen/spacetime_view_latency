//! One coherent reading of the SDK's per-database frame histogram, and the window arithmetic.

use anyhow::{ensure, Context, Result};

use crate::view_read_set_campaign::subscriber_delivery::received_wire_delivery::ReceivedWireDelivery;

/// `2^53` — the point above which consecutive integers stop being exactly representable in `f64`.
///
/// The byte figure is a Prometheus histogram *sum*: an accumulating `f64` of frame lengths, each of
/// which entered as an exactly-representable `usize as f64`. The running sum stays exact only while
/// it is at most this, and past it an addition can silently round — which is why an accumulated
/// absolute reading must be checked, not only the difference between two of them.
const EXACT_INTEGER_LIMIT: f64 = 9_007_199_254_740_992.0;

/// What the SDK's `websocket_received_msg_size` histogram had accumulated for one database at one
/// instant.
///
/// **One instant, and that is load-bearing.** Both numbers come from a single collect, which is why
/// this type carries the pair and never a separately-read frame counter; how that collect is made
/// coherent is at [`subscriber_delivery_meter`](super::subscriber_delivery_meter). The arithmetic
/// below assumes only that each field is cumulative and that the two describe the same instant.
///
/// **Deliberately not sealed, because this is not evidence.** Its fields are named and crate-visible,
/// it holds no invariant, and constructing one asserts nothing about any connection — it is the
/// input to [`Self::since`], the same shape and for the same reason as
/// [`ObservedRuntimePins`](crate::view_read_set_campaign::observed_runtime_pins::ObservedRuntimePins).
/// That is not a weakening. Delivery evidence is minted only by consuming a live meter (see
/// [`subscriber_delivery_meter`](super::subscriber_delivery_meter)), which takes both of its own
/// readings off the live histogram and accepts neither as a parameter. What being constructible buys
/// is that the arithmetic below — the part that can actually be wrong — is testable without a
/// server.
pub(crate) struct WireCounterSnapshot {
    /// The histogram's cumulative sample count: one observation per frame the connection received.
    pub(crate) frames: u64,
    /// The histogram's cumulative sum of compressed frame lengths, coherent with `frames`.
    pub(crate) byte_sum: f64,
}

impl WireCounterSnapshot {
    /// Reduce the window that ends at `self` and began at `open` to its exact received traffic,
    /// failing loud on any reading that cannot be converted exactly.
    ///
    /// **Why the absolute readings are validated before the subtraction.** The histogram accumulates
    /// for the life of the process, so a byte sum that has already left the consecutive-integer range
    /// yields a *small, plausible-looking* difference that is quietly wrong. Checking only the delta
    /// would accept it. Both ends therefore go through the same gate first, and the difference goes
    /// through it again: once both absolutes are exact integers at most [`EXACT_INTEGER_LIMIT`],
    /// their `f64` difference is itself exact, so the second pass's finite, integral, and range
    /// clauses can no longer fire and its live clause is non-negativity — a close reading below the
    /// open one, which means the series was reset rather than that a window elapsed.
    ///
    /// Frames are subtracted the same way, and a decrease is refused for the same reason.
    pub(crate) fn since(&self, open: &Self) -> Result<ReceivedWireDelivery> {
        let frames = self.frames.checked_sub(open.frames).with_context(|| {
            format!(
                "the histogram counted {} frames at close but {} at open; a cumulative count only \
                 ever increases, so this window spans a reset rather than a run of received frames",
                self.frames, open.frames,
            )
        })?;

        let opened = exact_byte_count(open.byte_sum, "at the meter's open")?;
        let closed = exact_byte_count(self.byte_sum, "at the meter's close")?;
        let bytes = exact_byte_count(self.byte_sum - open.byte_sum, "over the measured window")
            .with_context(|| {
                format!("the byte histogram summed {opened} at open and {closed} at close")
            })?;

        Ok(ReceivedWireDelivery { frames, bytes })
    }
}

/// Convert one accumulated byte sum to the exact whole number of bytes it represents, or fail
/// naming which reading was unusable.
///
/// Four clauses, in the order that makes each one's message true: a non-finite value is not
/// comparable, a negative one is not a count of bytes, a fractional one did not come from whole
/// frame lengths, and one past [`EXACT_INTEGER_LIMIT`] may already have been rounded by its own
/// accumulation. Only after all four does `as u64` truncate nothing.
fn exact_byte_count(sum: f64, reading: &str) -> Result<u64> {
    ensure!(
        sum.is_finite(),
        "the received-byte sum {reading} is {sum}, which is not a finite number",
    );
    ensure!(
        sum >= 0.0,
        "the received-byte sum {reading} is {sum}; a count of bytes received is never negative",
    );
    ensure!(
        sum.fract() == 0.0,
        "the received-byte sum {reading} is {sum}, not a whole number of bytes, though every \
         observation added to it is a frame length in bytes",
    );
    ensure!(
        sum <= EXACT_INTEGER_LIMIT,
        "the received-byte sum {reading} is {sum}, above the {EXACT_INTEGER_LIMIT} at which an f64 \
         stops representing consecutive integers exactly; it may already have been rounded, so no \
         exact byte count can be recovered from it",
    );
    Ok(sum as u64)
}

#[cfg(test)]
mod tests;
