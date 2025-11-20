use crate::{MatchMetadata, MetaRegion};

/// Trait for providing metadata based on byte offsets.
///
/// Implementations supply metadata for specific byte offsets in a file or stream.
/// This abstraction allows different adapters (PDF, ZIP, MKV, etc.) to provide
/// metadata in a uniform way.
///
/// Implementations must be thread-safe (`Send + Sync`) as they may be used
/// across multiple threads during parallel search operations.
pub trait MetadataProvider: Send + Sync {
    /// Returns metadata for the given byte offset, if available.
    ///
    /// Returns `None` if no metadata exists for the given offset.
    fn metadata_for_offset(&self, offset: u64) -> Option<&MatchMetadata>;
}

/// A simple vector-based metadata provider.
///
/// Stores metadata regions in a sorted vector and performs lookups
/// using linear search. For small to medium numbers of regions, this
/// is efficient enough. For large numbers of regions, consider using
/// a more sophisticated data structure (e.g., interval tree).
///
/// # Overlapping Regions
///
/// If regions overlap, the last matching region (highest index) wins.
/// This allows layering metadata where more specific metadata can
/// override more general metadata.
#[derive(Debug, Clone)]
pub struct VecMetaProvider {
    regions: Vec<MetaRegion>,
}

impl VecMetaProvider {
    /// Creates a new provider from a vector of regions.
    ///
    /// The regions will be sorted by start offset for efficient lookup.
    /// Empty regions (where start >= end) are silently ignored.
    pub fn new(mut regions: Vec<MetaRegion>) -> Self {
        // Remove empty regions
        regions.retain(|r| !r.is_empty());

        // Sort by start offset
        regions.sort_by_key(|r| r.start);

        Self { regions }
    }

    /// Returns the number of regions in this provider.
    pub fn region_count(&self) -> usize {
        self.regions.len()
    }

    /// Returns a slice of all regions (sorted by start offset).
    pub fn regions(&self) -> &[MetaRegion] {
        &self.regions
    }
}

impl MetadataProvider for VecMetaProvider {
    fn metadata_for_offset(&self, offset: u64) -> Option<&MatchMetadata> {
        // Linear search from the end (last matching region wins)
        // This handles overlapping regions correctly
        self.regions
            .iter()
            .rfind(|r| r.contains(offset))
            .map(|r| &r.meta)
    }
}

/// An empty metadata provider that returns `None` for all offsets.
///
/// Useful as a default or placeholder when no metadata is available.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptyMetadataProvider;

impl MetadataProvider for EmptyMetadataProvider {
    fn metadata_for_offset(&self, _offset: u64) -> Option<&MatchMetadata> {
        None
    }
}
