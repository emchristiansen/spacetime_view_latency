//! Validation of one attempt's observed result set against what its role must return.
//!
//! The spec requires, at every scale point, the exact expected backing-table composition, the exact
//! expected delivery, and visibility of the measured mutation — and for the Arm additionally that
//! the sender-scoped view leaks none of the foreign slice. This module is where those claims are
//! checked.
//!
//! **Who can construct what.** [`expected_composition::ExpectedComposition`] and
//! [`expected_foreign_visibility::ExpectedForeignVisibility`] are freely constructible, because they
//! are claims to be checked rather than findings; no guarantee attaches to them and none is implied.
//! [`validated_composition::ValidatedComposition`] has entirely private fields and a single
//! constructor declared in its own file, so no other module — sibling, parent, or elsewhere in the
//! crate — can mint one without going through the comparison. Its nested digest and mutation
//! payloads are private types in that same file, so they cannot be assembled independently either.
//!
//! **What makes a finding auditable.** A digest cannot be inverted and counts cannot re-run an
//! owner, payload, or key-range comparison, so the ledger alone would never be enough. The observed
//! rows are therefore retained on disk as content-addressed
//! [`observed_row_set::ObservedRowSet`] artifacts — path, algorithm, canonicalization domain,
//! digest, and row count — and the validated finding embeds its expectation, its per-range counts,
//! and the mutated row's key and both payloads while pointing at those artifacts. A reader can fetch
//! the exact rows and redo every comparison rather than trusting that a validator once returned
//! `Ok`.
//!
//! One public entity per file; this entry file is declarative re-exports only.

pub(crate) mod digest_algorithm;
pub(crate) mod expected_composition;
pub(crate) mod expected_foreign_visibility;
pub(crate) mod observed_row_set;
pub(crate) mod validated_composition;
