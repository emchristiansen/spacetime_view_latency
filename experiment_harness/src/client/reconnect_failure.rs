//! What a token-preserving reconnect left behind when it could not complete.

use anyhow::Error;

use crate::client::connected_client::ConnectedClient;
use crate::client::measured_step_failure::MeasuredStepFailure;

/// The release state of a failed reconnect, carried by the error itself.
///
/// [`ConnectedClient::reconnect_preserving_token`] consumes the old client and releases it *before*
/// initiating the new one, so there is no `&mut self` to leave valid and no dummy or `Option` client
/// anywhere. Each variant is a total answer to what the caller still owns: one live client, or
/// nothing.
///
/// **No variant means a connection may still be running.** Every release path joins the
/// message-processing thread, and a join returns only once that thread has ended, so
/// [`Self::ReleaseFailed`] is an abnormal *ending* — still campaign-terminal, but not a leak.
pub(crate) enum ReconnectFailure {
    /// A connection this reconnect owned — the old one, or a replacement whose handshake failed
    /// after its thread had started — ended abnormally while being released. No client remains; the
    /// carried error is a resource-release failure, kept distinct from the attempt's measured cause.
    ReleaseFailed(Error),
    /// The old connection was released cleanly and no replacement is running.
    NotConnected(MeasuredStepFailure),
    /// A new connection was established and **is owned by this value**, but its resubscription did
    /// not apply. The caller must release it exactly once.
    NotResubscribed {
        client: ConnectedClient,
        failure: MeasuredStepFailure,
    },
}
