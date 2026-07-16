//! The matched direct-base-table control.
//!
//! Every arm run is paired with a direct-base-table control run in the same
//! randomized block. This is a base table selection, **not** an eighth view arm:
//! the control run subscribes directly to the base table an arm's view reads, with
//! returned columns and payload width held equivalent to the matched arm.

/// The base table a control run subscribes to directly.
///
/// Per the spec matrix: A/B/D and F use the `message` control; C/E/F′ use the
/// `chronicle_message` control. The mapping from arm to control table is derived
/// privately in [`crate::plan::cell`], so it cannot drift per call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlTable {
    /// The `message` base table.
    Message,
    /// The `chronicle_message` base table.
    ChronicleMessage,
}
