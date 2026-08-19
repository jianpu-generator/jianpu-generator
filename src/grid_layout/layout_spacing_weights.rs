//! The "spring" half of `layout_spacing`'s spring-and-rod model: each
//! column/measure's proportional width weight, split out of
//! `layout_spacing.rs` to keep that file under the max line-count lint. See
//! **Rod and spring** in `ARCHITECTURE.md`.

use crate::ast::parsed::Accidental;
use crate::compiler::types::{ElementContent, MeasureBlock, MULTI_MEASURE_REST_WIDTH};
use crate::font_metrics;
use crate::render_config::RenderConfig;

/// Relative width weight of a `BarLine` column — just a thin mark, so it
/// gets much less than a fresh note. Elements that don't occupy their own
/// column-worth of ink (`Underline`) or that are handled separately
/// (`MultiMeasureRest`) contribute nothing here. Kept as an arbitrary flat
/// ratio rather than a measured width: a bar line is a drawn stroke, not a
/// glyph, so "actual rendered width" doesn't apply to it.
pub(super) const THIN_MARK_WEIGHT: f32 = 0.25;

/// Real advance width (in points) of one notehead/rest/percussion-hit digit
/// glyph at `config`'s notes font size — the unit every other weight below
/// is expressed in relative terms of, since notehead/rest/percussion glyphs
/// all render as a single monospace character (see `render_note_head`/
/// `render_rest`/`render_percussion_hit` in `glyph_renderers.rs`), so any one
/// of their digits/`0`/`x` characters measures the same.
fn note_glyph_weight(config: &RenderConfig) -> f32 {
    font_metrics::monospace_char_advance_width('0', config.notes_font_size())
}

/// Extra clearance (points) added on top of a dot glyph's own measured reach
/// ([`font_metrics::note_ish_dot_reach`]), so a dotted column's rod doesn't
/// land exactly flush with the dot's rightmost ink. Its own dedicated
/// padding, deliberately smaller than the general `super::COLUMN_CLEARANCE_PT`
/// every column gets, so a dot still reads as bound tightly to the
/// note/rest/dash it decorates rather than as spaced out like a column of
/// its own.
const DOT_CLEARANCE_PT: f32 = 0.5;

/// Extra clearance (points) added on top of an accidental glyph's own
/// measured reach ([`accidental_extra_weight`]). Its own dedicated padding,
/// deliberately smaller than the general `super::COLUMN_CLEARANCE_PT` every
/// column gets, so a sharp/flat still reads as bound tightly to the note it
/// modifies.
const ACCIDENTAL_CLEARANCE_PT: f32 = 0.5;

/// Extra weight given to a dotted `NoteHead`/`Rest`/`NoteDash` column to make
/// room for its augmentation dot(s), which are drawn alongside the glyph
/// rather than being baked into it (see `glyph_renderers.rs`'s
/// `render_note_head`/`render_rest` and `glyph_renderers_note_dash.rs`'s
/// `render_note_dash`). `base_weight` is what `column_weight` already
/// reserves for the glyph itself ([`note_glyph_weight`]/[`dash_weight`]) —
/// only the reach beyond that, plus [`DOT_CLEARANCE_PT`], is added, so a dot
/// that happens to fit within the glyph's own width contributes nothing
/// extra beyond that small clearance.
fn note_ish_dotted_extra_weight(
    dotted: bool,
    double_dotted: bool,
    note_number_width: f32,
    dot_font_size: f32,
    base_weight: f32,
) -> f32 {
    if !dotted {
        return 0.0;
    }
    let dot_count = if double_dotted { 2 } else { 1 };
    let reach = font_metrics::note_ish_dot_reach(dot_count, note_number_width, dot_font_size);
    (reach - base_weight).max(0.0) + DOT_CLEARANCE_PT
}

/// Extra weight given to a dotted `ChordSymbol` column, mirroring
/// [`note_ish_dotted_extra_weight`] but against `render_chord_symbol`'s own
/// distinct offset formula (first dot at `text_width + chords_font_size *
/// 0.4`, further dots `chords_font_size * 0.4` apart) rather than the
/// note/rest/dash one.
fn chord_symbol_dotted_extra_weight(
    text: &str,
    dotted: bool,
    double_dotted: bool,
    config: &RenderConfig,
) -> f32 {
    if !dotted {
        return 0.0;
    }
    let dot_count = if double_dotted { 2 } else { 1 };
    let font_size = config.chords_font_size();
    let spacing = font_size * 0.4;
    let last_dot_anchor = font_metrics::monospace_text_width(text, font_size)
        + spacing
        + (dot_count - 1) as f32 * spacing;
    let reach =
        last_dot_anchor + font_metrics::monospace_char_advance_width('\u{b7}', font_size) / 2.0;
    (reach - chord_symbol_weight(text, config)).max(0.0) + DOT_CLEARANCE_PT
}

