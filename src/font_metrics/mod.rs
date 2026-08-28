//! Real glyph-advance measurement for directive-line and lyric text, shared
//! by the layout pass that positions the Segno glyph
//! (`src/coordinate_resolver/content_conversion.rs`) and the renderer pass
//! that sizes the section-label box (`src/renderer/new_renderer.rs`). Both
//! measure against the same pinned fonts so a width computed during layout
//! matches what actually renders (see Task 3/4 of
//! `PLAN-section-label-engraving-quality.md`).
//!
//! Two separate pinned faces back this, mirroring the
//! `DIRECTIVE_LINE_FONT_FAMILY`/`TITLE_FONT_FAMILY` split in
//! `src/serializer/mod.rs`: `directive_line_font()` (the `sansSerif` role in
//! `fonts/fonts.json`) for the directive line's own text, and `lyric_font()`
//! (the `title` role) for lyric syllables, which render in the same font as
//! the song title instead — currently the same Zhuque Fangsong file as
//! `directive_line_font()`, though the two roles can differ.
//!
//! Also used by `grid_layout::layout_spacing` for measure-spacing weights
//! (notehead/rest/chord-symbol/note-dash/lyric glyph widths, via
//! `monospace_char_advance_width`/`monospace_text_width`/`cjk_text_width`)
//! and by `renderer::new_renderer::glyph_renderers` (chord-symbol dot
//! placement, note-dash font size), so a measure's computed layout width and
//! its actually-rendered glyph widths can't drift apart.

use crate::compositor::types::TextSpan;

mod font_source;
#[cfg(test)]
mod tests;

pub(crate) use font_source::{
    set_directive_line_font_bytes, set_lyric_font_bytes, set_monospace_font_bytes,
};

/// The pinned font only ships a Regular weight, so bold text (e.g. a section
/// label) is approximated by scaling Regular advance widths up, rather than
/// measuring an actual bold font that doesn't exist here.
const SYNTHETIC_BOLD_WIDTH_RATIO: f32 = 1.08;

/// Fallback advance width (as a fraction of `font_size`) for a character
/// missing from the pinned font, or if the font failed to parse.
const FALLBACK_ADVANCE_WIDTH_RATIO: f32 = 0.6;

/// Fallback vertical extent (as a fraction of `font_size`) used only if the
/// pinned lyric font fails to parse or isn't loaded yet — mirrors
/// `FALLBACK_ADVANCE_WIDTH_RATIO`'s role for horizontal measurement.
const FALLBACK_VERTICAL_EXTENT_RATIO: f32 = 1.0;

/// Real advance width (in points) of one character at the given font size,
/// measured from `face`'s `hmtx` table, or the fallback ratio if the
/// character is missing from the font (or the font failed to parse).
fn face_char_advance_width(
    face: Option<&ttf_parser::Face<'static>>,
    c: char,
    font_size: f32,
) -> f32 {
    let measured = face.and_then(|face| {
        let glyph_id = face.glyph_index(c)?;
        let advance_units = face.glyph_hor_advance(glyph_id)?;
        Some(advance_units as f32 / face.units_per_em() as f32 * font_size)
    });
    measured.unwrap_or(font_size * FALLBACK_ADVANCE_WIDTH_RATIO)
}

/// Left-side bearing (in points) of one character in `face` at `font_size`
/// — the gap between the glyph's advance-box origin and where its ink
/// actually starts, per `glyph_bounding_box`. `0.0` if the glyph is
/// missing, has no outline (e.g. a space), or the font failed to parse.
fn face_glyph_left_bearing(
    face: Option<&ttf_parser::Face<'static>>,
    c: char,
    font_size: f32,
) -> f32 {
    let bearing = face.and_then(|face| {
        let glyph_id = face.glyph_index(c)?;
        let bbox = face.glyph_bounding_box(glyph_id)?;
        Some(bbox.x_min.max(0) as f32 / face.units_per_em() as f32 * font_size)
    });
    bearing.unwrap_or(0.0)
}

/// Real vertical extent (in points, ascender to descender) of `face` at the
/// given font size — `Face::height()` already equals
/// `ascender() - descender()`. Falls back to `FALLBACK_VERTICAL_EXTENT_RATIO`
/// if the font failed to parse or isn't loaded yet.
fn face_vertical_extent(face: Option<&ttf_parser::Face<'static>>, font_size: f32) -> f32 {
    let measured = face.map(|face| face.height() as f32 / face.units_per_em() as f32 * font_size);
    measured.unwrap_or(font_size * FALLBACK_VERTICAL_EXTENT_RATIO)
}

/// Left-side bearing (in points) of one character in the pinned lyric font
/// (see `lyric_font`), used to compensate `GLYPH_LEFT_PADDING` for CJK lyric
/// syllables' own built-in inset — see
/// `coordinate_resolver::resolve::flush_left_padding`.
pub(crate) fn cjk_glyph_left_bearing(c: char, font_size: f32) -> f32 {
    face_glyph_left_bearing(font_source::lyric_font(), c, font_size)
}

/// Real vertical extent (in points, ascender to descender) of the pinned
/// lyric font (see `lyric_font`) at the given font size — used to size a
/// lyric syllable's click-target row tall enough for its actual glyph
/// height instead of a hardcoded ratio (see `grid_layout::layout_heights::
/// lyric_row_height`).
pub(crate) fn lyric_vertical_extent(font_size: f32) -> f32 {
    face_vertical_extent(font_source::lyric_font(), font_size)
}

/// Left-side bearing (in points) of one character in the pinned monospace
/// font (see `font_source::monospace_font`), used to compensate a
/// centered/flush-left glyph's anchor for its own leading character's
/// built-in inset — the monospace counterpart to `cjk_glyph_left_bearing`.
pub(crate) fn monospace_glyph_left_bearing(c: char, font_size: f32) -> f32 {
    face_glyph_left_bearing(font_source::monospace_font(), c, font_size)
}

