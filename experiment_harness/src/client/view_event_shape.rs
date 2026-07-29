//! Which row callback the SDK delivered for one view-cache change.

/// One row-level change the SDK reported on a subscribed view, identified by *which* callback fired.
///
/// The site 4 latest-per-control probe records this rather than assuming it. The view's declared
/// primary key is `control_uuid`, and the change under test replaces a control's latest row with a
/// strictly later one — the same key carrying a different `id` and `ts`. Whether the server reports
/// that as an update in place or as a delete plus an insert is an observation about the pinned
/// release's view maintenance, not something the probe may decide in advance: the spec requires the
/// observed shape be recorded rather than presumed.
///
/// A three-variant enum rather than three counters because the probe reports the *sequence* it saw.
/// Counters would already separate one delete plus one insert from one update — the tallies differ.
/// What they lose is delivery order, which is what makes the recorded shape a description of how the
/// server reported the change rather than merely of how much it reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ViewEventShape {
    /// The SDK ran the view table's `on_insert` callback.
    Insert,
    /// The SDK ran the view table's `on_update` callback — an in-place replacement under one key.
    Update,
    /// The SDK ran the view table's `on_delete` callback.
    Delete,
}

impl ViewEventShape {
    /// The name used when reporting an observed sequence, so the probe's output and this type's
    /// variants cannot drift apart.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Insert => "insert",
            Self::Update => "update",
            Self::Delete => "delete",
        }
    }
}