/// Extra weight given to a `NoteHead` column carrying a sharp/flat
/// accidental, to make room for the `♯`/`♭` glyph drawn to the right of the
/// note head rather than being baked into it (see `render_note_head` in
/// `glyph_renderers.rs`, which starts the glyph at `elem.x +
/// note_number_width * ACCIDENTAL_LEFT_GAP_RATIO` and draws it at `1.25x`
/// the notes font size). The left gap keeps the glyph visually hugging the
/// note it modifies; on the right, rather than a flat guessed ratio, the
/// glyph's own measured advance width is used plus a small dedicated
/// [`ACCIDENTAL_CLEARANCE_PT`] so the reach tracks what the renderer actually
/// draws instead of over- or under-shooting it. Only the reach beyond what
/// [`note_glyph_weight`] already covers is added, so an accidental that
/// happens to fit within the note glyph's own width (e.g. at a large
/// `note_number_width`) contributes nothing extra. `Natural` renders no
/// glyph (see `render_note_head`), so it needs no extra weight either.
pub(super) fn accidental_extra_weight(accidental: &Accidental, config: &RenderConfig) -> f32 {
    let symbol = match accidental {
        Accidental::Sharp => "\u{266F}",
        Accidental::Flat => "\u{266D}",
        Accidental::Natural => return 0.0,
    };
    let reach = config.note_number_width as f32 * font_metrics::ACCIDENTAL_LEFT_GAP_RATIO
        + font_metrics::monospace_text_width(symbol, config.notes_font_size() * 1.25)
        + ACCIDENTAL_CLEARANCE_PT;
    (reach - note_glyph_weight(config)).max(0.0)
}

/// Real advance width (in points) of the note-dash glyph (`—`), measured at
/// its own fixed rendered font size (`NOTE_DASH_FONT_SIZE`) rather than
/// `config`'s lyric font size, matching what `render_note_dash` actually
/// draws.
fn dash_weight() -> f32 {
    font_metrics::monospace_char_advance_width('\u{2014}', font_metrics::NOTE_DASH_FONT_SIZE)
}

/// Width weight for a chord symbol's own glyph, measured from its real
/// rendered width in the monospace font (chord symbols render as monospace
/// text — see `render_chord_symbol`). Floored at [`note_glyph_weight`] so a
/// single-character chord (e.g. `1`) keeps the same weight as any other
/// note-like event.
fn chord_symbol_weight(symbol: &str, config: &RenderConfig) -> f32 {
    font_metrics::monospace_text_width(symbol, config.chords_font_size())
        .max(note_glyph_weight(config))
}

/// Width weight for a lyric syllable, measured from its real rendered width
/// — the CJK font/size if the syllable contains a CJK codepoint, the
/// monospace font/size otherwise — mirroring the same check `render_lyric`
/// already does to pick a font size.
fn lyric_weight(text: &str, config: &RenderConfig) -> f32 {
    if text.chars().any(font_metrics::is_cjk_char) {
        font_metrics::cjk_text_width(text, config.lyric_cjk_font_size())
    } else {
        font_metrics::monospace_text_width(text, config.lyric_font_size())
    }
}

