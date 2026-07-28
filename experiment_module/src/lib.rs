//! Controlled SpacetimeDB module for the view read-set experiment, built and tested against the
//! official published 2.7.0-line release (`v2.7.0-hotfix3`).
//!
//! Defines three base tables and the seven module-view arms A/B/C/D/E/F/F′ from
//! the governing historical spec, plus seeding reducers. Read-set behavior is the object of
//! study; the view *bodies* here are the experiment, so they are written in full
//! (a query-builder view body simply IS its query expression). Access-path forms
//! are sourced from `modules/sdk-test-procedural-view-pk/src/lib.rs` at v2.6.1 — a
//! provenance citation for those forms only, not the module's current runtime target.
//!
//! Also defines `EntityOwner` and `entity_owner_sender_view`: the deployable
//! sublinear-pattern spec's `EntityOwnerSenderView` candidate, an exact
//! production-composition analogue of Muninn's real `entity_owner` table and its
//! `entity_owner_view` (`callosum/callosum/src/tables/entity_owner{,_view}.rs`) —
//! same `(entity_uuid primary key, owner indexed Identity)` composition and the
//! same bare sender-equality query-builder filter, with an opaque `record` payload
//! standing in for the real `EntityRecord` enum (the payload's shape is not the
//! object of study).
//!
//! Also defines `ControlActivity`/`control_activity_sender_view` and
//! `IndexedControlActivity`/`indexed_control_activity_sender_view`: the same spec's
//! `ControlActivitySenderView` and `IndexedControlActivitySenderView` candidates for site 4,
//! analogues of Muninn's real `control_activity` table and its `control_activity_view`
//! (`callosum/callosum/src/tables/control_activity{,_view}.rs`). The two arms differ in exactly
//! one thing — whether the filtered `user_identity` column carries a btree index — because that
//! absence in production is site 4's held static finding.

use std::ops::Bound;

use spacetimedb::{
    reducer, table, view, Identity, Query, ReducerContext, Table, Timestamp, ViewContext,
};

// ---------------------------------------------------------------------------
// Base tables
// ---------------------------------------------------------------------------

#[table(accessor = message, public)]
pub struct Message {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub sender: Identity,
    pub payload: String,
}

#[table(accessor = message_visibility, public)]
pub struct MessageVisibility {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub viewer: Identity,
    // Indexed so the 2.6.1 query-builder `right_semijoin` (C/E) can name it in
    // the join closure, which only sees generated `IxCols`. See spec Type design
    // constraint and Implementation-Time Decisions.
    #[index(btree)]
    pub message_uuid: u64,
}

#[table(accessor = chronicle_message, public)]
pub struct ChronicleMessage {
    #[primary_key]
    pub uuid: u64,
    pub payload: String,
}

/// Production-composition analogue of Muninn's real `entity_owner` table: current ownership of an
/// entity, keyed by the entity and indexed by owner exactly as production is. `record` is an opaque
/// stand-in for production's `EntityRecord` enum — the view's read-set behavior, not the record's
/// internal shape, is the object of study.
#[table(accessor = entity_owner, public)]
pub struct EntityOwner {
    #[primary_key]
    pub entity_uuid: u64,
    #[index(btree)]
    pub owner: Identity,
    pub record: String,
}

/// Production-composition analogue of Muninn's real `control_activity` table: one row per Control
/// entity appearing in a chronicle message's source or target, carrying the message timestamp, the
/// Control's uuid, and the identity of the user who owns that Control's UX
/// (`callosum/callosum/src/tables/control_activity.rs`).
///
/// **`user_identity` deliberately carries no index, exactly as production does.** That absence is
/// what site 4 studies: the sender-scoped view below filters on this column, while the analogous
/// production `message_visibility.viewer` *is* indexed. [`IndexedControlActivity`] is the same
/// composition with that one index added, and the two tables exist separately only because an
/// index is a property of the table, not of the view that reads it.
///
/// Two departures from production's declaration, both following departures the arms above already
/// make from the real tables they mirror, so neither is new to this module: the primary
/// key is harness-assigned rather than `#[auto_inc]` (production's `message_visibility.id` is
/// auto-inc while this module's [`MessageVisibility`] takes an explicit `id`), and `control_uuid`
/// is a `u64` stand-in for production's `Uuid` (as `message_uuid` and `entity_uuid` already are).
/// Neither the key sequence nor the uuid width is part of the read set under study.
#[table(accessor = control_activity, public)]
pub struct ControlActivity {
    #[primary_key]
    pub id: u64,
    pub ts: Timestamp,
    pub control_uuid: u64,
    pub user_identity: Identity,
}

