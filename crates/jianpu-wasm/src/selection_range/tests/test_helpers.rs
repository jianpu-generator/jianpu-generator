use crate::selection_range::types::{ClickableElementId, LyricCellOut, NoteCellOut};
use crate::types::{LyricSpanOut, NoteSpanOut};

pub(super) fn note_span(
    source_part_index: usize,
    note_id: usize,
    measure_index: usize,
) -> NoteSpanOut {
    NoteSpanOut {
        source_part_index,
        note_id,
        measure_index,
        start: Some(note_id * 10),
        end: Some(note_id * 10 + 1),
    }
}

pub(super) fn lyric_span(
    source_part_index: usize,
    note_id: usize,
    verse: usize,
    measure_index: usize,
) -> LyricSpanOut {
    LyricSpanOut {
        source_part_index,
        note_id,
        verse,
        measure_index,
        start: note_id * 10,
        end: note_id * 10 + 1,
    }
}

pub(super) fn measure(start: usize, end: usize) -> ClickableElementId {
    ClickableElementId::Measure {
        measure_index_start: start,
        measure_index_end: end,
    }
}

pub(super) fn note(source_part_index: usize, note_id: usize) -> ClickableElementId {
    ClickableElementId::Note {
        source_part_index,
        note_id,
    }
}

pub(super) fn lyric(source_part_index: usize, note_id: usize, verse: usize) -> ClickableElementId {
    ClickableElementId::Lyric {
        source_part_index,
        note_id,
        verse,
    }
}

pub(super) fn part_label(
    source_part_index: usize,
    measure_index_start: usize,
    measure_index_end: usize,
) -> ClickableElementId {
    ClickableElementId::PartLabel {
        source_part_index,
        measure_index_start,
        measure_index_end,
    }
}

pub(super) fn lyric_label(
    source_part_index: usize,
    verse: usize,
    measure_index_start: usize,
    measure_index_end: usize,
) -> ClickableElementId {
    ClickableElementId::LyricLabel {
        source_part_index,
        verse,
        measure_index_start,
        measure_index_end,
    }
}

pub(super) fn note_cell(source_part_index: usize, note_id: usize) -> NoteCellOut {
    NoteCellOut {
        source_part_index,
        note_id,
    }
}

pub(super) fn lyric_cell(source_part_index: usize, note_id: usize, verse: usize) -> LyricCellOut {
    LyricCellOut {
        source_part_index,
        note_id,
        verse,
    }
}

/// Fixture shared by every case below: three note-carrying measures (0, 1,
/// 2), a second part only present in measure 1, and lyrics on part 0 only.
pub(super) fn fixture() -> (Vec<NoteSpanOut>, Vec<LyricSpanOut>) {
    let note_spans = vec![
        note_span(0, 0, 0),
        note_span(0, 1, 1),
        note_span(0, 2, 2),
        note_span(1, 3, 1),
    ];
    let lyric_spans = vec![
        lyric_span(0, 0, 0, 0),
        lyric_span(0, 1, 0, 1),
        lyric_span(0, 2, 0, 2),
    ];
    (note_spans, lyric_spans)
}

/// Fixture for the cross-verse `Lyric ↔ Lyric` arm: extends `fixture()`
/// (part 0's note_ids 0-2, verse-0 lyrics on all three) with a verse-1 lyric
/// on note_id 1 only and a verse-2 lyric on note_id 0 — enough to prove the
/// arm ranges over both `note_id` and `verse` independently, not unioning
/// every verse it happens to find.
pub(super) fn cross_verse_lyric_fixture() -> (Vec<NoteSpanOut>, Vec<LyricSpanOut>) {
    let (note_spans, mut lyric_spans) = fixture();
    lyric_spans.push(lyric_span(0, 1, 1, 1));
    lyric_spans.push(lyric_span(0, 0, 2, 0));
    (note_spans, lyric_spans)
}

/// Fixture for the cross-part `Lyric ↔ Lyric` arm: extends `fixture()` with
/// a verse-0 lyric on part 1 (measure 1, same note_id/measure as part 1's
/// only note), so a cross-part pair has a second part's lyric to range
/// into.
pub(super) fn cross_part_lyric_fixture() -> (Vec<NoteSpanOut>, Vec<LyricSpanOut>) {
    let (note_spans, mut lyric_spans) = fixture();
    lyric_spans.push(lyric_span(1, 3, 0, 1));
    (note_spans, lyric_spans)
}

/// Fixture for `LyricLabel ↔ LyricLabel` cases: extends `fixture()` with a
/// verse-0 lyric on part 1 (measure 1), so a multi-part sweep at verse 0 has
/// more than one part's lyrics to union together.
pub(super) fn lyric_label_fixture() -> (Vec<NoteSpanOut>, Vec<LyricSpanOut>) {
    let (note_spans, mut lyric_spans) = fixture();
    lyric_spans.push(lyric_span(1, 3, 0, 1));
    (note_spans, lyric_spans)
}
