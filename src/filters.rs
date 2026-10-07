use crate::ast::grouped::Score;

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
    }
}

/// Retain only parts whose names appear in `tracks`. No-op when `tracks` is empty.
pub fn filter_tracks(score: &mut Score, tracks: &[String]) {
    if tracks.is_empty() {
        return;
    }
    apply_track_filter(score, Some(tracks));
}

/// Filters `score` down to the parts in `enabled_tracks`, for drawing.
///
/// A lyric part is enabled by its own abbreviation, like any other part: a
/// part keeps only the lyric verses whose abbreviation is enabled, and a part
/// whose own abbreviation is not enabled stays drawn — with its notes row
/// dropped later by the consolidator — as long as one of its verses is.
/// Returns the abbreviations of those lyrics-only parts.
///
/// `None` keeps everything. `Some([])` removes every part.
pub fn apply_visibility_filter(
    score: &mut Score,
    enabled_tracks: Option<&[String]>,
) -> Vec<String> {
    let Some(tracks) = enabled_tracks else {
        return Vec::new();
    };
    let mut lyrics_only = Vec::new();
    for measure in &mut score.measures {
        measure.parts.retain_mut(|part| {
            let slice = part.slice_mut();
            slice.lyrics.retain(|lyrics| tracks.contains(&lyrics.label));
            let Some(name) = slice.name.clone() else {
                return false;
            };
            if tracks.contains(&name) {
                return true;
            }
            let keeps_lyrics = !slice.lyrics.is_empty();
            if keeps_lyrics && !lyrics_only.contains(&name) {
                lyrics_only.push(name);
            }
            keeps_lyrics
        });
    }
    lyrics_only
}

/// The part-toggle UI's raw show/hide/solo state, as abbreviations. Every part
/// — notes, chords, percussion or lyrics — is toggled the same way;
/// [`resolve_part_visibility`] turns this into what is drawn and what sounds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PartToggleState {
    pub hidden_parts: Vec<String>,
    pub soloed_parts: Vec<String>,
}

/// What [`resolve_part_visibility`] decided for a score's parts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedPartVisibility {
    /// Parts drawn, for [`apply_visibility_filter`] and the part-list legend.
    /// `None` when every part is drawn.
    pub rendered_tracks: Option<Vec<String>>,
    /// Parts that sound in playback. `None` when every part sounds.
    pub sounding_tracks: Option<Vec<String>>,
    /// Display names of `sounding_tracks`, for export filenames.
    pub sounding_display_names: Option<Vec<String>>,
}

/// Resolves the raw toggle state against `parts`.
///
/// Solo wins over hide: once any part is soloed, only soloed parts show and
/// hide flags are ignored. Solo entries naming no part are ignored.
pub fn resolve_part_visibility(
    parts: &[crate::part_info::PartInfo],
    state: &PartToggleState,
) -> ResolvedPartVisibility {
    let contains =
        |names: &[String], part: &crate::part_info::PartInfo| names.contains(&part.abbreviation);
    let any_solo = parts.iter().any(|part| contains(&state.soloed_parts, part));
    let shown: Vec<_> = parts
        .iter()
        .filter(|part| {
            if any_solo {
                contains(&state.soloed_parts, part)
            } else {
                !contains(&state.hidden_parts, part)
            }
        })
        .collect();
    let or_none_when_all = |map: fn(&crate::part_info::PartInfo) -> String| {
        (shown.len() != parts.len()).then(|| shown.iter().map(|part| map(part)).collect())
    };

    ResolvedPartVisibility {
        rendered_tracks: or_none_when_all(|part| part.abbreviation.clone()),
        sounding_tracks: or_none_when_all(|part| part.abbreviation.clone()),
        sounding_display_names: or_none_when_all(|part| part.display_name.clone()),
    }
}

#[cfg(test)]
#[path = "filters_tests.rs"]
mod tests;
