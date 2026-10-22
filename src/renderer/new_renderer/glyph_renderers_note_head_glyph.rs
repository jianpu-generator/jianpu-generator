use crate::ast::parsed::{Accidental, JianPuPitch};
use crate::compiler::NoteHeadLabel;
use crate::compositor::types::FontFamily;
use crate::font_metrics;
use crate::renderer::new_types::SvgVariant;

/// What a note head reads as: its jianpu degree (with any written
/// sharp/flat), or, in the letter-name and harmonica-tab views, `head_label`
/// in its place.
pub(in crate::renderer::new_renderer) struct NoteHeadGlyph<'a> {
    pub(in crate::renderer::new_renderer) pitch: &'a JianPuPitch,
    pub(in crate::renderer::new_renderer) accidental: &'a Accidental,
    pub(in crate::renderer::new_renderer) head_label: Option<&'a NoteHeadLabel>,
}

impl NoteHeadGlyph<'_> {
    /// The text run's content: the label, or the accidental and digit.
    pub(super) fn content(&self) -> String {
        match self.head_label {
            Some(head_label) => head_label.text.clone(),
            None => format!(
                "{}{}",
                font_metrics::accidental_symbol(self.accidental),
                self.pitch.to_digit()
            ),
        }
    }

    /// The text run's styling hook: an unplayable label is flagged as such.
    pub(super) fn variant(&self) -> SvgVariant {
        match self.head_label {
            Some(NoteHeadLabel {
                unplayable: true, ..
            }) => SvgVariant::UnplayableNoteHead,
            _ => SvgVariant::NoteHead,
        }
    }

    /// Where the text run starts, given the glyph's anchor `x`. A digit's
    /// accidental leads it, so its run starts one accidental-width earlier; a
    /// label's accidental trails its letter (`F♯`), so its run starts at `x`
    /// itself.
    pub(super) fn run_x(&self, x: f32, font_family: FontFamily, font_size: f32) -> f32 {
        match self.head_label {
            Some(_) => x,
            None => {
                x - font_metrics::accidental_width_for_family(
                    font_family,
                    self.accidental,
                    font_size,
                )
            }
        }
    }
}
