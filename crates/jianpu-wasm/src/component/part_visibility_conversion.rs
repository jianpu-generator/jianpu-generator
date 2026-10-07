use super::*;

pub(super) fn part_info_from_wit(part: Part) -> jianpu_generator::PartInfo {
    jianpu_generator::PartInfo {
        abbreviation: part.abbreviation,
        display_name: part.display_name,
    }
}

pub(super) fn part_toggle_state_from_wit(
    state: PartToggleState,
) -> jianpu_generator::PartToggleState {
    jianpu_generator::PartToggleState {
        hidden_parts: state.hidden_parts,
        soloed_parts: state.soloed_parts,
    }
}

pub(super) fn resolved_part_visibility_from_wit(
    visibility: ResolvedPartVisibility,
) -> jianpu_generator::ResolvedPartVisibility {
    jianpu_generator::ResolvedPartVisibility {
        rendered_tracks: visibility.rendered_tracks,
        sounding_tracks: visibility.sounding_tracks,
        sounding_display_names: visibility.sounding_display_names,
    }
}

pub(super) fn resolved_part_visibility_to_wit(
    visibility: jianpu_generator::ResolvedPartVisibility,
) -> ResolvedPartVisibility {
    ResolvedPartVisibility {
        rendered_tracks: visibility.rendered_tracks,
        sounding_tracks: visibility.sounding_tracks,
        sounding_display_names: visibility.sounding_display_names,
    }
}
