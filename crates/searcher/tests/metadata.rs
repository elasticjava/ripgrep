/*!
Integration tests for metadata-aware searching.

These tests verify that the metadata system works end-to-end:
- Metadata regions are defined with byte ranges
- Matches inherit metadata based on their absolute byte offset
- The metadata flows through the sink correctly
*/

use std::io;

use grep_regex::RegexMatcher;
use grep_metadata::{MatchMetadata, MetaRegion, MetaValue, VecMetaProvider};
use grep_searcher::{
    Searcher, Sink, SinkError, SinkMatch, SinkMatchWithMeta, SinkWithMeta,
};

/// A test sink that captures matches along with their metadata.
struct MetadataCaptureSink {
    matches: Vec<(u64, Option<i64>)>, // (offset, page_number)
}

impl Sink for MetadataCaptureSink {
    type Error = io::Error;

    fn matched(
        &mut self,
        _searcher: &Searcher,
        _mat: &SinkMatch<'_>,
    ) -> Result<bool, Self::Error> {
        // This shouldn't be called when using matched_with_meta
        panic!("matched() called instead of matched_with_meta()");
    }
}

impl SinkWithMeta for MetadataCaptureSink {
    fn matched_with_meta(
        &mut self,
        _searcher: &Searcher,
        mat: &SinkMatchWithMeta<'_, '_>,
    ) -> Result<bool, Self::Error> {
        let offset = mat.base.absolute_byte_offset();
        let page = mat
            .metadata
            .and_then(|m| m.get("page"))
            .and_then(|v| match v {
                MetaValue::Int(i) => Some(*i),
                _ => None,
            });

        self.matches.push((offset, page));
        Ok(true)
    }
}

#[test]
fn test_search_with_metadata_basic() {
    // Haystack with two lines
    let haystack = b"Temperature: 25C\nHumidity: 60%\n";
    //                0-16            17-32

    // Create metadata regions
    let mut page1_meta = MatchMetadata::new();
    page1_meta.insert("page", MetaValue::Int(17));

    let mut page2_meta = MatchMetadata::new();
    page2_meta.insert("page", MetaValue::Int(18));

    let regions = vec![
        MetaRegion {
            start: 0,
            end: 17,
            meta: page1_meta,
        },
        MetaRegion {
            start: 17,
            end: 33,
            meta: page2_meta,
        },
    ];

    let provider = VecMetaProvider::new(regions);

    // Search for "Temp" and "Humi"
    let mut sink = MetadataCaptureSink {
        matches: Vec::new(),
    };
    let matcher = RegexMatcher::new("Temp|Humi").unwrap();
    let mut searcher = Searcher::new();

    searcher
        .search_slice_with_metadata(matcher, haystack, Some(&provider), &mut sink)
        .unwrap();

    // Verify results
    assert_eq!(sink.matches.len(), 2);

    // First match: "Temperature" at offset 0, page 17
    assert_eq!(sink.matches[0].0, 0);
    assert_eq!(sink.matches[0].1, Some(17));

    // Second match: "Humidity" at offset 17, page 18
    assert_eq!(sink.matches[1].0, 17);
    assert_eq!(sink.matches[1].1, Some(18));
}

#[test]
fn test_search_without_metadata_provider() {
    // When provider is None, metadata should be None
    let haystack = b"Temperature: 25C\n";

    struct NoMetaSink {
        received_metadata: bool,
    }

    impl Sink for NoMetaSink {
        type Error = io::Error;
        fn matched(&mut self, _: &Searcher, _: &SinkMatch<'_>) -> Result<bool, Self::Error> {
            panic!("matched() should not be called");
        }
    }

    impl SinkWithMeta for NoMetaSink {
        fn matched_with_meta(
            &mut self,
            _: &Searcher,
            mat: &SinkMatchWithMeta<'_, '_>,
        ) -> Result<bool, Self::Error> {
            self.received_metadata = mat.metadata.is_some();
            Ok(true)
        }
    }

    let mut sink = NoMetaSink {
        received_metadata: true,
    };
    let matcher = RegexMatcher::new("Temp").unwrap();
    let mut searcher = Searcher::new();

    searcher
        .search_slice_with_metadata(matcher, haystack, None, &mut sink)
        .unwrap();

    assert!(!sink.received_metadata, "Expected no metadata when provider is None");
}