/// Whether `c` falls in the CJK Unified Ideographs block, the single check
/// shared by every place that needs to pick a CJK-sized font/glyph metric
/// for a character or decide a string counts as CJK (a lyric syllable's
/// rendered font size, its layout weight, and its left-side-bearing
/// correction all key off this).
pub(crate) fn is_cjk_char(c: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&c)
}

/// Font size (points) a lyric syllable/word renders at: `cjk` if any
/// character in `text` is CJK (see [`is_cjk_char`]), `base` otherwise —
/// mirroring `render_lyric`'s own font-size choice so a syllable's measured
/// width/bearing never disagrees with what actually renders.
pub(crate) fn lyric_font_size(text: &str, base: f32, cjk: f32) -> f32 {
    if text.chars().any(is_cjk_char) {
        cjk
    } else {
        base
    }
}

/// Real advance width (in points) of one character at the given font size,
/// measured from the pinned font's `hmtx` table (see `font_source::directive_line_font`).
pub(crate) fn char_advance_width(c: char, font_size: f32, bold: bool) -> f32 {
    let width = face_char_advance_width(font_source::directive_line_font(), c, font_size);
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

/// Real advance width (in points) of one character in the pinned monospace
/// font (see `font_source::monospace_font`), used for
/// notehead/rest/chord/dash/Latin-lyric glyphs.
pub(crate) fn monospace_char_advance_width(c: char, font_size: f32) -> f32 {
    face_char_advance_width(font_source::monospace_font(), c, font_size)
}

/// Real rendered width (in points) of a string in the pinned monospace font,
/// summing `monospace_char_advance_width` for each character.
pub(crate) fn monospace_text_width(s: &str, font_size: f32) -> f32 {
    s.chars()
        .map(|c| monospace_char_advance_width(c, font_size))
        .sum()
}

/// Real rendered width (in points) of a string in the pinned lyric font (see
/// `LYRIC_FONT`), used for CJK lyric syllables. A plain-`&str` counterpart
/// to `span_width`, which requires a `TextSpan` and measures against
/// `DIRECTIVE_LINE_FONT` instead.
pub(crate) fn cjk_text_width(s: &str, font_size: f32) -> f32 {
    s.chars()
        .map(|c| face_char_advance_width(font_source::lyric_font(), c, font_size))
        .sum()
}

/// Fixed horizontal padding (in points) between a column's left edge and the
/// anchor of every glyph inside it — note head, rest, percussion hit, chord
/// symbol, note dash, and lyric syllable — plus the
/// tie/slur/underline/tuplet-bracket span markings that key off the same
/// anchor. A flat point value rather than a ratio of `note_number_width`:
/// the padding should read as a fixed visual gap from the bar line/column
/// edge, not shrink toward invisible as the user's configured note size
/// shrinks. The same value everywhere so everything in a column lines up
/// flush at one offset from `x_start`, regardless of the column's own width
/// or what else shares it (see `ColumnGeometry::glyph_left_anchor_x`).
pub(crate) const GLYPH_LEFT_PADDING: f32 = 4.0;

/// The augmentation-dot(s) text (`.`/`..`, drawn as literal middle-dot
/// characters) appended directly onto a note/rest/chord/dash glyph's own
/// text run — `render_note_head`/`render_rest`/`render_note_dash`/
/// `render_chord_symbol` all append this to their glyph's own `content`
/// string rather than drawing the dot(s) as a separately-positioned glyph,
/// so the dot's position falls out of normal flush-left text flow instead
/// of a hand-computed offset. Shared with the layout pass (`column_weight`
/// measures the combined string's real rendered width via
/// `monospace_text_width`) so the two can't silently drift apart. `""` if
/// not dotted.
pub(crate) fn augmentation_dot_suffix(dotted: bool, double_dotted: bool) -> &'static str {
    if !dotted {
        ""
    } else if double_dotted {
        "\u{b7}\u{b7}"
    } else {
        "\u{b7}"
    }
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

/// Gap (points) reserved between adjacent directive-line elements (bar
/// number, section label, key/bpm/time-signature spans) — shared by
/// `coordinate_resolver::content_conversion` (positions them) and
/// `grid_layout::layout_decoration` (must reserve enough measure width for
/// them before positions are known).
pub(crate) const DIRECTIVE_LINE_ELEMENT_GAP: f32 = 20.0;

/// Total rendered width (points) of a directive line's bar number, section
/// label box, and trailing spans, laid out left-to-right in that order.
/// Mirrors the offset math in
/// `coordinate_resolver::content_conversion::directive_line_content` (the
/// authoritative positioning logic) — kept here so that layer and
/// `grid_layout::layout_decoration` (which must reserve width for this line
/// before rendering happens) share one implementation.
pub(crate) fn directive_line_width(
    bar_number: Option<&TextSpan>,
    label: Option<&str>,
    spans: &[TextSpan],
) -> f32 {
    let bar_number_width = bar_number.map(span_width).unwrap_or(0.0);
    let spans_width: f32 = spans.iter().map(span_width).sum();
    match label {
        Some(label_str) => {
            let label_x_offset = if bar_number.is_some() {
                bar_number_width + DIRECTIVE_LINE_ELEMENT_GAP
            } else {
                0.0
            };
            let label_box_right = label_x_offset + section_label_box_width(label_str);
            let spans_x_offset = label_box_right + DIRECTIVE_LINE_ELEMENT_GAP;
            bar_number_width
                .max(label_box_right)
                .max(spans_x_offset + spans_width)
        }
        None => bar_number_width + spans_width,
    }
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
