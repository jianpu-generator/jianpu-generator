use crate::ast::parsed::{Accidental, JianPuPitch};
use crate::compiler::NoteHeadLabel;
use crate::compositor::types::FontFamily;
use crate::font_metrics;
use crate::renderer::new_types::SvgVariant;

/// What a note head reads as: its jianpu degree (with any written
/// sharp/flat), or, in the letter-name and harmonica-tab views, `head_label`
/// in its place, or, in the guitar chord view, the degree followed by
/// `head_label`.
pub(in crate::renderer::new_renderer) struct NoteHeadGlyph<'a> {
    pub(in crate::renderer::new_renderer) pitch: &'a JianPuPitch,
    pub(in crate::renderer::new_renderer) accidental: &'a Accidental,
    pub(in crate::renderer::new_renderer) head_label: Option<&'a NoteHeadLabel>,
}

impl NoteHeadGlyph<'_> {
    /// The text run's content: the label, or the accidental and digit
    /// (followed by the label in the guitar chord view).
    pub(super) fn content(&self) -> String {
        let digit = self.pitch.to_digit();
        match self.head_label {
            Some(head_label) if head_label.replaces_degree() => head_label.run_text(digit),
            Some(head_label) => format!(
                "{}{}",
                font_metrics::accidental_symbol(self.accidental),
                head_label.run_text(digit)
            ),
            None => format!(
                "{}{}",
                font_metrics::accidental_symbol(self.accidental),
                digit
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
    /// label drawn in its place has its accidental trail its letter (`F♯`),
    /// so its run starts at `x` itself.
    pub(super) fn run_x(&self, x: f32, font_family: FontFamily, font_size: f32) -> f32 {
        match self.head_label {
            Some(head_label) if head_label.replaces_degree() => x,
            _ => {
                x - font_metrics::accidental_width_for_family(
                    font_family,
                    self.accidental,
                    font_size,
                )
            }
        }
    }
}
