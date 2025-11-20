use crate::MatchMetadata;

/// A byte range with associated metadata.
///
/// Represents a contiguous region in a file or stream where specific
/// metadata applies. For example, in a PDF, pages 1-10 might be bytes
/// 0-50000, and each page would have its own MetaRegion.
#[derive(Debug, Clone, PartialEq)]
pub struct MetaRegion {
    /// Starting byte offset (inclusive)
    pub start: u64,
    /// Ending byte offset (exclusive)
    pub end: u64,
    /// Metadata that applies to this region
    pub meta: MatchMetadata,
}

impl MetaRegion {
    /// Creates a new metadata region.
    pub fn new(start: u64, end: u64, meta: MatchMetadata) -> Self {
        Self { start, end, meta }
    }

    /// Returns true if the given offset falls within this region.
    ///
    /// The start offset is inclusive, the end offset is exclusive.
    pub fn contains(&self, offset: u64) -> bool {
        offset >= self.start && offset < self.end
    }

    /// Returns the length of this region in bytes.
    pub fn len(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }

    /// Returns true if this region has zero length.
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}
