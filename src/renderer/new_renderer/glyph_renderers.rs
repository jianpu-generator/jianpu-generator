use crate::ast::parsed::{Accidental, JianPuPitch};
use crate::compositor::types::AbsoluteElement;
use crate::compositor::types::{DominantBaseline, FontFamily, FontWeight, TextAnchor};
use crate::font_metrics;
use crate::renderer::new_types::{SvgElement, SvgKind, SvgVariant};

pub(super) struct NoteRenderParams<'a> {
    pub(super) base_font_size: &'a f32,
    pub(super) note_number_width: &'a f32,
}

/// Whether a note/rest/chord-symbol/note-dash carries a first and/or second
/// duration dot (`.`/`..`).
pub(super) struct DotState {
    pub(super) dotted: bool,
    pub(super) double_dotted: bool,
}

impl DotState {
    pub(super) fn new(dotted: bool, double_dotted: bool) -> Self {
        Self {
            dotted,
            double_dotted,
        }
    }
}

/// A middle dot (`·`) rendered as its own centered text glyph, used only for
/// a note's *octave* dots (drawn above/below the digit — see
/// `render_note_head`'s `octave` loop). A note/rest/chord/dash's
/// *augmentation* dot(s) are handled differently: appended directly onto the
/// glyph's own text run (see `font_metrics::augmentation_dot_suffix`)
/// instead of drawn as a separate positioned glyph like this one.
pub(super) fn dot_glyph(x: f32, y: f32, font_size: f32, variant: SvgVariant) -> SvgElement {
    SvgElement {
        x,
        y,
        variant: Some(variant),
        kind: SvgKind::Text {
            content: "\u{b7}".to_string(),
            font_size,
            anchor: TextAnchor::Middle,
            baseline: DominantBaseline::Middle,
            font: FontFamily::Monospace,
            weight: FontWeight::Normal,
            italic: false,
        },
    }
}

pub(super) fn render_note_head(
    elem: &AbsoluteElement,
    pitch: &JianPuPitch,
    accidental: &Accidental,
    octave: i8,
    dots: &DotState,
    params: &NoteRenderParams<'_>,
) -> Vec<SvgElement> {
    let NoteRenderParams {
        base_font_size,
        note_number_width,
    } = params;
    let mut results = Vec::new();

    let accidental_symbol = match accidental {
        Accidental::Sharp => "♯",
        Accidental::Flat => "♭",
        Accidental::Natural => "",
    };

    // The digit, its sharp/flat accidental (if any), and its augmentation
    // dot(s) (if any) all draw as one flush-left text run at `elem.x` (see
    // `coordinate_resolver::resolve::flush_left_padding`, which already
    // corrects `elem.x` for this glyph's own left-side bearing) — the
    // accidental and dot(s) simply fall out of normal text flow immediately
    // after the digit, rather than each being drawn as its own
    // separately-positioned glyph at a hand-computed offset.
    let content = format!(
        "{}{}{}",
        pitch.to_digit(),
        accidental_symbol,
        font_metrics::augmentation_dot_suffix(dots.dotted, dots.double_dotted)
    );

    results.push(SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::NoteHead),
        kind: SvgKind::Text {
            content,
            font_size: **base_font_size,
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            font: FontFamily::Monospace,
            weight: FontWeight::Normal,
            italic: false,
        },
    });

    // Every decoration below still wants to sit relative to the digit's
    // nominal center, so `center` reconstructs that from the flat,
    // user-configurable `note_number_width` box, exactly as before this
    // glyph was flush-left anchored.
    let center = elem.x + *note_number_width * 0.5;
    let dot_radius = *base_font_size * 0.1;

    // Octave-up dots sit above the digit and need extra clearance (`gap`) that
    // octave-down dots, sitting below, don't.
    let dot_spacing = dot_radius * 3.0;
    let gap = dot_radius * 2.0;
    for i in 0..octave.unsigned_abs() {
        let offset = dot_radius + (i as f32) * dot_spacing;
        let dot_y = if octave > 0 {
            elem.y - *base_font_size / 2.0 - offset - gap
        } else {
            elem.y + *base_font_size / 2.0 + offset
        };
        results.push(dot_glyph(
            center,
            dot_y,
            **base_font_size,
            SvgVariant::NoteHead,
        ));
    }

    results
}

#[path = "glyph_renderers_note_dash.rs"]
mod note_dash;
pub(super) use note_dash::render_note_dash;

#[path = "glyph_renderers_rest.rs"]
mod rest;
pub(super) use rest::render_rest;