/// [`ControlActivity`] with exactly one difference: a btree index on `user_identity`, the column
/// the sender-scoped view filters on. Every other column, its order, its type, and the primary key
/// are identical, so a comparison between the two arms varies the index and nothing else.
///
/// This table has no production counterpart — production's `control_activity.user_identity` is
/// unindexed. It is the site 4 `IndexedControlActivitySenderView` candidate: the experiment-module
/// analogue of the minimal useful sender index the spec proposes for control activity.
#[table(accessor = indexed_control_activity, public)]
pub struct IndexedControlActivity {
    #[primary_key]
    pub id: u64,
    pub ts: Timestamp,
    pub control_uuid: u64,
    #[index(btree)]
    pub user_identity: Identity,
}

// ---------------------------------------------------------------------------
// Views — the seven arms
// ---------------------------------------------------------------------------

/// Arm A — procedural `Vec<Row>` full-domain unbounded range over the same
/// single-column `sender` btree as F. Passing explicit unbounded lower and upper
/// bounds is a non-point range scan (not `B::POINT`), so it records a full-domain
/// range dependency; 2.6.1 view handles expose no table `.iter()`/`.count()`.
#[view(accessor = message_range_view, public)]
pub fn message_range_view(ctx: &ViewContext) -> Vec<Message> {
    ctx.db
        .message()
        .sender()
        .filter((Bound::<Identity>::Unbounded, Bound::Unbounded))
        .collect()
}

/// Arm B — query-builder `impl Query<Row>` full pass-through (Anton's current form).
#[view(accessor = message_query_view, public)]
pub fn message_query_view(ctx: &ViewContext) -> impl Query<Message> {
    ctx.from.message()
}

/// Arm C — query-builder Papaya-shaped viewer-equality + message-UUID semijoin.
#[view(accessor = chronicle_query_view, public)]
pub fn chronicle_query_view(ctx: &ViewContext) -> impl Query<ChronicleMessage> {
    ctx.from
        .message_visibility()
        .r#where(|visibility| visibility.viewer.eq(ctx.sender()))
        .right_semijoin(ctx.from.chronicle_message(), |visibility, message| {
            visibility.message_uuid.eq(message.uuid)
        })
}

/// Arm D — arm B plus an explicit custom view primary key.
#[view(accessor = message_query_pk_view, public, primary_key = id)]
pub fn message_query_pk_view(ctx: &ViewContext) -> impl Query<Message> {
    ctx.from.message()
}

/// Arm E — arm C plus an explicit custom view primary key; identical query body.
#[view(accessor = chronicle_query_pk_view, public, primary_key = uuid)]
pub fn chronicle_query_pk_view(ctx: &ViewContext) -> impl Query<ChronicleMessage> {
    ctx.from
        .message_visibility()
        .r#where(|visibility| visibility.viewer.eq(ctx.sender()))
        .right_semijoin(ctx.from.chronicle_message(), |visibility, message| {
            visibility.message_uuid.eq(message.uuid)
        })
}

/// Arm F — procedural bounded point-filter over a single-column btree index.
#[view(accessor = messages_point_view, public)]
pub fn messages_point_view(ctx: &ViewContext) -> Vec<Message> {
    ctx.db.message().sender().filter(ctx.sender()).collect()
}

