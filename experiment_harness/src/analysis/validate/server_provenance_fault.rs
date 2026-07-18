//! The typed contradiction behind a
//! [`ServerProvenanceShape`](super::integrity_error::IntegrityError) failure.

use std::net::SocketAddr;
use std::path::PathBuf;

/// Why a per-run server-provenance fact has an invalid shape or relationship. These per-run facts vary
/// legitimately across runs, so they are validated only for internal shape and relationship, never for
/// homogeneity. Each mode carries its own fact-specific typed evidence — parsed socket addresses as
/// [`SocketAddr`], paths as [`PathBuf`] — with a raw `String` used only where parsing the wire value
/// into its domain type is itself the failure, or where the field (the client URL) has no domain newtype
/// and is checked against a typed expected structure.
#[derive(Debug)]
pub(crate) enum ServerProvenanceFault {
    /// The recorded process id is not the required nonzero value. `observed` is the recorded pid; the
    /// expected requirement is *any* nonzero pid — the manifest's own pid domain is
    /// [`NonZeroU32`](std::num::NonZeroU32), so zero is the sole representable violation and `observed`
    /// carries it rather than leaving it implicit in the variant name.
    ZeroPid { observed: u32 },
    /// The listen address failed to parse as an explicit `host:port`; `raw` is the unparseable wire
    /// string.
    UnparseableListenAddr { raw: String },
    /// The client URL does not equal `http://{listen_addr}`. `expected_authority` is the typed parsed
    /// listen address the URL must embed; `observed` is the raw client-URL string (no URL domain newtype
    /// exists).
    ClientUrlMismatch {
        expected_authority: SocketAddr,
        observed: String,
    },
    /// The data and keys directories are not sibling `data`/`keys` children of one root.
    DataKeysNotSiblings { data_dir: PathBuf, keys_dir: PathBuf },
    /// The resolved server executable does not equal the pinned standalone executable.
    ResolvedExeMismatch { expected: PathBuf, observed: PathBuf },
}