/// Standard multi-bar-rest engraving: a thick horizontal bar with short vertical
/// ticks at both ends, and the collapsed measure count printed centered above it.
pub(super) fn render_multi_measure_rest(
    elem: &AbsoluteElement,
    count: u32,
    width: f32,
    row_height: &f32,
    base_font_size: &f32,
) -> Vec<SvgElement> {
    let bar_stroke_width = row_height * 0.18;
    let tick_half_height = row_height * 0.25;

    vec![
        SvgElement {
            x: elem.x,
            y: elem.y,
            variant: Some(SvgVariant::MultiMeasureRest),
            kind: SvgKind::Line {
                x2: elem.x + width,
                y2: elem.y,
                stroke_width: bar_stroke_width,
            },
        },
        SvgElement {
            x: elem.x,
            y: elem.y - tick_half_height,
            variant: Some(SvgVariant::MultiMeasureRest),
            kind: SvgKind::Line {
                x2: elem.x,
                y2: elem.y + tick_half_height,
                stroke_width: 1.0,
            },
        },
        SvgElement {
            x: elem.x + width,
            y: elem.y - tick_half_height,
            variant: Some(SvgVariant::MultiMeasureRest),
            kind: SvgKind::Line {
                x2: elem.x + width,
                y2: elem.y + tick_half_height,
                stroke_width: 1.0,
            },
        },
        SvgElement {
            x: elem.x + width * 0.5,
            y: elem.y - *row_height * 0.5,
            variant: Some(SvgVariant::MultiMeasureRest),
            kind: SvgKind::Text {
                content: count.to_string(),
                font_size: *base_font_size,
                anchor: TextAnchor::Middle,
                baseline: DominantBaseline::Middle,
                font: FontFamily::Monospace,
                weight: FontWeight::Bold,
                italic: false,
            },
        },
    ]
}

pub(super) fn render_percussion_hit(
    elem: &AbsoluteElement,
    base_font_size: &f32,
) -> Vec<SvgElement> {
    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::PercussionHit),
        kind: SvgKind::Text {
            content: "x".to_string(),
            font_size: *base_font_size,
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            font: FontFamily::Monospace,
            weight: FontWeight::Normal,
            italic: false,
        },
    }]
}

pub(super) fn render_chord_symbol(
    elem: &AbsoluteElement,
    s: &str,
    dots: &DotState,
    base_font_size: &f32,
) -> Vec<SvgElement> {
    // `elem.x` is already corrected for this chord's own root character's
    // left-side bearing (see
    // `coordinate_resolver::resolve::flush_left_padding`), so the string can
    // draw flush-left at `elem.x` directly, exactly like `render_lyric` —
    // no renderer-side recentering needed. The augmentation dot(s), if any,
    // are appended directly onto the chord text itself so they fall out of
    // normal text flow rather than being drawn as separately-positioned
    // glyphs.
    let content = format!(
        "{s}{}",
        font_metrics::augmentation_dot_suffix(dots.dotted, dots.double_dotted)
    );

    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::ChordSymbol),
        kind: SvgKind::Text {
            content,
            font_size: *base_font_size,
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            font: FontFamily::Monospace,
            weight: FontWeight::Normal,
            italic: false,
        },
    }]
}

pub(super) fn render_horizontal_line(elem: &AbsoluteElement, width: &f32) -> Vec<SvgElement> {
    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::HorizontalLine),
        kind: SvgKind::Line {
            x2: elem.x + width,
            y2: elem.y,
            stroke_width: 0.5,
        },
    }]
}

pub(super) fn render_underline(elem: &AbsoluteElement, width: &f32) -> Vec<SvgElement> {
    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::Underline),
        kind: SvgKind::Line {
            x2: elem.x + width,
            y2: elem.y,
            stroke_width: 1.0,
        },
    }]
}

pub(super) fn render_tie_or_slur(
    elem: &AbsoluteElement,
    width: &f32,
    row_height: &f32,
) -> Vec<SvgElement> {
    let cx = elem.x + width / 2.0;
    let cy = elem.y - row_height * 0.3;
    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::TieOrSlur),
        kind: SvgKind::Path {
            control_x: cx,
            control_y: cy,
            end_x: elem.x + width,
            end_y: elem.y,
            stroke_width: 1.0,
        },
    }]
}

#[path = "glyph_renderers_tuplet_bracket.rs"]
mod tuplet_bracket;
pub(super) use tuplet_bracket::render_tuplet_bracket;

pub(super) fn render_bar_line(elem: &AbsoluteElement, height: &f32) -> Vec<SvgElement> {
    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::BarLine),
        kind: SvgKind::Line {
            x2: elem.x,
            y2: elem.y + height,
            stroke_width: 0.5,
        },
    }]
}

#[path = "glyph_renderers_lyric.rs"]
mod lyric;
pub(super) use lyric::{render_lyric, render_lyric_line};
