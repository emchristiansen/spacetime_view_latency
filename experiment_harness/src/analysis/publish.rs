//! Analyze's corrected-v2 bundle publication: deterministic paths, and the affine
//! staged → rename-committed → durably-published state chain that makes an invalid publication
//! ordering unrepresentable.
//!
//! One public entity per file; this entry file is declarative module declarations only.

pub(crate) mod corrected_v2_paths;
pub(crate) mod corrected_v2_staging;
pub(crate) mod corrected_v2_staging_create_error;
pub(crate) mod indeterminate_publication;
pub(crate) mod promotion_failure;
pub(crate) mod published_corrected_v2;
pub(crate) mod rename_committed;
