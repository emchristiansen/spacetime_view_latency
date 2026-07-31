//! How a record relates to an earlier attempt at the same logical slot.

use anyhow::{ensure, Result};
use serde::Serialize;

use crate::control_registry_discovery_screen::retry_ordinal::RetryOrdinal;

/// Whether this record is an original attempt or supersedes an earlier one at its slot.
///
/// The spec requires supersession *links*, not merely a retry ordinal. An ordinal alone says a
/// record is the second attempt at a slot; it does not say which record it replaces, so a ledger
/// carrying an original, a failure, and a retry leaves the reader to re-derive the relationship by
/// matching keys. Analysis still selects the valid complete attempt with the lowest ordinal per
/// slot, and the ledger still displays originals, failures, and retries — the link is what makes
/// that display readable rather than reconstructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum Supersession {
    /// The first attempt at this logical slot. All sixteen predeclared attempts are originals.
    Original,
    /// A retry, naming the attempt it supersedes.
    Supersedes { superseded_retry: RetryOrdinal },
}

impl Supersession {
    /// Derive the link for an attempt at `retry`, given the ordinal it replaces.
    ///
    /// Constructed against the record's own ordinal rather than supplied freely, so an original
    /// cannot claim to supersede anything and a retry cannot claim to supersede itself or a later
    /// attempt.
    pub(crate) fn of(retry: RetryOrdinal, superseded: Option<RetryOrdinal>) -> Result<Self> {
        match (retry == RetryOrdinal::ORIGINAL, superseded) {
            (true, None) => Ok(Self::Original),
            (true, Some(_)) => {
                anyhow::bail!("an original attempt supersedes nothing, but one was named")
            }
            (false, None) => {
                anyhow::bail!("a retry must name the attempt it supersedes")
            }
            (false, Some(superseded_retry)) => {
                ensure!(
                    superseded_retry < retry,
                    "a retry may only supersede an earlier attempt at its slot"
                );
                Ok(Self::Supersedes { superseded_retry })
            }
        }
    }
}
