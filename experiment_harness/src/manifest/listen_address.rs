//! The explicit, validated listen address of the isolated experiment server.

use std::net::SocketAddr;
use std::str::FromStr;

use anyhow::{Context, Result};

/// The explicit `host:port` the isolated experiment standalone binds to, parsed to a
/// [`SocketAddr`] so a malformed or empty address fails fast (spec: "an explicit
/// non-conflicting listen address"). No default is assumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ListenAddress(SocketAddr);

impl ListenAddress {
    /// Parse an explicit `host:port` listen address.
    pub(crate) fn parse(raw: &str) -> Result<Self> {
        let addr = SocketAddr::from_str(raw)
            .with_context(|| format!("invalid listen address {raw:?}; expected explicit host:port"))?;
        Ok(Self(addr))
    }

    /// The socket address.
    pub(crate) fn socket_addr(&self) -> SocketAddr {
        self.0
    }

    /// The HTTP client URL derived from the listen address (spec: derive the client URL
    /// from the listen address rather than accepting a separate, possibly inconsistent
    /// URL).
    pub(crate) fn client_url(&self) -> String {
        format!("http://{}", self.0)
    }
}