pub(super) fn column_weight(content: &ElementContent, config: &RenderConfig) -> f32 {
    match content {
        ElementContent::NoteHead {
            accidental,
            dotted,
            double_dotted,
            ..
        } => {
            let base = note_glyph_weight(config);
            base + accidental_extra_weight(accidental, config)
                + note_ish_dotted_extra_weight(
                    *dotted,
                    *double_dotted,
                    config.note_number_width as f32,
                    config.notes_font_size(),
                    base,
                )
        }
        ElementContent::Rest {
            dotted,
            double_dotted,
        } => {
            let base = note_glyph_weight(config);
            base + note_ish_dotted_extra_weight(
                *dotted,
                *double_dotted,
                config.note_number_width as f32,
                config.notes_font_size(),
                base,
            )
        }
        ElementContent::ChordSymbol {
            text,
            dotted,
            double_dotted,
        } => {
            chord_symbol_weight(text, config)
                + chord_symbol_dotted_extra_weight(text, *dotted, *double_dotted, config)
        }
        ElementContent::PercussionHit => note_glyph_weight(config),
        ElementContent::NoteDash {
            dotted,
            double_dotted,
        } => {
            let base = dash_weight();
            base + note_ish_dotted_extra_weight(
                *dotted,
                *double_dotted,
                config.note_number_width as f32,
                font_metrics::NOTE_DASH_FONT_SIZE,
                base,
            )
        }
        ElementContent::Lyric { text, .. } => lyric_weight(text, config),
        // A `LyricLine` spans the whole measure via `column_span` rather than
        // occupying one grid column, so it contributes no per-column weight here;
        // its width is instead folded into the measure's total via `measure_note_weight`.
        ElementContent::LyricLine { .. } => 0.0,
        ElementContent::BarLine => THIN_MARK_WEIGHT,
        ElementContent::MultiMeasureRest { .. } | ElementContent::Underline { .. } => 0.0,
    }
}

/// How much horizontal room `block` should get relative to *other measures*
/// in its system — not to be confused with `measure_column_weights`, which
/// splits width *within* one measure. Only counts real note-starting
/// elements (notehead, rest, percussion hit, chord symbol); dashes and bar
/// lines don't contribute, so a measure of quarter notes gets roughly
/// double the aggregate weight of a measure of half notes spanning the same
/// duration (4 fresh notes vs. 2 notes + 2 dashes) — the whole point being
/// that dash-extended measures shouldn't out-compete note-dense measures for
/// width just because a dash happens to occupy its own column. A chord
/// symbol contributes its own [`chord_symbol_weight`] (its real rendered
/// width) rather than a flat [`note_glyph_weight`], so a slash chord with a
/// bass note (e.g. `2m/5`) out-competes a bare-degree chord (e.g. `1`) for
/// width. Weight is the max (not sum) across the block's part rows, so a
/// measure isn't penalized for having many parts, only sized for its
/// densest one. A collapsed `MultiMeasureRest` row gets a fixed weight
/// matching its current fixed column allocation instead of being counted as
/// one note. Clamped to a minimum of one note glyph's width
/// ([`note_glyph_weight`]) so an empty/rest-only measure never collapses to
/// zero weight, and so two equal-density measures always compare equal
/// regardless of which one happens to open the system (see
/// `build_measure_column_layout`'s leading bar-line column, which never
/// contributes here).
///
/// Deliberately **not** divided by the measure's tuplet `resolution_multiplier` (see
/// **Tuplet** in `ARCHITECTURE.md`): this counts *written note occurrences*, not raw grid
/// columns, and a tuplet's rescaled duration never changes how many `NoteHead`/`Rest`/
/// `PercussionHit` elements a measure has — 3 triplet-eighth notes still count as 3, the
/// same as 3 plain notes elsewhere, matching this function's existing note-count (not
/// note-duration) philosophy. A tuplet measure's grid column *count* is inflated by its
/// multiplier (see `block_column_width`), but that inflation never reaches a raw pixel
/// width anywhere in this module — every consumer of `col_count` below only ever indexes
/// or iterates it, then folds back down to a proportional (multiplier-invariant) split via
/// `measure_column_weights`/`column_weight`.
pub(super) fn measure_note_weight(block: &MeasureBlock, config: &RenderConfig) -> f32 {
    block
        .rows
        .iter()
        .map(|row| {
            let has_multi_measure_rest = row
                .elements
                .iter()
                .any(|e| matches!(e.content, ElementContent::MultiMeasureRest { .. }));
            if has_multi_measure_rest {
                MULTI_MEASURE_REST_WIDTH as f32
            } else {
                row.elements
                    .iter()
                    .map(|e| match &e.content {
                        ElementContent::NoteHead { accidental, .. } => {
                            note_glyph_weight(config) + accidental_extra_weight(accidental, config)
                        }
                        ElementContent::Rest { .. } | ElementContent::PercussionHit => {
                            note_glyph_weight(config)
                        }
                        ElementContent::ChordSymbol { text, .. } => {
                            chord_symbol_weight(text, config)
                        }
                        ElementContent::LyricLine { text, .. } => lyric_weight(text, config),
                        _ => 0.0,
                    })
                    .sum::<f32>()
            }
        })
        .fold(note_glyph_weight(config), f32::max)
}
