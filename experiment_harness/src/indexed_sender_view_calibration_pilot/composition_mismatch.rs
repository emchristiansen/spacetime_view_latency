//! Every way an attempt's observed rows failed their preregistered populations.

use anyhow::{ensure, Result};
use serde::Serialize;

/// A nonempty list of the exact ways an attempt's rows diverged from the frozen composition.
///
/// **Nonempty by construction**, which is the same defect the discovery screen's semantic-failure
/// invariant closes from the other side: a mismatch carrying no mismatches would put a *passing*
/// composition check into the ledger under a failing verdict, and a reader would have no way to tell
/// it from a real one.
///
/// Retained in full on the failure record rather than summarised, because which rows diverged and
/// how is the entire diagnostic content of a composition failure. An arm short by one row is a lost
/// append; an arm at the right count whose rows carry another identity is a **sender-scope leak**;
/// a witness short by a thousand is an unseeded backing table. Those are three different faults, and
/// a count would render all three identically.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub(crate) struct CompositionMismatch(Vec<String>);

impl CompositionMismatch {
    /// Retain the faults a verification found, failing loud if handed none.
    pub(crate) fn of(faults: Vec<String>) -> Result<Self> {
        ensure!(
            !faults.is_empty(),
            "a composition mismatch must name at least one fault; an empty list is a passing check \
             being recorded as a failing one"
        );
        Ok(Self(faults))
    }

    /// The faults, for a diagnostic message.
    pub(crate) fn faults(&self) -> &[String] {
        &self.0
    }
}
