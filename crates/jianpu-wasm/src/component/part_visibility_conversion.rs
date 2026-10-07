use super::*;

pub(super) fn part_info_from_wit(part: Part) -> jianpu_generator::PartInfo {
    jianpu_generator::PartInfo {
        abbreviation: part.abbreviation,
        display_name: part.display_name,
        has_lyrics: part.has_lyrics,
    }
}

pub(super) fn part_toggle_state_from_wit(
    state: PartToggleState,
) -> jianpu_generator::PartToggleState {
    jianpu_generator::PartToggleState {
        hidden_notes: state.hidden_notes,
        soloed_notes: state.soloed_notes,
        hidden_lyrics: state.hidden_lyrics,
        soloed_lyrics: state.soloed_lyrics,
    }
}

pub(super) fn resolved_part_visibility_from_wit(
    visibility: ResolvedPartVisibility,
) -> jianpu_generator::ResolvedPartVisibility {
    jianpu_generator::ResolvedPartVisibility {
        rendered_tracks: visibility.rendered_tracks,
        sounding_tracks: visibility.sounding_tracks,
        sounding_display_names: visibility.sounding_display_names,
        lyrics_only_tracks: visibility.lyrics_only_tracks,
        disabled_lyrics: visibility.disabled_lyrics,
    }
}

pub(super) fn resolved_part_visibility_to_wit(
    visibility: jianpu_generator::ResolvedPartVisibility,
) -> ResolvedPartVisibility {
    ResolvedPartVisibility {
        rendered_tracks: visibility.rendered_tracks,
        sounding_tracks: visibility.sounding_tracks,
        sounding_display_names: visibility.sounding_display_names,
        lyrics_only_tracks: visibility.lyrics_only_tracks,
        disabled_lyrics: visibility.disabled_lyrics,
    }
}
