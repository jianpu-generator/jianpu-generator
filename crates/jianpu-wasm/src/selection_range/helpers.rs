use crate::types::{LyricSpanOut, NoteSpanOut};

/// A `Note` endpoint of a range-resolution pair, grouped into a struct
/// (rather than passed as loose arguments) per this repo's "never use
/// tuples in new data structures" rule — bundles the
/// `(source_part_index, note_id)` pair the cross-scope helper functions
/// below need together, and keeps each of those functions under
/// clippy's `too-many-arguments` limit.
#[derive(Clone, Copy)]
pub(crate) struct NoteEndpoint {
    pub part: usize,
    pub note_id: usize,
}

/// The `Lyric`-endpoint analog of [`NoteEndpoint`], additionally carrying
/// `verse`.
#[derive(Clone, Copy)]
pub(crate) struct LyricEndpoint {
    pub part: usize,
    pub note_id: usize,
    pub verse: usize,
}

/// A part-scoped measure span with no verse — the shared shape of a
/// `PartLabel` endpoint, and of a `Note` endpoint once collapsed to its own
/// degenerate single-measure span (see [`note_measure_index`]).
#[derive(Clone, Copy)]
pub(crate) struct MeasureSpan {
    pub part: usize,
    pub start: usize,
    pub end: usize,
}

/// The verse-carrying analog of [`MeasureSpan`] — the shared shape of a
/// `LyricLabel` endpoint, and of a `Lyric` endpoint once collapsed to its
/// own degenerate single-measure span (see [`lyric_measure_index`]).
#[derive(Clone, Copy)]
pub(crate) struct VerseMeasureSpan {
    pub part: usize,
    pub verse: usize,
    pub start: usize,
    pub end: usize,
}

/// Looks up a `Note` endpoint's own `measure_index` from `note_spans` by
/// `(source_part_index, note_id)` — shared by every arm that treats a
/// `Note` endpoint as a degenerate single-measure span for its own part
/// (the cross-part `Note ↔ Note` arm's own lookup, and every label-mixed
/// arm that reuses the same trick).
pub(crate) fn note_measure_index(
    note_spans: &[NoteSpanOut],
    part: usize,
    note_id: usize,
) -> Option<usize> {
    note_spans
        .iter()
        .find(|span| span.source_part_index == part && span.note_id == note_id)
        .map(|span| span.measure_index)
}

/// The `Lyric`-endpoint analog of `note_measure_index`, additionally keyed
/// by `verse`.
pub(crate) fn lyric_measure_index(
    lyric_spans: &[LyricSpanOut],
    part: usize,
    note_id: usize,
    verse: usize,
) -> Option<usize> {
    lyric_spans
        .iter()
        .find(|span| {
            span.source_part_index == part && span.note_id == note_id && span.verse == verse
        })
        .map(|span| span.measure_index)
}