#[test]
fn test_multiple_metadata_fields() {
    let haystack = b"Chapter 1: Introduction\nThis is the first chapter.\n";

    // Create metadata with multiple fields
    let mut meta = MatchMetadata::new();
    meta.insert("page", MetaValue::Int(5));
    meta.insert("chapter", MetaValue::Str("Introduction".into()));
    meta.insert("section", MetaValue::Int(1));

    let regions = vec![MetaRegion {
        start: 0,
        end: 100,
        meta,
    }];

    let provider = VecMetaProvider::new(regions);

    struct MultiFieldSink {
        page: Option<i64>,
        chapter: Option<String>,
        section: Option<i64>,
    }

    impl Sink for MultiFieldSink {
        type Error = io::Error;
        fn matched(&mut self, _: &Searcher, _: &SinkMatch<'_>) -> Result<bool, Self::Error> {
            panic!("matched() should not be called");
        }
    }

    impl SinkWithMeta for MultiFieldSink {
        fn matched_with_meta(
            &mut self,
            _: &Searcher,
            mat: &SinkMatchWithMeta<'_, '_>,
        ) -> Result<bool, Self::Error> {
            if let Some(meta) = mat.metadata {
                self.page = meta.get("page").and_then(|v| match v {
                    MetaValue::Int(i) => Some(*i),
                    _ => None,
                });
                self.chapter = meta.get("chapter").and_then(|v| match v {
                    MetaValue::Str(s) => Some(s.to_string()),
                    _ => None,
                });
                self.section = meta.get("section").and_then(|v| match v {
                    MetaValue::Int(i) => Some(*i),
                    _ => None,
                });
            }
            Ok(false) // Stop after first match
        }
    }

    let mut sink = MultiFieldSink {
        page: None,
        chapter: None,
        section: None,
    };
    let matcher = RegexMatcher::new("Chapter").unwrap();
    let mut searcher = Searcher::new();

    searcher
        .search_slice_with_metadata(matcher, haystack, Some(&provider), &mut sink)
        .unwrap();

    assert_eq!(sink.page, Some(5));
    assert_eq!(sink.chapter, Some("Introduction".to_string()));
    assert_eq!(sink.section, Some(1));
}

#[test]
fn test_metadata_at_region_boundaries() {
    let haystack = b"AAA\nBBB\nCCC\n";
    //                0-3  4-7  8-11

    let mut meta1 = MatchMetadata::new();
    meta1.insert("region", MetaValue::Str("A".into()));

    let mut meta2 = MatchMetadata::new();
    meta2.insert("region", MetaValue::Str("B".into()));

    let mut meta3 = MatchMetadata::new();
    meta3.insert("region", MetaValue::Str("C".into()));

    let regions = vec![
        MetaRegion {
            start: 0,
            end: 4,
            meta: meta1,
        },
        MetaRegion {
            start: 4,
            end: 8,
            meta: meta2,
        },
        MetaRegion {
            start: 8,
            end: 12,
            meta: meta3,
        },
    ];

    let provider = VecMetaProvider::new(regions);

    let mut sink = MetadataCaptureSink {
        matches: Vec::new(),
    };
    let matcher = RegexMatcher::new("A|B|C").unwrap();
    let mut searcher = Searcher::new();

    searcher
        .search_slice_with_metadata(matcher, haystack, Some(&provider), &mut sink)
        .unwrap();

    // Should find matches at different offsets (one per line)
    assert_eq!(sink.matches.len(), 3, "Expected exactly 3 matches (one per line)");
}
