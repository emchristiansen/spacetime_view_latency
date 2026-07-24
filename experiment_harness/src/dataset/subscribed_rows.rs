//! A subscriber's observed result set, typed by table family, for correctness checks.

use anyhow::{bail, ensure, Result};

use crate::module_artifact::bindings::{ChronicleMessage, Message};

/// A subscribed result set, tagged by the row family of the subscribed table. Two sets are
/// compared by sorting each by primary key and asserting full row equality, so delivery
/// order never affects the comparison. Used both for the seed-derived expected set and the
/// cache read-back.
#[derive(Debug, Clone)]
pub(crate) enum SubscribedRows {
    Message(Vec<Message>),
    Chronicle(Vec<ChronicleMessage>),
}

impl SubscribedRows {
    /// The number of observed rows.
    pub(crate) fn len(&self) -> usize {
        match self {
            SubscribedRows::Message(rows) => rows.len(),
            SubscribedRows::Chronicle(rows) => rows.len(),
        }
    }

    /// Assert this observed set equals `expected`: same family, same cardinality, and the
    /// same rows once each side is sorted by primary key. Fails fast, naming the mismatch.
    pub(crate) fn assert_equals(&self, expected: &SubscribedRows) -> Result<()> {
        match (self, expected) {
            (SubscribedRows::Message(actual), SubscribedRows::Message(expected)) => {
                assert_row_sets(actual, expected, |m| m.id)
            }
            (SubscribedRows::Chronicle(actual), SubscribedRows::Chronicle(expected)) => {
                assert_row_sets(actual, expected, |m| m.uuid)
            }
            _ => bail!(
                "subscribed-row family mismatch: observed and expected sets are different \
                 table families"
            ),
        }
    }
}

/// Assert two row sets are equal as multisets keyed by primary key. Primary keys are unique
/// per table, so sorting by key yields a canonical order for a full row-equality check.
fn assert_row_sets<R: Clone + PartialEq + std::fmt::Debug>(
    actual: &[R],
    expected: &[R],
    key: impl Fn(&R) -> u64,
) -> Result<()> {
    ensure!(
        actual.len() == expected.len(),
        "cardinality mismatch: observed {} rows, expected {}",
        actual.len(),
        expected.len()
    );
    let mut actual = actual.to_vec();
    let mut expected = expected.to_vec();
    actual.sort_by_key(&key);
    expected.sort_by_key(&key);
    ensure!(
        actual == expected,
        "subscribed row-set mismatch after sorting by primary key:\n  observed: {actual:?}\n  \
         expected: {expected:?}"
    );
    Ok(())
}
