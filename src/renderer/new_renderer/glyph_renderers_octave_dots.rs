use super::dot_glyph;
use crate::compositor::types::AbsoluteElement;
use crate::renderer::new_types::{SvgElement, SvgVariant};

/// Size and center-to-center spacing of a note's octave dots, shared by the
/// inline dots in `inline_octave_dots` and `render_low_octave_dots`.
struct OctaveDotMetrics {
    radius: f32,
    spacing: f32,
}

impl OctaveDotMetrics {
    fn new(base_font_size: f32) -> Self {
        let radius = base_font_size * 0.1;
        Self {
            radius,
            spacing: radius * 3.0,
        }
    }
}

/// An underlined note's below-octave dots, stacked downward beneath its
/// lowest underline (`elem.y` is that underline's y). `elem.x` is the note
/// head's own flush-left anchor, so the dots center on the digit exactly as
/// `render_note_head`'s inline dots do.
pub(in crate::renderer::new_renderer) fn render_low_octave_dots(
    elem: &AbsoluteElement,
    count: u8,
    base_font_size: &f32,
    note_number_width: &f32,
) -> Vec<SvgElement> {
    let center = elem.x + *note_number_width * 0.5;
    let OctaveDotMetrics { radius, spacing } = OctaveDotMetrics::new(*base_font_size);
    (0..count)
        .map(|i| {
            // The first dot clears the underline by one `spacing` (center
            // to center), the same gap as between two stacked dots.
            let dot_y = elem.y + spacing * (i as f32 + 1.0);
            dot_glyph(center, dot_y, radius, SvgVariant::NoteHead)
        })
        .collect()
}

/// A note head's octave dots drawn inline with its digit, centered at
/// `center` around the digit's vertical center `y`: above for `octave > 0`,
/// below for `octave < 0`.
pub(super) fn inline_octave_dots(
    center: f32,
    y: f32,
    octave: i8,
    base_font_size: f32,
) -> Vec<SvgElement> {
    let OctaveDotMetrics {
        radius: dot_radius,
        spacing: dot_spacing,
    } = OctaveDotMetrics::new(base_font_size);
    // Octave-up dots sit above the digit and need extra clearance (`gap`) that
    // octave-down dots, sitting below, don't.
    let gap = dot_radius * 2.0;
    (0..octave.unsigned_abs())
        .map(|i| {
            let offset = dot_radius + (i as f32) * dot_spacing;
            let dot_y = if octave > 0 {
                y - base_font_size / 2.0 - offset - gap
            } else {
                y + base_font_size / 2.0 + offset
            };
            dot_glyph(center, dot_y, dot_radius, SvgVariant::NoteHead)
        })
        .collect()
}
