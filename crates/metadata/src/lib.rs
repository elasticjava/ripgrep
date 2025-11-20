/*!
This crate provides structured metadata support for search matches.

Adapters (PDF, ZIP, MKV subtitles, DOCX, ODT, SQLite, etc.) can associate
arbitrary key-value metadata with byte ranges in searched content. Search
matches automatically inherit metadata based on their absolute byte offset.

# Example

```rust
use grep_metadata::{MetaValue, MatchMetadata, MetaRegion, MetadataProvider, VecMetaProvider};

// Create metadata for a PDF document
let mut page1_meta = MatchMetadata::new();
page1_meta.insert("page", MetaValue::Int(1));
page1_meta.insert("chapter", MetaValue::Str("Introduction".into()));

let mut page2_meta = MatchMetadata::new();
page2_meta.insert("page", MetaValue::Int(2));

// Define regions (byte ranges with metadata)
let regions = vec![
    MetaRegion { start: 0, end: 1000, meta: page1_meta },
    MetaRegion { start: 1000, end: 2000, meta: page2_meta },
];

// Create provider
let provider = VecMetaProvider::new(regions);

// Look up metadata for a match at byte offset 500
let meta = provider.metadata_for_offset(500);
assert!(meta.is_some());
```
*/

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

mod value;
mod metadata;
mod region;
mod provider;

