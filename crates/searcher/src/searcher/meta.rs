/*!
Metadata-aware search strategies.

This module provides search strategy implementations that support metadata
providers. These strategies are similar to their non-metadata counterparts
but additionally look up and attach metadata to matches and context lines.
*/

use grep_matcher::Matcher;
use grep_metadata::MetadataProvider;

use crate::{
    lines::LineIter,
    searcher::Searcher,
    sink::{SinkError, SinkFinish, SinkMatch, SinkMatchWithMeta, SinkWithMeta},
};

/// A metadata-aware line-by-line search strategy for slices.
///
/// This is similar to `SliceByLine` but adds metadata support.
pub(crate) struct SliceByLineWithMeta<'s, 'm, M, S> {
    searcher: &'s Searcher,
    matcher: M,
    slice: &'s [u8],
    provider: Option<&'m dyn MetadataProvider>,
    sink: S,
    absolute_byte_offset: u64,
    line_number: Option<u64>,
}

impl<'s, 'm, M: Matcher, S: SinkWithMeta> SliceByLineWithMeta<'s, 'm, M, S> {
    pub(crate) fn new(
        searcher: &'s Searcher,
        matcher: M,
        slice: &'s [u8],
        provider: Option<&'m dyn MetadataProvider>,
        sink: S,
    ) -> Self {
        let line_number = if searcher.config.line_number {
            Some(1)
        } else {
            None
        };

        SliceByLineWithMeta {
            searcher,
            matcher,
            slice,
            provider,
            sink,
            absolute_byte_offset: 0,
            line_number,
        }
    }

    pub(crate) fn run(mut self) -> Result<(), S::Error>
    where
        S::Error: From<<M as Matcher>::Error>,
    {
        self.sink.begin(self.searcher)?;

        let line_term = self.searcher.line_terminator();
        let mut line_iter = LineIter::new(line_term.as_byte(), self.slice);

        while let Some(line) = line_iter.next() {
            let line_offset = self.absolute_byte_offset;

            // Strip line terminator for matching (LineIter includes it)
            let line_without_term = if line.ends_with(&[line_term.as_byte()]) {
                &line[..line.len() - 1]
            } else {
                line
            };

            // Check if this line matches
            let is_match = self.matcher.is_match(line_without_term)?;

            // Apply invert_match logic: if invert is enabled, flip the match result
            let should_report = if self.searcher.config.invert_match {
                !is_match
            } else {
                is_match
            };

            if should_report {
                // Look up metadata for this match
                let metadata = self.provider
                    .and_then(|p| p.metadata_for_offset(line_offset));

                // Create SinkMatch
                let base = SinkMatch {
                    line_term,
                    bytes: line,
                    absolute_byte_offset: line_offset,
                    line_number: self.line_number,
                    buffer: self.slice,
                    bytes_range_in_buffer: 0..line.len(), // Simplified for prototype
                };

                let mat = SinkMatchWithMeta { base, metadata };

                if !self.sink.matched_with_meta(self.searcher, &mat)? {
                    return Ok(());
                }
            }

            // Update position tracking
            self.absolute_byte_offset += line.len() as u64;
            if let Some(ref mut line_num) = self.line_number {
                *line_num += 1;
            }
        }

        self.sink.finish(
            self.searcher,
            &SinkFinish {
                byte_count: self.absolute_byte_offset,
                binary_byte_offset: None,
            },
        )?;

        Ok(())
    }
}

/// A metadata-aware multi-line search strategy for slices.
///
/// This is similar to `MultiLine` but adds metadata support.
pub(crate) struct MultiLineWithMeta<'s, 'm, M, S> {
    searcher: &'s Searcher,
    matcher: M,
    slice: &'s [u8],
    provider: Option<&'m dyn MetadataProvider>,
    sink: S,
    absolute_byte_offset: u64,
}

impl<'s, 'm, M: Matcher, S: SinkWithMeta> MultiLineWithMeta<'s, 'm, M, S> {
    pub(crate) fn new(
        searcher: &'s Searcher,
        matcher: M,
        slice: &'s [u8],
        provider: Option<&'m dyn MetadataProvider>,
        sink: S,
    ) -> Self {
        MultiLineWithMeta {
            searcher,
            matcher,
            slice,
            provider,
            sink,
            absolute_byte_offset: 0,
        }
    }

    pub(crate) fn run(mut self) -> Result<(), S::Error>
    where
        S::Error: From<<M as Matcher>::Error>,
    {
        self.sink.begin(self.searcher)?;

        // For multi-line search, find all matches in the entire slice
        let mut offset = 0;
        while offset < self.slice.len() {
            match self.matcher.find_at(&self.slice[offset..], offset)? {
                None => break,
                Some(mat) => {
                    let match_offset = offset + mat.start();
                    let match_end = offset + mat.end();

                    // Look up metadata
                    let metadata = self.provider
                        .and_then(|p| p.metadata_for_offset(match_offset as u64));

                    // Create SinkMatch (simplified - doesn't handle line numbers properly)
                    let line_term = self.searcher.line_terminator();
                    let base = SinkMatch {
                        line_term,
                        bytes: &self.slice[match_offset..match_end],
                        absolute_byte_offset: match_offset as u64,
                        line_number: None, // Multi-line search doesn't track line numbers in this simple version
                        buffer: self.slice,
                        bytes_range_in_buffer: match_offset..match_end,
                    };

                    let sink_mat = SinkMatchWithMeta { base, metadata };

                    if !self.sink.matched_with_meta(self.searcher, &sink_mat)? {
                        break;
                    }

                    offset = match_end;
                    if offset == match_offset {
                        // Avoid infinite loop on zero-width matches
                        offset += 1;
                    }
                }
            }
        }

        self.absolute_byte_offset = self.slice.len() as u64;

        self.sink.finish(
            self.searcher,
            &SinkFinish {
                byte_count: self.absolute_byte_offset,
                binary_byte_offset: None,
            },
        )?;

        Ok(())
    }
}
