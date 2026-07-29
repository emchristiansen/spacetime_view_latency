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
//!
//! Also defines `control_activity_empty_view`, the capability reproducer for site 4's second
//! question: whether the pinned release can express a genuinely empty view result without the
//! sentinel predicate production's `control_activity_view_all` resorts to.
//!
//! Also defines `control_activity_latest_by_control_view`, the bounded capability probe for site
//! 4's discovery comparator: whether a procedural view can compute latest-per-`control_uuid` over
//! the unindexed production-shaped history table at all, and whether such a view is invalidated
//! when that table changes. It reaches the table through the macro-generated in-crate table handle
//! because the public `spacetimedb::Local` context is `#[non_exhaustive]` and unconstructible here.
//! It probes a capability and is never a measured candidate.
//!
//! Also defines `ControlRegistry`, `control_registry_all_view`, and the two atomic activity
//! reducers: site 4's discovery Arm A, a bounded current-state relation maintained by the activity
//! writer, against that procedural view as the O(N) Arm B comparator.

use std::collections::btree_map::{BTreeMap, Entry};
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

/// The `ControlRegistry` candidate's bounded current-state relation: under that candidate's writer
/// contract, one row per Control that has had activity, carrying the identity that owns it and the
/// timestamp of its latest activity.
///
/// The scoping is not pedantry. The retained history-only [`insert_control_activity`] and
/// [`insert_indexed_control_activity`] belong to the sender-view candidates and write audit rows
/// with no registry maintenance at all, so nothing structural excludes a control from having
/// activity and no registry row. The registry reproducer never calls those paths and re-checks
/// composition after every phase; that discipline, not the schema, is what holds the correspondence.
///
/// This is Arm A of site 4's discovery pair — an O(K) relation maintained by the activity writer —
/// against Arm B's O(N) [`control_activity_latest_by_control_view`] scan of the audit history. Both
/// answer the same question; they differ in read set, which is the object of study.
///
/// **Keyed on `control_uuid` alone**, because production establishes `control_uuid ->
/// user_identity` as a function: `entity_owner.entity_uuid` is a primary key, all three activity
/// insert paths read that stored identity, and no writer mutates it after claim
/// (`callosum/callosum/src/tables/insert_chronicle_message/control_activity_update.rs`). The
/// experiment preserves that dependency rather than assuming it — repeat activity derives the
/// identity from this row instead of accepting one from the caller.
///
/// **`user_identity` deliberately carries no index.** Discovery is unfiltered — the deployed
/// consumers subscribe to every control regardless of identity — so an index here would serve no
/// read while adding write maintenance to the very write this candidate exists to justify.
#[table(accessor = control_registry, public)]
pub struct ControlRegistry {
    #[primary_key]
    pub control_uuid: u64,
    pub user_identity: Identity,
    pub last_ts: Timestamp,
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

/// Capability reproducer for site 4's second question — admin empty-result capability: can a view
/// return a genuinely empty result *without* naming a sentinel value that some row could one day
/// hold?
///
/// Production's `control_activity_view_all`
/// (`callosum/callosum/src/tables/control_activity_view_all.rs`) answers its non-admin branch with
/// `row.id.eq(u64::MAX)`, an always-false filter written, in its own comment, "for want of an empty
/// query primitive". Its emptiness rests on no row ever holding that `id`, and no schema invariant
/// says so. This view is that branch with the sentinel removed: `row.id.ne(row.id)` compares the
/// column to *itself*, so no row can satisfy it whatever values the table holds — the predicate is
/// unsatisfiable by construction rather than by assumption about the data.
///
/// Pinned 2.7.0 exposes no `Query::empty()`, but the typed builder does accept a column on the
/// right-hand side — `Col<T, V>` is `Copy` and implements `RHS<T, V>`
/// (`spacetimedb-query-builder-2.7.0/src/table.rs:57`, `src/expr.rs:70`) — and lowers this to
/// `WHERE ("control_activity"."id" <> "control_activity"."id")`. That is *static* expressibility,
/// which the type checker alone settles. Whether the server accepts the query and applies an empty
/// subscription is what running this view is for, and is the only thing it can establish.
///
/// The typed contradiction is the whole of this view. The governing spec admits raw-query and
/// procedural-empty alternatives only if this form fails, so neither is pre-wired here.
///
/// **No admin branch, deliberately.** This module has no analogue of production's `admin` table,
/// and the authorization half of site 4's question is the separate, unrun G4 gate — non-admin
/// callers receive no unauthorized output from admin views — whose deployed premise is itself in
/// doubt, since the admin subscription currently reads the raw public table and never reaches
/// `control_activity_view_all`. Capability and authorization are answered apart; this view answers
/// only capability.
#[view(accessor = control_activity_empty_view, public)]
pub fn control_activity_empty_view(ctx: &ViewContext) -> impl Query<ControlActivity> {
    ctx.from
        .control_activity()
        .r#where(|row| row.id.ne(row.id))
}

/// `ControlRegistry` Arm A — the whole registry, unfiltered.
///
/// A bare pass-through, because discovery *is* unfiltered: the deployed admin consumers subscribe to
/// every control regardless of identity, so there is no sender predicate to transcribe here. The
/// arm's interest is entirely in its read set — O(K) registry rows rather than Arm B's O(N) history
/// scan — not in its predicate.
#[view(accessor = control_registry_all_view, public)]
pub fn control_registry_all_view(ctx: &ViewContext) -> impl Query<ControlRegistry> {
    ctx.from.control_registry()
}

/// Bounded capability probe for site 4's discovery comparator: can a procedural view compute
/// latest-per-`control_uuid` over production-shaped `control_activity`, which carries no index the
/// view could range over?
///
/// The governing spec first classified this pattern `Blocked(CapabilityDesign)`, reasoning that a
/// `#[view]` body receives `LocalReadOnly`, whose generated per-table handle exposes only `count()`
/// and read-only index accessors, so the only whole-table read expressible is an unbounded range
/// over a non-unique btree this table does not carry. Those facts are true and the conclusion does
/// not follow: they bound the accessor surface a view is *handed*, not what its body may execute.
///
/// **The preferred public route does not exist, and P1 proved it.** The probe was first written as
/// `Local {}.control_activity().iter()`, on the claim that `spacetimedb::Local` is a public
/// field-less struct without `#[non_exhaustive]`. It carries that attribute
/// (`spacetimedb-2.7.0/src/lib.rs:1543-1545`), as does `LocalReadOnly`, so this foreign module crate
/// cannot construct it and the pinned compiler rejected the body with `E0639: cannot create
/// non-exhaustive struct using struct expression`.
///
/// What compiles is the route below. The `#[table]` macro emits `control_activity__TableHandle`
/// **into this crate** (`spacetimedb-bindings-macro-2.7.0/src/table.rs:1263-1267`) and implements
/// [`Table`] for it (`:1117-1128`); `#[non_exhaustive]` constrains other crates, never the defining
/// one, so the handle is constructible right here, and [`Table::iter`] is a defaulted public method
/// on it (`spacetimedb-2.7.0/src/table.rs:40-44`). So the pinned release does not lack the ability
/// for a view body to scan an unindexed table — it withholds the public context needed to reach it.
/// That is a construction rule, not an engine limit.
///
/// This is a **generated typed** surface, version-coupled to the exact pinned release and not
/// documented as stable. That does not by itself make the pattern undeployable, but it is a cost
/// recorded for Control's deployability judgment, not one this module may discount. Raw
/// `spacetimedb::sys` stays out of candidate code entirely; it is a lower-level diagnostic only.
///
/// **Static expressibility is all the type checker settles**, exactly as
/// [`control_activity_empty_view`] found for its typed contradiction. Whether the server publishes
/// this view, materializes a subscription over it, and — the question that actually matters —
/// *invalidates* it when the scanned table changes are runtime questions no inspection can answer.
/// An imperative scan is invisible to the machinery that learns a view's dependencies from the
/// query it compiles, so a view that materializes correctly and is then never recomputed is a real
/// possible outcome, and the reproducer's live-insert step exists to separate it from success.
///
/// Returns [`ControlActivity`] rather than a bespoke current-state row type: the latest row per
/// control *is* one of the table's own rows, so the probe needs no new table, no new reducer, and
/// no new index — only this view. `control_uuid` is a non-primary-key column of the returned row
/// type, which is precisely the case an explicit view primary key exists for; pinned schema
/// validation retains a declared procedural-view primary key rather than inferring one
/// (`spacetimedb-schema-2.7.0/src/def/validate/v10.rs:1071-1077`).
///
/// The comparator is `(ts, id)` rather than `ts` alone. The seed allocates globally unique
/// timestamps, so the tie-break can never fire; making the order *total* anyway means the result
/// does not depend on the order the scan happens to yield rows in, which is not a property this
/// module should have to assume.
///
/// **This view is a capability probe, not a candidate implementation.** It is never timed, and no
/// performance run may use it: whether the composition-matched comparator is reinstated at all is a
/// decision this probe returns to Control rather than settles.
#[view(accessor = control_activity_latest_by_control_view, public, primary_key = control_uuid)]
pub fn control_activity_latest_by_control_view(_ctx: &ViewContext) -> Vec<ControlActivity> {
    let mut latest: BTreeMap<u64, ControlActivity> = BTreeMap::new();
    // Parenthesized because a struct literal is not allowed in `for … in` expression position: the
    // bare braces would parse as the loop body.
    for row in (control_activity__TableHandle {}).iter() {
        match latest.entry(row.control_uuid) {
            Entry::Vacant(slot) => {
                slot.insert(row);
            }
            Entry::Occupied(mut slot) => {
                let incumbent = slot.get();
                if (row.ts, row.id) > (incumbent.ts, incumbent.id) {
                    slot.insert(row);
                }
            }
        }
    }
    latest.into_values().collect()
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
/// and a *production-representative fixed-cardinality* mutation is not currently defined for this
/// candidate.
///
/// The governing spec rules on this candidate directly: E2 paced visible-apply evidence is
/// required for `IndexedControlActivitySenderView` to reach `Evaluated`, because the deployed
/// client consumes both the initial backfill and ongoing `on_insert` delivery; E3 cold-subscription
/// evidence is screen-only and cannot substitute; no Site 4 performance run may begin until an
/// append-only E2 estimand is explicitly frozen in the spec; and production representativeness may
/// not be weakened with a synthetic delete or an in-place update to manufacture one. Seeding and
/// view composition do not depend on that ruling, which is why they land first.
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

/// A Control's **first** recorded activity: writes the audit row and creates the
/// [`ControlRegistry`] row that summarizes it, in one transaction.
///
/// This is where a control's `user_identity` enters **the registry candidate's writer contract**,
/// and the only place in that contract it ever does — [`record_control_activity`] derives it from
/// the registry rather than accepting one. It is not the only reducer in the module that takes an
/// identity: the sender-view candidates' [`insert_control_activity`] and
/// [`insert_indexed_control_activity`] each accept one, and neither maintains a registry row. That
/// mirrors production's sole writer
/// (`callosum/callosum/src/tables/insert_chronicle_message/control_activity_update.rs`), which reads
/// the identity from the owning record rather than taking it from its caller.
///
/// **The two writes are one transaction, and that is the contract.** A registry row must never
/// exist without the audit row it summarizes: `last_ts` claims to be the maximum timestamp over
/// that control's history, and a registry row committed before any history would summarize nothing.
/// A withdrawn earlier design did exactly that — a `seed_control_registry` taking `last_ts`
/// directly — and was replaced for this reason, not for style.
///
/// **Both preconditions fail loud with the offending key named**, following
/// [`insert_message_visibility`]'s boundary discipline: reaching the pinned `insert`'s generic
/// unique-constraint failure would say a key collided without saying which.
///
/// **Returns `Result` rather than panicking, and that is a deliberate local divergence** from
/// [`update_entity_owner`] and [`insert_message_visibility`], which panic. Both abort the
/// transaction, so rollback is not what separates them: a panic traps the WASM instance and the
/// caller receives only "The instance encountered a fatal error", losing the message entirely,
/// while a returned `Err` is written to the error sink and delivered as the reducer's own text
/// (`spacetimedb-2.7.0/src/rt.rs:219-224,1090-1097`). This candidate's contract is that a refusal
/// *names the offending key or timestamp condition* at the caller, which a panic cannot honor. The
/// divergence is confined to these two reducers and is not a licence to convert the existing ones.
#[reducer]
pub fn record_first_control_activity(
    ctx: &ReducerContext,
    id: u64,
    control_uuid: u64,
    user_identity: Identity,
    ts: Timestamp,
) -> Result<(), String> {
    if ctx
        .db
        .control_registry()
        .control_uuid()
        .find(control_uuid)
        .is_some()
    {
        return Err(format!(
            "record_first_control_activity: control_registry already holds \
             control_uuid={control_uuid}"
        ));
    }
    if ctx.db.control_activity().id().find(id).is_some() {
        return Err(format!(
            "record_first_control_activity: control_activity already holds id={id}"
        ));
    }

    ctx.db.control_activity().insert(ControlActivity {
        id,
        ts,
        control_uuid,
        user_identity,
    });
    ctx.db.control_registry().insert(ControlRegistry {
        control_uuid,
        user_identity,
        last_ts: ts,
    });
    Ok(())
}

/// Every **subsequent** activity for a control: appends the audit row and advances the registry's
/// `last_ts`, preserving its identity, in one transaction.
///
/// **There is no `user_identity` parameter, and that is the contract.** The identity is read back
/// from the registry row, so a caller cannot re-attribute a control's activity by passing a
/// different one — the `control_uuid -> user_identity` dependency production establishes is made
/// unreachable to violate here rather than merely left unviolated. This is the same reasoning that
/// keeps `owner` out of [`update_entity_owner`].
///
/// **`ts` must be strictly greater than the stored `last_ts`.** With that, the invariant
/// `registry.last_ts == max(ts)` over the control's history follows by induction: the base case is
/// [`record_first_control_activity`] committing both rows together, and each step advances the
/// maximum to exactly the value written. Strictness also makes an out-of-order or replayed event
/// *unrepresentable* rather than merely unwritten — a `>=` test would silently accept a duplicate
/// timestamp and leave two rows tied for latest.
///
/// **Scope of that invariant.** It holds at every committed state reached *through these two
/// reducers*. It is not module-global: [`insert_control_activity`] still writes history alone for
/// the sender-view fixtures, deliberately untouched because its cost must stay representative of
/// that separate candidate's pending write estimand. Nothing in the schema prevents mixing the two
/// paths; the registry reproducer simply never calls the history-only one, and validates
/// composition after every phase. A production migration would extend production's sole writer
/// atomically rather than add a second writer.
///
/// All three preconditions fail loud and abort the transaction, so a refused write leaves both
/// tables exactly as they were. As with [`record_first_control_activity`], the refusal is a returned
/// `Err` rather than a panic so the caller receives the offending key or timestamp condition instead
/// of an opaque instance trap.
#[reducer]
pub fn record_control_activity(
    ctx: &ReducerContext,
    id: u64,
    control_uuid: u64,
    ts: Timestamp,
) -> Result<(), String> {
    let Some(existing) = ctx.db.control_registry().control_uuid().find(control_uuid) else {
        return Err(format!(
            "record_control_activity: no control_registry row with control_uuid={control_uuid}"
        ));
    };
    if ctx.db.control_activity().id().find(id).is_some() {
        return Err(format!(
            "record_control_activity: control_activity already holds id={id}"
        ));
    }
    if ts <= existing.last_ts {
        return Err(format!(
            "record_control_activity: ts={ts:?} is not strictly later than the recorded \
             last_ts={:?} for control_uuid={control_uuid}",
            existing.last_ts,
        ));
    }

    ctx.db.control_activity().insert(ControlActivity {
        id,
        ts,
        control_uuid,
        user_identity: existing.user_identity,
    });
    ctx.db
        .control_registry()
        .control_uuid()
        .update(ControlRegistry {
            control_uuid,
            user_identity: existing.user_identity,
            last_ts: ts,
        });
    Ok(())
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