pub use value::MetaValue;
pub use metadata::MatchMetadata;
pub use region::MetaRegion;
pub use provider::{MetadataProvider, VecMetaProvider};

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Step 1.2 Tests: MetaValue and MatchMetadata
    // ========================================================================

    #[test]
    fn test_metadata_creation_and_retrieval() {
        let mut meta = MatchMetadata::new();

        // Insert page=17, chapter="Intro"
        meta.insert("page", MetaValue::Int(17));
        meta.insert("chapter", MetaValue::Str("Intro".into()));

        // Assert retrieval works
        assert_eq!(meta.get("page"), Some(&MetaValue::Int(17)));
        assert_eq!(meta.get("chapter"), Some(&MetaValue::Str("Intro".into())));
        assert_eq!(meta.get("missing"), None);
    }

    #[test]
    fn test_metavalue_display() {
        // Test Int
        assert_eq!(MetaValue::Int(17).to_string(), "17");
        assert_eq!(MetaValue::Int(-42).to_string(), "-42");

        // Test Str
        assert_eq!(MetaValue::Str("test".into()).to_string(), "test");
        assert_eq!(MetaValue::Str("Introduction".into()).to_string(), "Introduction");

        // Test Float
        assert_eq!(MetaValue::Float(3.14).to_string(), "3.14");
        assert_eq!(MetaValue::Float(-2.5).to_string(), "-2.5");

        // Test Bool
        assert_eq!(MetaValue::Bool(true).to_string(), "true");
        assert_eq!(MetaValue::Bool(false).to_string(), "false");
    }

    #[test]
    fn test_metavalue_from_conversions() {
        // Test From<&'static str>
        let v: MetaValue = "hello".into();
        assert_eq!(v, MetaValue::Str("hello".into()));

        // Test From<String>
        let v: MetaValue = String::from("world").into();
        assert_eq!(v, MetaValue::Str("world".into()));

        // Test From<i64>
        let v: MetaValue = 42i64.into();
        assert_eq!(v, MetaValue::Int(42));

        // Test From<f64>
        let v: MetaValue = 3.14f64.into();
        assert_eq!(v, MetaValue::Float(3.14));

        // Test From<bool>
        let v: MetaValue = true.into();
        assert_eq!(v, MetaValue::Bool(true));
    }

    #[test]
    fn test_metadata_insert_and_replace() {
        let mut meta = MatchMetadata::new();

        // Insert initial value
        meta.insert("key", MetaValue::Int(1));
        assert_eq!(meta.get("key"), Some(&MetaValue::Int(1)));

        // Replace with new value
        meta.insert("key", MetaValue::Int(2));
        assert_eq!(meta.get("key"), Some(&MetaValue::Int(2)));
    }

    #[test]
    fn test_metadata_len_and_is_empty() {
        let mut meta = MatchMetadata::new();
        assert_eq!(meta.len(), 0);
        assert!(meta.is_empty());

        meta.insert("key1", MetaValue::Int(1));
        assert_eq!(meta.len(), 1);
        assert!(!meta.is_empty());

        meta.insert("key2", MetaValue::Int(2));
        assert_eq!(meta.len(), 2);
        assert!(!meta.is_empty());
    }

    #[test]
    fn test_metadata_iter() {
        let mut meta = MatchMetadata::new();
        meta.insert("page", MetaValue::Int(17));
        meta.insert("chapter", MetaValue::Str("Intro".into()));

        let items: Vec<_> = meta.iter().collect();
        assert_eq!(items.len(), 2);

        // Check that both items are present (order is not guaranteed)
        let has_page = items.iter().any(|(k, v)| k.as_ref() == "page" && **v == MetaValue::Int(17));
        let has_chapter = items.iter().any(|(k, v)| k.as_ref() == "chapter" && **v == MetaValue::Str("Intro".into()));

        assert!(has_page);
        assert!(has_chapter);
    }

    // ========================================================================
    // Step 2.1 Tests: MetaRegion and VecMetaProvider
    // ========================================================================

    #[test]
    fn test_vec_provider_with_regions() {
        // Create 3 regions:
        //   0..100  → page=1
        //   100..200 → page=2
        //   200..300 → page=3, chapter="Methods"

        let mut meta1 = MatchMetadata::new();
        meta1.insert("page", MetaValue::Int(1));

        let mut meta2 = MatchMetadata::new();
        meta2.insert("page", MetaValue::Int(2));

        let mut meta3 = MatchMetadata::new();
        meta3.insert("page", MetaValue::Int(3));
        meta3.insert("chapter", MetaValue::Str("Methods".into()));

        let regions = vec![
            MetaRegion { start: 0, end: 100, meta: meta1 },
            MetaRegion { start: 100, end: 200, meta: meta2 },
            MetaRegion { start: 200, end: 300, meta: meta3 },
        ];

        let provider = VecMetaProvider::new(regions);

        // Assert offset=50 returns page=1
        let meta = provider.metadata_for_offset(50);
        assert!(meta.is_some());
        assert_eq!(meta.unwrap().get("page"), Some(&MetaValue::Int(1)));

        // Assert offset=150 returns page=2
        let meta = provider.metadata_for_offset(150);
        assert!(meta.is_some());
        assert_eq!(meta.unwrap().get("page"), Some(&MetaValue::Int(2)));

        // Assert offset=250 returns page=3, chapter="Methods"
        let meta = provider.metadata_for_offset(250);
        assert!(meta.is_some());
        assert_eq!(meta.unwrap().get("page"), Some(&MetaValue::Int(3)));
        assert_eq!(meta.unwrap().get("chapter"), Some(&MetaValue::Str("Methods".into())));

        // Assert offset=400 returns None
        let meta = provider.metadata_for_offset(400);
        assert!(meta.is_none());

        // Assert offset=100 returns page=2 (start is inclusive)
        let meta = provider.metadata_for_offset(100);
        assert!(meta.is_some());
        assert_eq!(meta.unwrap().get("page"), Some(&MetaValue::Int(2)));
    }

    #[test]
    fn test_empty_provider() {
        // VecMetaProvider with no regions
        let provider = VecMetaProvider::new(vec![]);

        // Assert all lookups return None
        assert!(provider.metadata_for_offset(0).is_none());
        assert!(provider.metadata_for_offset(100).is_none());
        assert!(provider.metadata_for_offset(1000).is_none());

        assert_eq!(provider.region_count(), 0);
    }

    #[test]
    fn test_overlapping_regions() {
        // Region 1: 0..100 → tag="outer"
        // Region 2: 50..75 → tag="inner"

        let mut meta1 = MatchMetadata::new();
        meta1.insert("tag", MetaValue::Str("outer".into()));

        let mut meta2 = MatchMetadata::new();
        meta2.insert("tag", MetaValue::Str("inner".into()));

        let regions = vec![
            MetaRegion { start: 0, end: 100, meta: meta1 },
            MetaRegion { start: 50, end: 75, meta: meta2 },
        ];

        let provider = VecMetaProvider::new(regions);

        // Assert offset=60 returns "inner" (last matching region wins)
        let meta = provider.metadata_for_offset(60);
        assert!(meta.is_some());
        assert_eq!(meta.unwrap().get("tag"), Some(&MetaValue::Str("inner".into())));

        // Assert offset=30 returns "outer" (only first region matches)
        let meta = provider.metadata_for_offset(30);
        assert!(meta.is_some());
        assert_eq!(meta.unwrap().get("tag"), Some(&MetaValue::Str("outer".into())));

        // Assert offset=80 returns "outer" (only first region matches)
        let meta = provider.metadata_for_offset(80);
        assert!(meta.is_some());
        assert_eq!(meta.unwrap().get("tag"), Some(&MetaValue::Str("outer".into())));
    }

    #[test]
    fn test_meta_region_contains() {
        let mut meta = MatchMetadata::new();
        meta.insert("test", MetaValue::Int(1));

        let region = MetaRegion::new(100, 200, meta);

        // Test boundary conditions
        assert!(!region.contains(99));   // Before start
        assert!(region.contains(100));   // At start (inclusive)
        assert!(region.contains(150));   // Middle
        assert!(region.contains(199));   // Before end
        assert!(!region.contains(200));  // At end (exclusive)
        assert!(!region.contains(201));  // After end
    }

    #[test]
    fn test_meta_region_len() {
        let meta = MatchMetadata::new();

        let region = MetaRegion::new(100, 200, meta.clone());
        assert_eq!(region.len(), 100);

        let region = MetaRegion::new(0, 50, meta.clone());
        assert_eq!(region.len(), 50);

        let region = MetaRegion::new(10, 10, meta.clone());
        assert_eq!(region.len(), 0);
        assert!(region.is_empty());
    }

    #[test]
    fn test_empty_regions_are_filtered() {
        let mut meta = MatchMetadata::new();
        meta.insert("test", MetaValue::Int(1));

        let regions = vec![
            MetaRegion { start: 0, end: 100, meta: meta.clone() },
            MetaRegion { start: 100, end: 100, meta: meta.clone() },  // Empty
            MetaRegion { start: 200, end: 300, meta: meta.clone() },
            MetaRegion { start: 250, end: 200, meta: meta.clone() },  // Invalid (start > end)
        ];

        let provider = VecMetaProvider::new(regions);

        // Only 2 valid regions should remain
        assert_eq!(provider.region_count(), 2);
    }

    #[test]
    fn test_regions_are_sorted() {
        let mut meta1 = MatchMetadata::new();
        meta1.insert("id", MetaValue::Int(1));

        let mut meta2 = MatchMetadata::new();
        meta2.insert("id", MetaValue::Int(2));

        let mut meta3 = MatchMetadata::new();
        meta3.insert("id", MetaValue::Int(3));

        // Create regions in random order
        let regions = vec![
            MetaRegion { start: 200, end: 300, meta: meta3 },
            MetaRegion { start: 0, end: 100, meta: meta1 },
            MetaRegion { start: 100, end: 200, meta: meta2 },
        ];

        let provider = VecMetaProvider::new(regions);

        // Verify regions are sorted by start offset
        let sorted_regions = provider.regions();
        assert_eq!(sorted_regions[0].start, 0);
        assert_eq!(sorted_regions[1].start, 100);
        assert_eq!(sorted_regions[2].start, 200);
    }

    #[test]
    fn test_empty_metadata_provider() {
        let provider = provider::EmptyMetadataProvider;

        // All lookups return None
        assert!(provider.metadata_for_offset(0).is_none());
        assert!(provider.metadata_for_offset(100).is_none());
        assert!(provider.metadata_for_offset(u64::MAX).is_none());
    }

    #[test]
    fn test_metadata_clone() {
        let mut meta1 = MatchMetadata::new();
        meta1.insert("page", MetaValue::Int(42));

        let meta2 = meta1.clone();

        assert_eq!(meta2.get("page"), Some(&MetaValue::Int(42)));
    }
}
