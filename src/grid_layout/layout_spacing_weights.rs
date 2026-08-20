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

/// Extra weight given to a dotted `NoteHead`/`Rest`/`NoteDash`/`ChordSymbol`
/// column to make room for its augmentation dot(s) — `render_note_head`/
/// `render_rest`/`render_note_dash`/`render_chord_symbol` (`glyph_renderers.rs`,
/// `glyph_renderers_note_dash.rs`) all append the dot(s) directly onto the
/// glyph's own text run (see [`font_metrics::augmentation_dot_suffix`])
/// rather than drawing them as a separately-positioned glyph, so the extra
/// width they need is exactly their own real rendered width — no clearance
/// or reach-vs-base-glyph reconciliation needed, since they're simply new
/// characters appended after the glyph's own.
fn dot_extra_weight(dotted: bool, double_dotted: bool, dot_font_size: f32) -> f32 {
    font_metrics::monospace_text_width(
        font_metrics::augmentation_dot_suffix(dotted, double_dotted),
        dot_font_size,
    )
}

/// Extra weight given to a `NoteHead` column carrying a sharp/flat
/// accidental, to make room for the `♯`/`♭` glyph — `render_note_head`
/// (`glyph_renderers.rs`) appends it directly onto the note digit's own text
/// run, at the same font size as the digit, rather than drawing it as its
/// own separately-positioned, larger glyph — so the extra width needed is
/// exactly the accidental's own real rendered width. `Natural` renders no
/// glyph, so it needs no extra weight either.
pub(super) fn accidental_extra_weight(accidental: &Accidental, config: &RenderConfig) -> f32 {
    let symbol = match accidental {
        Accidental::Sharp => "\u{266F}",
        Accidental::Flat => "\u{266D}",
        Accidental::Natural => return 0.0,
    };
    font_metrics::monospace_text_width(symbol, config.notes_font_size())
}

/// Real advance width (in points) of the note-dash glyph (`—`), measured at
/// `config`'s notes font size, matching what `render_note_dash` actually
/// draws.
fn dash_weight(config: &RenderConfig) -> f32 {
    font_metrics::monospace_char_advance_width('\u{2014}', config.notes_font_size())
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
            note_glyph_weight(config)
                + accidental_extra_weight(accidental, config)
                + dot_extra_weight(*dotted, *double_dotted, config.notes_font_size())
        }
        ElementContent::Rest {
            dotted,
            double_dotted,
        } => {
            note_glyph_weight(config)
                + dot_extra_weight(*dotted, *double_dotted, config.notes_font_size())
        }
        ElementContent::ChordSymbol {
            text,
            dotted,
            double_dotted,
        } => {
            chord_symbol_weight(text, config)
                + dot_extra_weight(*dotted, *double_dotted, config.chords_font_size())
        }
        ElementContent::PercussionHit => note_glyph_weight(config),
        ElementContent::NoteDash {
            dotted,
            double_dotted,
        } => {
            dash_weight(config)
                + dot_extra_weight(*dotted, *double_dotted, config.notes_font_size())
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
