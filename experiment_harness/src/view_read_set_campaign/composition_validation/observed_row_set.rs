//! One durably retained observation of a subscriber's whole result set.
//!
//! The type lives in the private, *childless* inline module [`sealed`] because its content address is
//! only meaningful if the code that recorded it is the code that wrote and hashed the rows. A private
//! field is visible to its declaring module **and every descendant**, so a `#[cfg(test)] mod tests`
//! child — or any child added later — could write the struct literal and pair a path with a digest of
//! something else entirely. `sealed` has no children, so [`ObservedRowSet::persisted`] really is the
//! only door.
//!
//! [`OBSERVED_ROW_SET_DOMAIN`] and [`canonical_encoding`] stay outside `sealed` for the same reason:
//! the constant is a frozen preregistration read by
//! [`CampaignParameters`](crate::view_read_set_campaign::campaign_parameters::CampaignParameters),
//! and neither a `&str` constant nor a pure total function carries an invariant that sole minting
//! could protect. Splitting the encoding out is what lets the frozen byte shape be tested without a
//! filesystem, leaving [`ObservedRowSet::persisted`] to be tested for exactly what it adds:
//! exclusive creation, and a digest that addresses the bytes it wrote.

use anyhow::{ensure, Result};

use crate::module_artifact::bindings::EntityOwner;

/// The canonicalization domain observed row sets are encoded and digested under.
///
/// Frozen because changing it changes every recorded digest, so it belongs with the preregistered
/// constants rather than inline at a hashing call site.
///
/// It **names** the convention; it is not a hash prefix. The recorded digest is SHA-256 of exactly
/// the persisted bytes, with no unrecorded framing, so a cold reader recomputes it directly from the
/// artifact path and uses this domain to select the parser. A domain folded into the preimage would
/// make the recorded digest unreproducible from the retained file, which is the one thing these
/// artifacts exist to permit.
pub(crate) const OBSERVED_ROW_SET_DOMAIN: &str = "view-read-set-experiment:observed-row-set";

/// The frozen extension every retained row set artifact carries.
///
/// A constant rather than a literal at the join, because it is part of the recorded artifact path a
/// reader locates evidence by, exactly as the phase stems are.
pub(crate) const ROW_SET_EXTENSION: &str = ".rows";

/// The frozen canonical text of an observed row set: rows in primary-key order, one per line as
/// `entity_uuid:owner_hex:record`, every line terminated by `\n`.
///
/// **Sorted by key, not by arrival.** `entity_uuid` is the table's primary key, so distinct rows
/// have distinct keys and the order is total. Two subscribers that received the same rows in
/// different orders must produce the same content address, which is only true if the order is
/// recomputed here rather than inherited.
///
/// **Terminated, not separated.** The final row ends in `\n` like every other, so `wc -l` equals the
/// recorded row count and no reader has to special-case the last line. Zero rows produce an empty
/// file rather than a blank line, which stays consistent with that: zero lines, zero rows.
///
/// **Line breaks in a record are rejected rather than escaped.** A record carrying `\n` or `\r`
/// would let one logical row forge two artifact lines — a row set that hashes and re-parses as a
/// composition it never held. Failing here is what keeps "one line, one row" true of every artifact
/// that exists, and no measured or seeded payload can contain either byte, so a rejection is
/// evidence of a fault rather than of a legitimate payload this encoding cannot express.
///
/// Colons are *not* rejected, because payloads legitimately contain them
/// (`view-read-set-campaign-mutation:<tag>:<index>`): a reader splits from the left into exactly
/// three fields, which is unambiguous because the first two are a `u64` and a fixed-width canonical
/// hex identity.
/// **Duplicate keys are rejected after sorting.** `entity_uuid` is the table's primary key, so two
/// rows sharing one is a state the database cannot hold — but this function takes an ordinary slice,
/// and a caller or a faulty cache snapshot can supply one anyway. Two equal keys would also make the
/// sort's tie-breaking, and therefore the content address, depend on the arrival order this encoding
/// exists to erase. Failing loud is what keeps an impossible table state from acquiring an artifact
/// and a digest that make it look observed.
fn canonical_encoding(rows: &[EntityOwner]) -> Result<String> {
    let mut ordered: Vec<&EntityOwner> = rows.iter().collect();
    ordered.sort_by_key(|row| row.entity_uuid);

    for pair in ordered.windows(2) {
        ensure!(
            pair[0].entity_uuid != pair[1].entity_uuid,
            "two observed rows share entity_uuid={}, which the table's primary key makes \
             impossible; the observation is not a state any server held",
            pair[0].entity_uuid,
        );
    }

    let mut text = String::new();
    for row in ordered {
        ensure!(
            !row.record.contains('\n') && !row.record.contains('\r'),
            "the record of row entity_uuid={} contains a line break, which would forge additional \
             artifact lines; no seeded or measured payload contains one",
            row.entity_uuid,
        );
        text.push_str(&format!(
            "{}:{}:{}\n",
            row.entity_uuid,
            row.owner.to_hex(),
            row.record
        ));
    }
    Ok(text)
}

mod sealed {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::path::{Path, PathBuf};

    use anyhow::{Context, Result};
    use serde::Serialize;
    use sha2::{Digest, Sha256};