/// Arm F′ — procedural hand-written semijoin: viewer point-filter, then one PK
/// `.find` per visibility row.
#[view(accessor = chronicle_point_view, public)]
pub fn chronicle_point_view(ctx: &ViewContext) -> Vec<ChronicleMessage> {
    ctx.db
        .message_visibility()
        .viewer()
        .filter(ctx.sender())
        .filter_map(|visibility| {
            ctx.db
                .chronicle_message()
                .uuid()
                .find(visibility.message_uuid)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Views — deployable candidates (spec c33f2e51)
// ---------------------------------------------------------------------------

/// `EntityOwnerSenderView` — exact production-composition sender-scoped view for entity
/// ownership. Query-builder single-table owner-equality filter, no semijoin — the bare form of
/// production's `entity_owner_view` (`callosum/callosum/src/tables/entity_owner_view.rs`).
#[view(accessor = entity_owner_sender_view, public)]
pub fn entity_owner_sender_view(ctx: &ViewContext) -> impl Query<EntityOwner> {
    ctx.from
        .entity_owner()
        .r#where(|row| row.owner.eq(ctx.sender()))
}

/// `ControlActivitySenderView` — the unindexed arm, a bare transcription of production's
/// `control_activity_view` (`callosum/callosum/src/tables/control_activity_view.rs`): every user,
/// admin included, sees only their own activity, expressed as a single-table sender-equality
/// filter over a column with no index.
///
/// The spec records this candidate `DiagnosticOnly`: it may explain mechanism but can never be
/// recommended.
#[view(accessor = control_activity_sender_view, public)]
pub fn control_activity_sender_view(ctx: &ViewContext) -> impl Query<ControlActivity> {
    ctx.from
        .control_activity()
        .r#where(|row| row.user_identity.eq(ctx.sender()))
}

/// `IndexedControlActivitySenderView` — the indexed arm. Its query structure is the same
/// single-table sender-equality predicate on `user_identity` as [`control_activity_sender_view`];
/// what differs is the backing table it names — and hence the accessor and result type — which
/// carries a btree index on that filtered column.
#[view(accessor = indexed_control_activity_sender_view, public)]
pub fn indexed_control_activity_sender_view(
    ctx: &ViewContext,
) -> impl Query<IndexedControlActivity> {
    ctx.from
        .indexed_control_activity()
        .r#where(|row| row.user_identity.eq(ctx.sender()))
}

// ---------------------------------------------------------------------------
// Seeding reducers
// ---------------------------------------------------------------------------
//
// Identities are explicit parameters so a single harness client can attribute
// rows to arbitrary roles (M, M2, G) without reconnecting per identity.

#[reducer]
pub fn insert_message(ctx: &ReducerContext, id: u64, sender: Identity, payload: String) {
    ctx.db.message().insert(Message {
        id,
        sender,
        payload,
    });
}

#[reducer]
pub fn insert_message_visibility(
    ctx: &ReducerContext,
    id: u64,
    viewer: Identity,
    message_uuid: u64,
) {
    // SpacetimeDB 2.6.1 cannot express a composite (viewer, message_uuid) UNIQUE
    // constraint — the table macro derives uniqueness only from single-field
    // `#[unique]`/`#[primary_key]`, and neither column is individually unique — so
    // enforce pair uniqueness fail-fast at this sole insertion boundary via a point
    // lookup on the `message_uuid` btree (no scan). A reducer panic rolls the
    // transaction back, so a duplicate pair is never inserted.
    let duplicate = ctx
        .db
        .message_visibility()
        .message_uuid()
        .filter(message_uuid)
        .any(|existing| existing.viewer == viewer);
    assert!(
        !duplicate,
        "duplicate visibility pair (viewer={viewer:?}, message_uuid={message_uuid})"
    );
    ctx.db.message_visibility().insert(MessageVisibility {
        id,
        viewer,
        message_uuid,
    });
}

#[reducer]
pub fn insert_chronicle_message(ctx: &ReducerContext, uuid: u64, payload: String) {
    ctx.db
        .chronicle_message()
        .insert(ChronicleMessage { uuid, payload });
}

#[reducer]
pub fn insert_entity_owner(
    ctx: &ReducerContext,
    entity_uuid: u64,
    owner: Identity,
    record: String,
) {
    ctx.db.entity_owner().insert(EntityOwner {
        entity_uuid,
        owner,
        record,
    });
}

/// Seeds one [`ControlActivity`] row. `ts` is an explicit parameter rather than `ctx.timestamp`
/// because production's `control_activity_update` likewise receives the originating chronicle
/// message's timestamp rather than reading the clock, and because a seeded row must be
/// reproducible from the seed alone.
///
/// No measured-mutation reducer accompanies these two tables. Production only ever *inserts* into
/// `control_activity` — `callosum/callosum/src/tables/insert_chronicle_message/control_activity_update.rs`
/// inserts at three sites and Callosum contains no update or delete of that table — so the
/// owner-preserving in-place update frozen for `EntityOwnerSenderView` has no counterpart here,
/// and the spec's requirement of a *production-representative fixed-cardinality* mutation is not
/// yet satisfiable for this candidate. The spec requires that mutation be frozen before this
/// candidate's first measured run; seeding and view composition do not depend on it.
#[reducer]
pub fn insert_control_activity(
    ctx: &ReducerContext,
    id: u64,
    ts: Timestamp,
    control_uuid: u64,
    user_identity: Identity,
) {
    ctx.db.control_activity().insert(ControlActivity {
        id,
        ts,
        control_uuid,
        user_identity,
    });
}

/// Seeds one [`IndexedControlActivity`] row — the indexed arm's twin of
/// [`insert_control_activity`], identical in every argument, so both arms can be seeded from one
/// row generator without a per-arm branch in the caller.
#[reducer]
pub fn insert_indexed_control_activity(
    ctx: &ReducerContext,
    id: u64,
    ts: Timestamp,
    control_uuid: u64,
    user_identity: Identity,
) {
    ctx.db
        .indexed_control_activity()
        .insert(IndexedControlActivity {
            id,
            ts,
            control_uuid,
            user_identity,
        });
}

/// The fixed-cardinality measured mutation for the `EntityOwnerSenderView` candidate: replace an
/// existing row's `record` in place, preserving its owner, keyed by the `entity_uuid` primary key.
///
/// **There is no `owner` parameter, and that is the contract.** The measured channels change the
/// payload and nothing else: cardinality is held at the scale point, and the row must stay inside
/// the same identity's read set for E2's "observable in the subscriber cache" to mean what it says.
/// An `owner` argument would make ownership transfer representable in the measured write — a
/// different operation with a different read-set effect, since it moves the row *out* of one
/// sender's view and into another's. Reading the existing owner back and writing it unchanged makes
/// that state unreachable rather than merely unused, so no future caller can measure a transfer by
/// mistake. Ownership transfer is a real production semantic and belongs to the security/semantics
/// gates, under its own reducer, not to the trend channels.
///
/// Fixed-cardinality is the rest of the point. `insert` on an existing key violates the constraint
/// outright. Caller-managed replacement — a delete reducer followed by an insert reducer — would
/// hold cardinality only *between* transactions: those commit separately, so subscribers observe
/// the row genuinely absent in between, and a failure after the first leaves the slice one row
/// short. This reducer is one atomic transaction: the `update` does lower to a delete plus an
/// insert in transaction state, as the pinned source establishes, but that pair commits together,
/// so no intermediate absence is ever externally observable and cardinality is constant at every
/// committed state.
///
/// **Fail-loud on a missing row.** The lookup is required anyway to recover the owner, so the
/// absent case is caught here with the offending key named, rather than reaching the pinned
/// `update`'s own missing-row panic. Either way a reducer panic rolls the transaction back and
/// surfaces as a reducer error on the caller's completion callback. A silent upsert is what must
/// not happen: it would insert an unseeded key, raising cardinality mid-measurement, and the
/// composition check would then be judging a scale point the attempt never held.
///
/// **What the lookup adds to the measured path.** Preserving the owner requires one primary-key
/// lookup inside the measured transaction, so that lookup's cost is deliberately part of what the
/// E1 and E2 channels measure and is not separable from the update. That is the
/// production-representative shape — a real ownership-preserving update reads the row it replaces —
/// and it must stay visible in provenance and in how the results are interpreted. What it does to
/// the measured numbers over the frozen finite range is for the experiment to determine, not for
/// this comment to assert.
#[reducer]
pub fn update_entity_owner(ctx: &ReducerContext, entity_uuid: u64, record: String) {
    let Some(existing) = ctx.db.entity_owner().entity_uuid().find(entity_uuid) else {
        panic!("update_entity_owner: no entity_owner row with entity_uuid={entity_uuid}");
    };
    ctx.db.entity_owner().entity_uuid().update(EntityOwner {
        entity_uuid,
        owner: existing.owner,
        record,
    });
}
