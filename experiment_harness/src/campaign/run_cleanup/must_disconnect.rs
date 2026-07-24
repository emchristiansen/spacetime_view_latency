//! A loud take-on-consume guard giving the measured client the same must-clean-up teeth the server has.

use anyhow::Result;

use crate::client::connected_client::ConnectedClient;

/// A measured [`ConnectedClient`] that *must* be disconnected before it is dropped. [`ConnectedClient`]
/// itself has no [`Drop`], so a forgotten disconnect silently leaks its background SDK message-processing
/// thread and socket — the exact hazard the linear cleanup owner exists to remove, and the one asymmetry
/// with [`RunningPinnedServer`](crate::provision::running_pinned_server::RunningPinnedServer) /
/// [`StagedModuleWasm`](crate::provision::staged_module_wasm::StagedModuleWasm), whose own `Drop`s already
/// assert they were handed off.
///
/// This guard closes that gap with the identical proven mechanism: the client is a private `Option`,
/// [`Self::disconnect`] `take()`s it (moving the inner value out through `&mut self`, so no field is moved
/// out of a `Drop` type and E0509 never fires), and [`Drop`] asserts the `Option` is `None` — i.e. the
/// client was disconnected. The `&self` accessor hands out only a borrow, so the `Option` never leaks.
pub(crate) struct MustDisconnect {
    /// `Some` while connected; taken by [`Self::disconnect`]. `Drop` requires it to be `None`.
    client: Option<ConnectedClient>,
}

impl MustDisconnect {
    /// Wrap a live client in its must-disconnect guard.
    pub(crate) fn new(client: ConnectedClient) -> Self {
        Self {
            client: Some(client),
        }
    }

    /// The live client, borrowed for the measured writes and correctness checks while it is owned.
    pub(crate) fn client(&self) -> &ConnectedClient {
        self.client
            .as_ref()
            .expect("MustDisconnect::client accessed after disconnect took the client")
    }

    /// Disconnect the client, joining its message-processing thread and surfacing any error. Takes the
    /// client out before the fallible disconnect, so [`Drop`] sees it handed off and cannot re-fire.
    pub(crate) fn disconnect(mut self) -> Result<()> {
        let client = self
            .client
            .take()
            .expect("MustDisconnect::disconnect called after the client was already taken");
        client.disconnect()
    }
}

impl Drop for MustDisconnect {
    fn drop(&mut self) {
        assert!(
            self.client.is_none(),
            "MustDisconnect dropped without an explicit disconnect(); the measured client's SDK \
             message-processing thread and socket would leak"
        );
    }
}
