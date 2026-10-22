use crate::ast::parsed::{Accidental, JianPuPitch};
use crate::compiler::types::{LetterNamePlacement, NoteHeadLetterName};
use crate::compositor::types::FontFamily;
use crate::font_metrics;

/// What a note head reads as: its jianpu degree (with any written
/// sharp/flat), or, in the letter-name view, `letter_name` in its place, or,
/// in the guitar chord view, the degree followed by `letter_name`.
pub(in crate::renderer::new_renderer) struct NoteHeadGlyph<'a> {
    pub(in crate::renderer::new_renderer) pitch: &'a JianPuPitch,
    pub(in crate::renderer::new_renderer) accidental: &'a Accidental,
    pub(in crate::renderer::new_renderer) letter_name: Option<&'a NoteHeadLetterName>,
}

impl NoteHeadGlyph<'_> {
    /// The text run's content: the letter name, or the accidental and digit
    /// (followed by the letter name in the guitar chord view).
    pub(super) fn content(&self) -> String {
        let digit = self.pitch.to_digit();
        match self.letter_name {
            Some(letter_name) if self.replaces_degree() => letter_name.label(digit),
            Some(letter_name) => format!(
                "{}{}",
                font_metrics::accidental_symbol(self.accidental),
                letter_name.label(digit)
            ),
            None => format!(
                "{}{}",
                font_metrics::accidental_symbol(self.accidental),
                digit
            ),
        }
    }

    /// Where the text run starts, given the glyph's anchor `x`. A digit's
    /// accidental leads it, so its run starts one accidental-width earlier; a
    /// letter name drawn in its place has its accidental trail its letter
    /// (`F♯`), so its run starts at `x` itself.
    pub(super) fn run_x(&self, x: f32, font_family: FontFamily, font_size: f32) -> f32 {
        if self.replaces_degree() {
            x
        } else {
            x - font_metrics::accidental_width_for_family(font_family, self.accidental, font_size)
        }
    }

    fn replaces_degree(&self) -> bool {
        self.letter_name
            .is_some_and(|name| name.placement == LetterNamePlacement::InPlaceOfDegree)
    }
}
