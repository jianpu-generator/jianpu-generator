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
                part_slice.lyrics = Vec::new();
            }
        }
    }
}

/// The part-toggle UI's raw show/hide/solo state, as abbreviations. A part
/// with lyrics has two independent rows — its notes row and its lyrics row —
/// each with its own hidden and soloed flag; [`resolve_part_visibility`] turns
/// this into what is drawn and what sounds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PartToggleState {
    pub hidden_notes: Vec<String>,
    pub soloed_notes: Vec<String>,
    pub hidden_lyrics: Vec<String>,
    pub soloed_lyrics: Vec<String>,
}

/// What [`resolve_part_visibility`] decided for a score's parts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedPartVisibility {
    /// Parts drawn at all (notes row and/or lyrics row), for
    /// [`apply_track_filter`] and the part-list legend. `None` when every
    /// part is drawn.
    pub rendered_tracks: Option<Vec<String>>,
    /// Parts whose notes row is shown, and so sound in playback. `None` when
    /// every part's notes row is shown.
    pub sounding_tracks: Option<Vec<String>>,
    /// Display names of `sounding_tracks`, for export filenames.
    pub sounding_display_names: Option<Vec<String>>,
    /// Parts drawn with only their lyrics: the notes row is hidden but the
    /// lyrics row is shown. A subset of `rendered_tracks`.
    pub lyrics_only_tracks: Vec<String>,
    /// Parts drawn with notes but without their lyrics, for
    /// [`apply_lyrics_filter`].
    pub disabled_lyrics: Vec<String>,
}

/// Resolves the raw toggle state against `parts`.
///
/// Rows are independent: hiding or soloing a notes row never changes its
/// lyrics row, and vice versa. Solo is uniform across every row — once any
/// row is soloed, only soloed rows show and hide flags are ignored. Solo
/// entries naming no row that exists (an unknown part, or lyrics on a part
/// with none) are ignored.
pub fn resolve_part_visibility(
    parts: &[crate::part_info::PartInfo],
    state: &PartToggleState,
) -> ResolvedPartVisibility {
    let contains =
        |names: &[String], part: &crate::part_info::PartInfo| names.contains(&part.abbreviation);
    let any_solo = parts.iter().any(|part| {
        contains(&state.soloed_notes, part)
            || (part.has_lyrics && contains(&state.soloed_lyrics, part))
    });
    let notes_shown = |part: &crate::part_info::PartInfo| {
        if any_solo {
            contains(&state.soloed_notes, part)
        } else {
            !contains(&state.hidden_notes, part)
        }
    };
    let lyrics_shown = |part: &crate::part_info::PartInfo| {
        part.has_lyrics
            && if any_solo {
                contains(&state.soloed_lyrics, part)
            } else {
                !contains(&state.hidden_lyrics, part)
            }
    };

    let abbreviations = |selected: Vec<&crate::part_info::PartInfo>| {
        selected
            .into_iter()
            .map(|part| part.abbreviation.clone())
            .collect::<Vec<_>>()
    };
    let or_none_when_all =
        |selected: Vec<&crate::part_info::PartInfo>,
         map: fn(&crate::part_info::PartInfo) -> String| {
            (selected.len() != parts.len())
                .then(|| selected.into_iter().map(map).collect::<Vec<_>>())
        };

    let rendered: Vec<_> = parts
        .iter()
        .filter(|part| notes_shown(part) || lyrics_shown(part))
        .collect();
    let sounding: Vec<_> = parts.iter().filter(|part| notes_shown(part)).collect();
    let lyrics_only: Vec<_> = parts
        .iter()
        .filter(|part| !notes_shown(part) && lyrics_shown(part))
        .collect();
    let disabled_lyrics: Vec<_> = parts
        .iter()
        .filter(|part| part.has_lyrics && notes_shown(part) && !lyrics_shown(part))
        .collect();

    ResolvedPartVisibility {
        rendered_tracks: or_none_when_all(rendered, |part| part.abbreviation.clone()),
        sounding_display_names: or_none_when_all(sounding.clone(), |part| {
            part.display_name.clone()
        }),
        sounding_tracks: or_none_when_all(sounding, |part| part.abbreviation.clone()),
        lyrics_only_tracks: abbreviations(lyrics_only),
        disabled_lyrics: abbreviations(disabled_lyrics),
    }
}

#[cfg(test)]
#[path = "filters_tests.rs"]
mod tests;
