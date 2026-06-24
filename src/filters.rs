use crate::ast::grouped::{PartRow, Score};
use crate::ast::parsed::PartKind;

/// Retain only parts whose names appear in `enabled_tracks`.
///
/// `None` keeps every part. `Some([])` removes every part.
pub fn apply_track_filter(score: &mut Score, enabled_tracks: Option<&[String]>) {
    let Some(tracks) = enabled_tracks else {
        return;
    };
    for measure in &mut score.measures {
        measure.parts.retain(|part| {
            part.name()
                .as_ref()
                .is_some_and(|name| tracks.contains(name))
        });
        // If the source part was filtered out, a leading ditto has no row to
        // merge into and its content would silently disappear. Promote the
        // first ditto part to Timed so it renders independently.
        if let Some(first) = measure.parts.first_mut() {
            if let PartRow::NotMentioned(slice) = first {
                *first = PartRow::Timed(slice.clone());
            }
        }
    }
}

/// Retain only parts whose names appear in `tracks`. No-op when `tracks` is empty.
pub fn filter_tracks(score: &mut Score, tracks: &[String]) {
    if tracks.is_empty() {
        return;
    }
    apply_track_filter(score, Some(tracks));
}

/// Hide lyrics on parts whose abbreviations appear in `disabled_lyrics`.
///
/// `None` and `Some([])` keep every lyric line.
pub fn apply_lyrics_filter(score: &mut Score, disabled_lyrics: Option<&[String]>) {
    let Some(tracks) = disabled_lyrics else {
        return;
    };
    if tracks.is_empty() {
        return;
    }
    for measure in &mut score.measures {
        for part in &mut measure.parts {
            let part_slice = part.slice_mut();
            if part_slice
                .name
                .as_ref()
                .is_some_and(|name| tracks.contains(name))
            {
                part_slice.lyrics = None;
                if matches!(
                    part_slice.kind,
                    PartKind::NotesWithLyrics | PartKind::LyricsWithNotes
                ) {
                    part_slice.kind = PartKind::Notes;
                }
            }
        }
    }
}
