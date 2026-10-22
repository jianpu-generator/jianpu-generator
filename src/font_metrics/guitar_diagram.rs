//! Geometry of the guitar chord diagram the guitar chord view draws above a
//! chord symbol, in points — shared by the layout pass, which reserves room
//! for it (`grid_layout`'s guitar-diagram band and the chord column's rod),
//! and the renderer, which draws it (`render_guitar_diagram`), so the two
//! can't drift apart.

use crate::pitch_description::GuitarVoicing;

pub(crate) const STRING_COUNT: usize = 6;
pub(crate) const FRET_ROWS: u8 = 4;
pub(crate) const STRING_SPACING_PT: f32 = 3.6;
pub(crate) const FRET_SPACING_PT: f32 = 4.4;
pub(crate) const DOT_RADIUS_PT: f32 = 1.5;
/// Radius of an open string's ring, and half the size of a muted string's `×`.
pub(crate) const STRING_MARKER_RADIUS_PT: f32 = 1.1;
/// Height of the band above the nut holding the open/muted string markers.
pub(crate) const STRING_MARKER_BAND_PT: f32 = 4.0;
pub(crate) const BASE_FRET_LABEL_FONT_SIZE_PT: f32 = 6.0;
/// Gap between the grid's right edge and the base-fret label (`5fr`).
pub(crate) const BASE_FRET_LABEL_GAP_PT: f32 = 1.5;
/// Gap between the diagram and the chord symbol under it.
pub(crate) const GAP_BELOW_PT: f32 = 3.0;
const GAP_ABOVE_PT: f32 = 2.0;

pub(crate) fn grid_width_pt() -> f32 {
    STRING_SPACING_PT * (STRING_COUNT - 1) as f32
}

pub(crate) fn grid_height_pt() -> f32 {
    FRET_SPACING_PT * f32::from(FRET_ROWS)
}

/// The base-fret label (`5fr`), or `None` for a diagram that starts at the
/// nut.
pub(crate) fn base_fret_label(voicing: &GuitarVoicing) -> Option<String> {
    (voicing.base_fret > 1).then(|| format!("{}fr", voicing.base_fret))
}

/// How wide `voicing`'s diagram draws, from its leftmost dot to the end of
/// its base-fret label, if any.
pub(crate) fn width_pt(voicing: &GuitarVoicing) -> f32 {
    let label_width = base_fret_label(voicing).map_or(0.0, |label| {
        BASE_FRET_LABEL_GAP_PT
            + super::text_width_for_family(
                crate::compositor::types::FontFamily::SansSerif,
                &label,
                BASE_FRET_LABEL_FONT_SIZE_PT,
            )
    });
    DOT_RADIUS_PT + grid_width_pt() + DOT_RADIUS_PT + label_width
}

/// Height of the band a chord part reserves above its chord symbols for
/// their diagrams.
pub(crate) fn band_height_pt() -> f32 {
    GAP_ABOVE_PT + STRING_MARKER_BAND_PT + grid_height_pt() + GAP_BELOW_PT
}
