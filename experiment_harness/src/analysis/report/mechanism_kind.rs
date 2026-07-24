//! The closed identity of each of the seven release-qualified v2.6.1 mechanism claims.

use serde::Serialize;

/// The number of release-qualified mechanism claims the report must cite (spec § "Source-grounded
/// mechanism claims": the seven numbered claims). It fixes the citation array length, so *exactly seven
/// citations* is a property of the report type rather than a runtime count.
pub(crate) const MECHANISM_CLAIM_COUNT: usize = 7;

/// The closed identity of a cited v2.6.1 mechanism claim — a typed enum, never a `String`, so a
/// misspelled or out-of-set claim identity is unrepresentable. The seven variants are exactly the seven
/// numbered claims of the spec's "Source-grounded mechanism claims" section, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum MechanismKind {
    /// Claim 1 — `impl Query<T>` lowers through `ViewResult::RawSql`/`SubscriptionPlan::compile` and
    /// records a `record_table_scan` per `plan.table_ids()`, so query predicates affect returned rows but
    /// not view-invalidation granularity.
    QueryPredicateInvalidationGranularity,
    /// Claim 2 — procedural `Vec<T>`/`Option<T>` views are accepted by `bindings-macro/src/view.rs` and
    /// execute their bodies directly.
    ProceduralViewDirectExecution,
    /// Claim 3 — generated full-key equality accessors emit `datastore_index_scan_point_bsatn` reaching
    /// `record_index_scan_point`; ranged-index point selection is the compile-time `bindings/src/table.rs`
    /// condition.
    FullKeyPointIndexScan,
    /// Claim 4 — procedural view handles expose only read-only index accessors; a full-domain unbounded
    /// range records a non-point range dependency, while a point-seek `TableIter` consumed by Rust adapters
    /// adds no read-set entry.
    ReadOnlyIndexAccessorReadSet,
    /// Claim 5 — view materialization clears and reinserts the result; view-PK metadata enables stable
    /// client identity and delete/insert-to-update pairing, not a different refresh algorithm.
    MaterializationClearReinsert,
    /// Claim 6 — byte-identical delete/reinsert operations in one transaction coalesce to zero net delta
    /// through `mut_tx.rs`'s insert/delete logic over `delete_table.rs`'s `DeleteTable`.
    DeleteReinsertCoalescing,
    /// Claim 7 — `crates/schema/src/def/validate/v10.rs:1048–1077` (`assign_query_view_primary_keys`)
    /// infers a query-builder view primary key from its returned product type when the annotation omits
    /// one; B/D and C/E return the same PK-bearing product types, so D/E are explicit-versus-inferred
    /// metadata replicates, not PK-versus-no-PK arms.
    QueryViewPkInferenceMetadataReplicates,
}

impl MechanismKind {
    /// The seven claim identities in spec order — the fixed, complete set the report cites exactly once
    /// each.
    pub(crate) const ALL: [MechanismKind; MECHANISM_CLAIM_COUNT] = [
        MechanismKind::QueryPredicateInvalidationGranularity,
        MechanismKind::ProceduralViewDirectExecution,
        MechanismKind::FullKeyPointIndexScan,
        MechanismKind::ReadOnlyIndexAccessorReadSet,
        MechanismKind::MaterializationClearReinsert,
        MechanismKind::DeleteReinsertCoalescing,
        MechanismKind::QueryViewPkInferenceMetadataReplicates,
    ];
}
