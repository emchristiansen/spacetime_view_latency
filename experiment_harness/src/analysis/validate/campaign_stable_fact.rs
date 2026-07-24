//! The closed discriminator naming which campaign-stable manifest fact a stable-provenance failure
//! concerns.

/// Which campaign-stable manifest field a stable-provenance failure concerns, as a closed typed
/// discriminator. Every field the source proves campaign-stable is enumerated individually — validation
/// never compares whole provenance blocks, so a failure always names the exact field. This discriminator
/// is the category of a
/// [`StableFactContradiction`](super::stable_fact_contradiction::StableFactContradiction) (see
/// [`StableFactContradiction::fact`](super::stable_fact_contradiction::StableFactContradiction::fact))
/// and is carried directly when parsing the raw wire value into its domain type is itself the failure, so
/// no domain value exists to name the field.
///
/// Deliberately excluded, with reasons:
/// - **Per-run facts** — the module database identity, server pid, listen address, client URL, and
///   data/keys directories vary legitimately per run and are validated only for shape and binding,
///   never for homogeneity.
/// - **`resolved_exe`** — campaign-stable, but its homogeneity is established *transitively*: each run
///   proves `resolved_exe == standalone_exe`
///   ([`ServerProvenanceFault::ResolvedExeMismatch`](super::server_provenance_fault::ServerProvenanceFault)),
///   and [`Self::StandaloneExe`] is proven homogeneous, so no separate homogeneity comparison is needed.
/// - **Pinned distribution identity** — the source distribution facts carry no such field, so none
///   appears here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CampaignStableFact {
    NixStoreBinDir,
    CliExe,
    CliVersion,
    CliReleaseCommit,
    CliVersionRaw,
    StandaloneExe,
    StandaloneVersion,
    StandaloneVersionRaw,
    WasmSha256,
    ScheduleSeed,
    BatchSize,
    NumDoses,
    DoseLadder,
    BatchDelayMs,
    RepetitionBlocks,
    ConfirmedReads,
}
