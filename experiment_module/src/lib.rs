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

use std::ops::Bound;

use spacetimedb::{reducer, table, view, Identity, Query, ReducerContext, Table, ViewContext};

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
        .filter_map(|visibility| ctx.db.chronicle_message().uuid().find(visibility.message_uuid))
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

// ---------------------------------------------------------------------------
// Seeding reducers
// ---------------------------------------------------------------------------
//
// Identities are explicit parameters so a single harness client can attribute
// rows to arbitrary roles (M, M2, G) without reconnecting per identity.

#[reducer]
pub fn insert_message(ctx: &ReducerContext, id: u64, sender: Identity, payload: String) {
    ctx.db.message().insert(Message { id, sender, payload });
}

#[reducer]
pub fn insert_message_visibility(ctx: &ReducerContext, id: u64, viewer: Identity, message_uuid: u64) {
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
    ctx.db.chronicle_message().insert(ChronicleMessage { uuid, payload });
}

#[reducer]
pub fn insert_entity_owner(ctx: &ReducerContext, entity_uuid: u64, owner: Identity, record: String) {
    ctx.db.entity_owner().insert(EntityOwner {
        entity_uuid,
        owner,
        record,
    });
}
