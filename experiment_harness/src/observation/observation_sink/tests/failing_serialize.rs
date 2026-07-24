//! Shared fixture: a value whose `Serialize` always errors.

use serde::ser::Error;
use serde::{Serialize, Serializer};

/// A payload whose serialization always fails, to drive the sink's pre-write (serialization) failure
/// path — [`PersistError::BeforeWrite`](crate::observation::persist_error::PersistError::BeforeWrite)
/// — without any I/O.
pub(super) struct FailingSerialize;

impl Serialize for FailingSerialize {
    fn serialize<S: Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
        Err(S::Error::custom("deliberate serialization failure"))
    }
}
