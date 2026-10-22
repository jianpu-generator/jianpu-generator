use crate::ast::parsed::{Accidental, JianPuPitch};
use crate::compositor::types::FontFamily;
use crate::font_metrics;

/// What a note head reads as: its jianpu degree (with any written
/// sharp/flat), or, in the letter-name view, `letter_name` in its place.
pub(in crate::renderer::new_renderer) struct NoteHeadGlyph<'a> {
    pub(in crate::renderer::new_renderer) pitch: &'a JianPuPitch,
    pub(in crate::renderer::new_renderer) accidental: &'a Accidental,
    pub(in crate::renderer::new_renderer) letter_name: Option<&'a str>,
}

impl NoteHeadGlyph<'_> {
    /// The text run's content: the letter name, or the accidental and digit.
    pub(super) fn content(&self) -> String {
        match self.letter_name {
            Some(letter_name) => letter_name.to_string(),
            None => format!(
                "{}{}",
                font_metrics::accidental_symbol(self.accidental),
                self.pitch.to_digit()
            ),
        }
    }

    /// Where the text run starts, given the glyph's anchor `x`. A digit's
    /// accidental leads it, so its run starts one accidental-width earlier; a
    /// letter name's accidental trails its letter (`F♯`), so its run starts
    /// at `x` itself.
    pub(super) fn run_x(&self, x: f32, font_family: FontFamily, font_size: f32) -> f32 {
        match self.letter_name {
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
