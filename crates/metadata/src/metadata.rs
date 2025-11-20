use std::borrow::Cow;
use std::collections::HashMap;

use crate::MetaValue;

/// Metadata associated with a match or context line.
///
/// Contains arbitrary key-value pairs that describe properties
/// of the location where a match occurred (e.g., page number,
/// chapter name, subtitle timestamp, table name, etc.).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MatchMetadata {
    fields: HashMap<Cow<'static, str>, MetaValue>,
}

impl MatchMetadata {
    /// Creates an empty metadata collection.
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Inserts a key-value pair into the metadata.
    ///
    /// If the key already exists, the value is replaced.
    pub fn insert(
        &mut self,
        key: impl Into<Cow<'static, str>>,
        value: MetaValue,
    ) {
        self.fields.insert(key.into(), value);
    }

    /// Retrieves a value by key.
    ///
    /// Returns `None` if the key doesn't exist.
    pub fn get(&self, key: &str) -> Option<&MetaValue> {
        self.fields.get(key)
    }

    /// Returns an iterator over all key-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&Cow<'static, str>, &MetaValue)> {
        self.fields.iter()
    }

    /// Returns the number of key-value pairs.
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Returns true if the metadata contains no key-value pairs.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}
