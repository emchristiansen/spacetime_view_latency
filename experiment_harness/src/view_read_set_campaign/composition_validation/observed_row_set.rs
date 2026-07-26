//! One durably retained observation of a subscriber's whole result set.
//!
//! The type lives in the private, *childless* inline module [`sealed`] because its content address is
//! only meaningful if the code that recorded it is the code that wrote and hashed the rows. A private
//! field is visible to its declaring module **and every descendant**, so a `#[cfg(test)] mod tests`
//! child — or any child added later — could write the struct literal and pair a path with a digest of
//! something else entirely. `sealed` has no children, so [`ObservedRowSet::persisted`] really is the
//! only door.
//!
//! [`OBSERVED_ROW_SET_DOMAIN`] stays outside `sealed`: it is a frozen preregistered constant, read by
//! [`CampaignParameters`](crate::view_read_set_campaign::campaign_parameters::CampaignParameters),
//! and a `&str` constant carries no invariant that sole minting could protect.

/// The canonicalization domain observed row sets are encoded and digested under.
///
/// Frozen because changing it changes every recorded digest, so it belongs with the preregistered
/// constants rather than inline at a hashing call site.
pub(crate) const OBSERVED_ROW_SET_DOMAIN: &str = "view-read-set-experiment:observed-row-set";

mod sealed {
    use std::path::{Path, PathBuf};

    use anyhow::Result;
    use serde::Serialize;

    use crate::module_artifact::bindings::EntityOwner;
    use crate::view_read_set_campaign::composition_validation::digest_algorithm::DigestAlgorithm;

    /// The complete set of rows one subscriber held at one moment, written to disk and
    /// content-addressed.
    ///
    /// **Why the rows are retained and not just summarized.** A digest cannot be inverted, and counts
    /// cannot re-run an owner, payload, or key-range comparison. Without the raw rows on disk, a
    /// composition finding in the ledger could only ever be *believed*; with them, it can be
    /// recomputed and disputed. So this is the durable artifact, and the validated finding merely
    /// points at it.
    ///
    /// **Who can construct it.** Every field is private to this childless module and
    /// [`Self::persisted`] is the only constructor, declared alongside them here. Nothing else in the
    /// crate can pair a path with a digest it did not compute, because the constructor is the code
    /// that writes the rows and hashes what it wrote.
    ///
    /// **What is serialized.** The path, algorithm, canonicalization domain, digest, and row count —
    /// the content address. The rows themselves are skipped: they live at `path`, which is the point.
    #[derive(Debug, Clone, Serialize)]
    pub(crate) struct ObservedRowSet {
        path: PathBuf,
        algorithm: DigestAlgorithm,
        domain: &'static str,
        sha256_hex: String,
        row_count: u64,
        #[serde(skip)]
        rows: Vec<EntityOwner>,
    }

    impl ObservedRowSet {
        /// Canonicalize `rows`, write them under `directory`, digest exactly what was written, and
        /// return the content address alongside the rows.
        ///
        /// **Phase 1 boundary.** The canonicalization is frozen in this doc and implemented in Phase
        /// 2: rows sorted ascending by `entity_uuid` — the table's primary key, so the order is total
        /// — and encoded one per line as `entity_uuid:owner_hex:record`, with the whole text digested
        /// under
        /// [`OBSERVED_ROW_SET_DOMAIN`](crate::view_read_set_campaign::composition_validation::observed_row_set::OBSERVED_ROW_SET_DOMAIN).
        /// Sorting by key rather than by arrival order is deliberate: two subscribers that received
        /// the same rows in different orders must produce the same address.
        pub(crate) fn persisted(
            directory: &Path,
            label: &str,
            rows: Vec<EntityOwner>,
        ) -> Result<Self> {
            let _ = (directory, label, rows);
            todo!(
                "Phase 2: sort by entity_uuid, encode one row per line as \
                 entity_uuid:owner_hex:record, write to <directory>/<label>.rows under O_EXCL so an \
                 observation can never overwrite another, then SHA-256 the written bytes under \
                 OBSERVED_ROW_SET_DOMAIN and record the path, digest, and row count"
            )
        }

        /// The observed rows, for a validator that is checking them right now. Not serialized — a
        /// reader gets them from [`Self::path`] instead, which is what makes the finding re-runnable
        /// later.
        pub(crate) fn rows(&self) -> &[EntityOwner] {
            &self.rows
        }

        /// Where the canonicalized rows were written.
        pub(crate) fn path(&self) -> &Path {
            &self.path
        }

        /// The content address of the written rows, as `(algorithm, domain, hex, row_count)`.
        pub(crate) fn content_address(&self) -> (DigestAlgorithm, &'static str, &str, u64) {
            (
                self.algorithm,
                self.domain,
                &self.sha256_hex,
                self.row_count,
            )
        }
    }
}

pub(crate) use sealed::ObservedRowSet;
