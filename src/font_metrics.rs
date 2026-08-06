//! Real glyph-advance measurement for directive-line text, shared by the
//! layout pass that positions the Segno glyph
//! (`src/coordinate_resolver/content_conversion.rs`) and the renderer pass
//! that sizes the section-label box (`src/renderer/new_renderer.rs`). Both
//! measure against the same pinned font so a width computed during layout
//! matches what actually renders (see Task 3/4 of
//! `PLAN-section-label-engraving-quality.md`).
//!
//! Also used by `grid_layout::layout_spacing` for measure-spacing weights
//! (notehead/rest/chord-symbol/note-dash/lyric glyph widths, via
//! `monospace_char_advance_width`/`monospace_text_width`/`cjk_text_width`)
//! and by `renderer::new_renderer::glyph_renderers` (chord-symbol dot
//! placement, note-dash font size), so a measure's computed layout width and
//! its actually-rendered glyph widths can't drift apart.

use crate::compositor::types::TextSpan;

// Non-wasm builds (the CLI, `cargo test`) embed the fonts at compile time,
// exactly as before.
#[cfg(not(target_arch = "wasm32"))]
mod font_source {
    /// The font pinned for directive-line text (see
    /// `DIRECTIVE_LINE_FONT_FAMILY` in `src/serializer/mod.rs`), parsed once
    /// so its real glyph advance widths can be used instead of a
    /// character-bucket heuristic. `None` only if the embedded font fails to
    /// parse, which shouldn't happen for a file fixed at compile time.
    static DIRECTIVE_LINE_FONT: std::sync::LazyLock<Option<ttf_parser::Face<'static>>> =
        std::sync::LazyLock::new(|| {
            ttf_parser::Face::parse(include_bytes!("../fonts/SourceHanSansSC-Regular.otf"), 0).ok()
        });

    /// The font pinned for monospace glyphs (notehead digits, rests, chord
    /// symbols, note dashes, Latin lyric syllables — see
    /// `FontFamily::Monospace` resolving to `"Noto Sans Mono", monospace` in
    /// `src/serializer/mod.rs`), parsed once so layout weights can be
    /// measured against the same font that actually renders.
    static MONOSPACE_FONT: std::sync::LazyLock<Option<ttf_parser::Face<'static>>> =
        std::sync::LazyLock::new(|| {
            ttf_parser::Face::parse(include_bytes!("../fonts/NotoSansMono-Regular.ttf"), 0).ok()
        });

    pub(crate) fn directive_line_font() -> Option<&'static ttf_parser::Face<'static>> {
        DIRECTIVE_LINE_FONT.as_ref()
    }

    pub(crate) fn monospace_font() -> Option<&'static ttf_parser::Face<'static>> {
        MONOSPACE_FONT.as_ref()
    }

    /// No-op on non-wasm builds: the font is already embedded at compile
    /// time, so there's nothing to receive at runtime. Exists so callers
    /// (e.g. `crates/jianpu-wasm`, which is also built for the host arch as
    /// a workspace member) don't need their own `cfg` gate.
    pub(crate) fn set_directive_line_font_bytes(_bytes: Vec<u8>) {}

    /// No-op on non-wasm builds — see `set_directive_line_font_bytes`.
    pub(crate) fn set_monospace_font_bytes(_bytes: Vec<u8>) {}
}

// The wasm build has no compile-time font bytes: `set_directive_line_font_bytes`/
// `set_monospace_font_bytes` are called at runtime (from `crates/jianpu-wasm`)
// once the app has fetched the same font bytes it already needs for PDF
// export. Two `OnceLock`s per font (rather than a `LazyLock`) so a render
// that races ahead of the fetch just falls back to
// `FALLBACK_ADVANCE_WIDTH_RATIO` for that one call, instead of a `LazyLock`
// permanently caching `None` if it happened to be evaluated before the bytes
// arrived.
#[cfg(target_arch = "wasm32")]
mod font_source {
    use std::sync::OnceLock;