    use crate::module_artifact::bindings::EntityOwner;
    use crate::view_read_set_campaign::composition_validation::attempt_artifact_directory::AttemptArtifactDirectory;
    use crate::view_read_set_campaign::composition_validation::digest_algorithm::DigestAlgorithm;
    use crate::view_read_set_campaign::composition_validation::observed_row_set::{
        canonical_encoding, OBSERVED_ROW_SET_DOMAIN, ROW_SET_EXTENSION,
    };
    use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;

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
    ///
    /// **Why `row_count` cannot drift from `rows`, without a check.** Both are written once, from
    /// the same `Vec`, in the single struct literal inside [`Self::persisted`]. There is no other
    /// constructor, no mutator, and no setter: the fields are private to this childless module, so
    /// nothing in the crate can move one without the other. `Clone` copies the pair together, and
    /// the type derives only [`Serialize`] — there is no `Deserialize`, so no wire form can rebuild
    /// one with `#[serde(skip)]` having dropped the rows. A runtime equality check would therefore
    /// be testing the compiler rather than the data, which is why there is none.
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
        /// Canonicalize `rows`, write them into this attempt's artifact directory under `label`,
        /// digest exactly what was written, and return the content address alongside the rows.
        ///
        /// The canonical text is
        /// [`canonical_encoding`](crate::view_read_set_campaign::composition_validation::observed_row_set)'s,
        /// so what this adds is the two properties that need a filesystem to be true at all.
        ///
        /// **Both path inputs are typed, and what that does and does not prove.** The directory is
        /// an [`AttemptArtifactDirectory`], which exists only by exclusive creation from *some*
        /// [`AttemptKey`](crate::view_read_set_campaign::attempt_key::AttemptKey) under the campaign
        /// root; the label is a closed [`ObservedRowSetLabel`], so the only two filenames in
        /// existence are the two phases a transition has. Together they establish **path origin and
        /// safety** — every artifact lands inside an existing campaign root, traversal and phase
        /// misspelling are unrepresentable rather than validated away — and **cross-attempt
        /// directory-name uniqueness**, since two distinct identities cannot name one child and a
        /// repeated identity fails at that child's creation.
        ///
        /// They do **not** bind these rows to that attempt. This function receives no `AttemptKey`,
        /// and an [`EntityOwner`] carries no attempt identity, so nothing here can tell that the
        /// rows handed in were observed by the attempt whose directory they are written into.
        /// Keeping the driver's directory and its client in correspondence is a later orchestration
        /// invariant, and ultimately a provenance-validation one; it is not a property of these path
        /// types and is not claimed by them.
        ///
        /// **Exclusive creation.** The file is created `create_new` (`O_EXCL`), so an observation
        /// can never overwrite another: writing the same phase twice within one attempt is a
        /// fail-fast error rather than a silently replaced artifact whose recorded digest still
        /// names the rows it no longer holds. Follows
        /// [`FileLineWriter::create`](crate::observation::file_line_writer::FileLineWriter) in
        /// choosing exclusivity over truncation.
        ///
        /// **The digest addresses the written bytes.** The same buffer is written and hashed, with
        /// no prefix, separator, or reread: `sha256sum` of the retained file reproduces the recorded
        /// digest exactly, which is what makes a finding recomputable by a reader who has only the
        /// artifact, the algorithm, and the domain. Hashing the buffer rather than rereading the
        /// file copies
        /// [`StagedModuleWasm::load`](crate::provision::staged_module_wasm::StagedModuleWasm) —
        /// there is no window in which the two could differ, since the file is created here and
        /// nothing else holds it open.
        ///
        /// Write, flush, and `sync_data` follow the same durability contract the ledger's line
        /// writer exercises. The containing directory is not fsynced: unlike the ledger, an artifact
        /// whose directory entry did not survive a crash is a *missing* artifact, which the finding
        /// that points at it detects, rather than a torn record that reads as complete.
        pub(crate) fn persisted(
            directory: &AttemptArtifactDirectory,
            label: ObservedRowSetLabel,
            rows: Vec<EntityOwner>,
        ) -> Result<Self> {
            let stem = label.file_stem();
            let text = canonical_encoding(&rows)
                .with_context(|| format!("canonicalizing the {stem} observed row set"))?;
            let path = directory.path().join(format!("{stem}{ROW_SET_EXTENSION}"));

            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .with_context(|| {
                    format!(
                        "creating the observed row set artifact exclusively at {}; an observation \
                         never overwrites another",
                        path.display()
                    )
                })?;
            file.write_all(text.as_bytes()).with_context(|| {
                format!("writing the observed row set artifact {}", path.display())
            })?;
            file.flush().with_context(|| {
                format!("flushing the observed row set artifact {}", path.display())
            })?;
            file.sync_data().with_context(|| {
                format!("syncing the observed row set artifact {}", path.display())
            })?;

            let mut hasher = Sha256::new();
            hasher.update(text.as_bytes());
            let digest: [u8; 32] = hasher.finalize().into();

            let row_count = u64::try_from(rows.len()).expect("an observed row count fits u64");

            Ok(Self {
                path,
                algorithm: DigestAlgorithm::Sha256,
                domain: OBSERVED_ROW_SET_DOMAIN,
                sha256_hex: hex::encode(digest),
                row_count,
                rows,
            })
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

#[cfg(test)]
mod tests;
