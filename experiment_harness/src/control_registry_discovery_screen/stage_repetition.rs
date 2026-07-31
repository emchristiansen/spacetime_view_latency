//! Which stage and repetition an attempt belongs to.

use serde::Serialize;

use crate::control_registry_discovery_screen::screen_block_index::ScreenBlockIndex;

/// The stage an attempt belongs to, carrying its repetition coordinate.
///
/// One variant, because this module runs one stage. It is an enum rather than a bare block index so
/// that a later disposition-tier stage — should capacity ever justify one — adds a variant instead
/// of reinterpreting screen blocks as something else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum StageRepetition {
    Screen(ScreenBlockIndex),
}