    static DIRECTIVE_LINE_FONT_BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    static DIRECTIVE_LINE_FONT_FACE: OnceLock<ttf_parser::Face<'static>> = OnceLock::new();
    static MONOSPACE_FONT_BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    static MONOSPACE_FONT_FACE: OnceLock<ttf_parser::Face<'static>> = OnceLock::new();

    pub(crate) fn set_directive_line_font_bytes(bytes: Vec<u8>) {
        DIRECTIVE_LINE_FONT_BYTES.set(bytes).ok();
    }

    pub(crate) fn set_monospace_font_bytes(bytes: Vec<u8>) {
        MONOSPACE_FONT_BYTES.set(bytes).ok();
    }

    fn face_from(
        bytes_cell: &'static OnceLock<Vec<u8>>,
        face_cell: &'static OnceLock<ttf_parser::Face<'static>>,
    ) -> Option<&'static ttf_parser::Face<'static>> {
        if let Some(face) = face_cell.get() {
            return Some(face);
        }
        let bytes = bytes_cell.get()?;
        let face = ttf_parser::Face::parse(bytes, 0).ok()?;
        Some(face_cell.get_or_init(|| face))
    }

    pub(crate) fn directive_line_font() -> Option<&'static ttf_parser::Face<'static>> {
        face_from(&DIRECTIVE_LINE_FONT_BYTES, &DIRECTIVE_LINE_FONT_FACE)
    }

    pub(crate) fn monospace_font() -> Option<&'static ttf_parser::Face<'static>> {
        face_from(&MONOSPACE_FONT_BYTES, &MONOSPACE_FONT_FACE)
    }
}

pub(crate) use font_source::{set_directive_line_font_bytes, set_monospace_font_bytes};

/// The pinned font only ships a Regular weight, so bold text (e.g. a section
/// label) is approximated by scaling Regular advance widths up, rather than
/// measuring an actual bold font that doesn't exist here.
const SYNTHETIC_BOLD_WIDTH_RATIO: f32 = 1.08;

/// Fallback advance width (as a fraction of `font_size`) for a character
/// missing from the pinned font, or if the font failed to parse.
const FALLBACK_ADVANCE_WIDTH_RATIO: f32 = 0.6;

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

/// Real rendered width (in points) of a string in the pinned CJK font (see
/// `DIRECTIVE_LINE_FONT`), used for CJK lyric syllables. A plain-`&str`
/// counterpart to `span_width`, which requires a `TextSpan`.
pub(crate) fn cjk_text_width(s: &str, font_size: f32) -> f32 {
    s.chars()
        .map(|c| char_advance_width(c, font_size, false))
        .sum()
}

/// Font size (points) at which a note-dash glyph (`—`) is rendered (see
/// `render_note_dash`), fixed independently of `base_font_size` so it always
/// reads as visually smaller than a full notehead. Shared with the layout
/// pass's dash weight (`layout_spacing.rs`) so the two can't silently drift
/// apart.
pub(crate) const NOTE_DASH_FONT_SIZE: f32 = 12.0;

/// Horizontal gap (as a ratio of `note_number_width`) between a note head
/// and its sharp/flat accidental glyph, drawn to the accidental's left in
/// `render_note_head`. Kept small so the accidental visually hugs the note
/// it modifies, rather than reading as its own free-floating glyph. Shared
/// with the layout pass's accidental weight (`layout_spacing.rs`) so the two
/// can't silently drift apart.
pub(crate) const ACCIDENTAL_LEFT_GAP_RATIO: f32 = 0.2;

/// Horizontal padding (as a ratio of `note_number_width`) reserved to the
/// right of a sharp/flat accidental's own glyph, on top of
/// [`ACCIDENTAL_LEFT_GAP_RATIO`]'s note-to-accidental gap. Deliberately
/// larger than the left gap: a small gap on the left reads as "this
/// accidental belongs to the note on its left," while a larger gap on the
/// right keeps it from reading as belonging to the *next* note/dash column
/// instead.
pub(crate) const ACCIDENTAL_RIGHT_PADDING_RATIO: f32 = 1.0;

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
