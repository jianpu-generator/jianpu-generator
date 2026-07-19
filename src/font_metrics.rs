//! Real glyph-advance measurement for directive-line text, shared by the
//! layout pass that positions the Segno glyph
//! (`src/coordinate_resolver/content_conversion.rs`) and the renderer pass
//! that sizes the section-label box (`src/renderer/new_renderer.rs`). Both
//! measure against the same pinned font so a width computed during layout
//! matches what actually renders (see Task 3/4 of
//! `PLAN-section-label-engraving-quality.md`).

use crate::compositor::types::TextSpan;

/// The font pinned for directive-line text (see `DIRECTIVE_LINE_FONT_FAMILY`
/// in `src/serializer/mod.rs`), parsed once so its real glyph advance
/// widths can be used instead of a character-bucket heuristic. `None` only
/// if the embedded font fails to parse, which shouldn't happen for a file
/// fixed at compile time.
static DIRECTIVE_LINE_FONT: std::sync::LazyLock<Option<ttf_parser::Face<'static>>> =
    std::sync::LazyLock::new(|| {
        ttf_parser::Face::parse(include_bytes!("../fonts/SourceHanSansSC-Regular.otf"), 0).ok()
    });

/// The pinned font only ships a Regular weight, so bold text (e.g. a section
/// label) is approximated by scaling Regular advance widths up, rather than
/// measuring an actual bold font that doesn't exist here.
const SYNTHETIC_BOLD_WIDTH_RATIO: f32 = 1.08;

/// Fallback advance width (as a fraction of `font_size`) for a character
/// missing from the pinned font, or if the font failed to parse.
const FALLBACK_ADVANCE_WIDTH_RATIO: f32 = 0.6;

/// Real advance width (in points) of one character at the given font size,
/// measured from the pinned font's `hmtx` table (see `DIRECTIVE_LINE_FONT`).
pub(crate) fn char_advance_width(c: char, font_size: f32, bold: bool) -> f32 {
    let measured = DIRECTIVE_LINE_FONT.as_ref().and_then(|face| {
        let glyph_id = face.glyph_index(c)?;
        let advance_units = face.glyph_hor_advance(glyph_id)?;
        Some(advance_units as f32 / face.units_per_em() as f32 * font_size)
    });
    let width = measured.unwrap_or(font_size * FALLBACK_ADVANCE_WIDTH_RATIO);
    if bold {
        width * SYNTHETIC_BOLD_WIDTH_RATIO
    } else {
        width
    }
}

/// Real rendered width (in points) of a text span, summing measured glyph
/// advances (see `char_advance_width`) for each character at the span's own
/// font size/weight.
pub(crate) fn span_width(span: &TextSpan) -> f32 {
    span.content
        .chars()
        .map(|c| char_advance_width(c, span.font_size, span.bold))
        .sum()
}

/// Font size (points) of a section label's own text run, shared by the
/// layout pass (`content_conversion.rs`, sizing the gap reserved before the
/// directives that follow a label) and the renderer pass (`new_renderer.rs`,
/// drawing the label's text and bounding box).
pub(crate) const SECTION_LABEL_FONT_SIZE: f32 = 12.0;

/// Horizontal padding between a section label's text and its bounding box,
/// applied equally on both sides — expressed as a ratio of the label's font
/// size, matching how real engraving software scales margins with the
/// notation's overall size, rather than as a fixed point value (see Task 5
/// of `PLAN-section-label-engraving-quality.md`).
const SECTION_LABEL_BOX_PADDING_RATIO: f32 = 1.0 / 3.0;

/// Height of a section label's bounding box, expressed as a ratio of the
/// label's font size for the same reason as
/// `SECTION_LABEL_BOX_PADDING_RATIO`.
const SECTION_LABEL_BOX_HEIGHT_RATIO: f32 = 1.5;

pub(crate) fn section_label_box_padding() -> f32 {
    SECTION_LABEL_FONT_SIZE * SECTION_LABEL_BOX_PADDING_RATIO
}

pub(crate) fn section_label_box_height() -> f32 {
    SECTION_LABEL_FONT_SIZE * SECTION_LABEL_BOX_HEIGHT_RATIO
}

/// Rendered width (in points) of a section label's bounding box, including
/// padding on both sides, measured from real font-metrics glyph advances
/// rather than a character-bucket heuristic. A section label is always bold
/// (see `section_label_span` in `content_conversion.rs`).
pub(crate) fn section_label_box_width(label: &str) -> f32 {
    label
        .chars()
        .map(|c| char_advance_width(c, SECTION_LABEL_FONT_SIZE, true))
        .sum::<f32>()
        + section_label_box_padding() * 2.0
}

/// Gap (in points) kept between a directive line and the row above it.
/// Larger than `DIRECTIVE_LINE_BOTTOM_PADDING` so the line reads as
/// attached to the musical row it annotates (below it) rather than the one
/// above.
pub(crate) const DIRECTIVE_LINE_TOP_PADDING: f32 = 24.0;

/// Gap (in points) kept between a bottom-aligned directive line (section
/// label, key, bpm, time signature) and the top of the musical row below
/// it, so the vertically-centered text doesn't dip into the measure
/// underneath.
pub(crate) const DIRECTIVE_LINE_BOTTOM_PADDING: f32 = 12.0;

/// Height of a directive-line row: exactly enough to hold its top and
/// bottom padding, since the directive text's own font size doesn't scale
/// with `base` (see `DirectiveLineArgs`'s fixed 12pt font in
/// `new_renderer/directive_line.rs`), so the row it sits in doesn't either.
pub(crate) fn directive_line_row_height() -> f32 {
    DIRECTIVE_LINE_TOP_PADDING + DIRECTIVE_LINE_BOTTOM_PADDING
}
