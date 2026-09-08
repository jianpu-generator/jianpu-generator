use crate::ast::parsed::{Accidental, JianPuPitch};
use crate::compositor::types::AbsoluteElement;
use crate::compositor::types::{DominantBaseline, FontFamily, FontWeight, TextAnchor};
use crate::font_metrics;
use crate::renderer::new_renderer::GlyphStyle;
use crate::renderer::new_types::{SvgElement, SvgKind, SvgVariant};

pub(super) struct NoteRenderParams<'a> {
    pub(super) base_font_size: &'a f32,
    pub(super) note_number_width: &'a f32,
    pub(super) bold: bool,
    pub(super) italic: bool,
    pub(super) underline: bool,
    pub(super) font_family: FontFamily,
}

fn glyph_weight(bold: bool) -> FontWeight {
    if bold {
        FontWeight::Bold
    } else {
        FontWeight::Normal
    }
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

/// A filled circle — the shared primitive behind both a note's *octave* dots
/// (drawn above/below the digit, see `render_note_head`'s `octave` loop) and
/// a note/rest/chord/dash's *augmentation* dot(s) (see
/// `augmentation_dot_glyphs`). Drawn as a vector shape rather than a font
/// glyph (e.g. `·`) so it renders as a true circle regardless of
/// font/renderer, instead of whatever shape a given font happens to draw its
/// middle-dot character as.
pub(super) fn dot_glyph(x: f32, y: f32, radius: f32, variant: SvgVariant) -> SvgElement {
    SvgElement {
        x,
        y,
        variant: Some(variant),
        kind: SvgKind::Circle { r: radius },
    }
}

/// Parameters for [`augmentation_dot_glyphs`], bundled because that
/// function's own logic needs more of them than a plain argument list can
/// hold under this crate's `clippy::too_many_arguments` limit.
pub(super) struct AugmentationDotParams<'a> {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) base_content: &'a str,
    pub(super) font_size: f32,
    pub(super) family: FontFamily,
    pub(super) variant: SvgVariant,
}

/// A note/rest/chord/note-dash's augmentation dot(s) (`.`/`..`), drawn as
/// filled circles immediately after `base_content`'s own rendered width,
/// rather than appended as `·` character(s) onto the glyph's text run — same
/// motivation as `dot_glyph`: a real circle regardless of font/renderer,
/// instead of whatever shape a font draws its middle-dot character as. Each
/// dot is placed within exactly one `augmentation_dot_suffix` character's
/// worth of advance width, matching what `dot_extra_weight`
/// (`grid_layout::layout_spacing_weights`) already reserves for it in column
/// layout, so this needs no accompanying change there. `family`/`font_size`
/// must match `base_content`'s own rendered font/size, the same requirement
/// `dot_extra_weight` documents.
pub(super) fn augmentation_dot_glyphs(
    params: &AugmentationDotParams<'_>,
    dots: &DotState,
) -> Vec<SvgElement> {
    if !dots.dotted {
        return Vec::new();
    }
    let &AugmentationDotParams {
        x,
        y,
        base_content,
        font_size,
        family,
        variant,
    } = params;
    let base_width = font_metrics::text_width_for_family(family, base_content, font_size);
    let single_dot_width = font_metrics::text_width_for_family(family, "\u{b7}", font_size);
    let dot_radius = font_size * 0.1;
    let count = if dots.double_dotted { 2 } else { 1 };
    (0..count)
        .map(|i| {
            let dot_center_x = x + base_width + single_dot_width * (i as f32 + 0.5);
            dot_glyph(dot_center_x, y, dot_radius, variant)
        })
        .collect()
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
        bold,
        italic,
        underline,
        font_family,
    } = params;
    let mut results = Vec::new();

    let accidental_symbol = match accidental {
        Accidental::Sharp => "♯",
        Accidental::Flat => "♭",
        Accidental::Natural => "",
    };

    // The digit and its sharp/flat accidental (if any) draw as one flush-left
    // text run at `elem.x` (see `coordinate_resolver::resolve::flush_left_padding`,
    // which already corrects `elem.x` for this glyph's own left-side bearing)
    // — the accidental simply falls out of normal text flow immediately
    // after the digit, rather than being drawn as its own separately-positioned
    // glyph at a hand-computed offset. The augmentation dot(s), if any, are
    // drawn separately as circles (see `augmentation_dot_glyphs`) rather than
    // appended onto this text run.
    let content = format!("{}{}", pitch.to_digit(), accidental_symbol);

    results.push(SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::NoteHead),
        kind: SvgKind::Text {
            content: content.clone(),
            font_size: **base_font_size,
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            font: *font_family,
            weight: glyph_weight(*bold),
            italic: *italic,
            underline: *underline,
        },
    });

    results.extend(augmentation_dot_glyphs(
        &AugmentationDotParams {
            x: elem.x,
            y: elem.y,
            base_content: &content,
            font_size: **base_font_size,
            family: *font_family,
            variant: SvgVariant::NoteHead,
        },
        dots,
    ));

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
        results.push(dot_glyph(center, dot_y, dot_radius, SvgVariant::NoteHead));
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
                underline: false,
            },
        },
    ]
}

pub(super) fn render_percussion_hit(
    elem: &AbsoluteElement,
    base_font_size: &f32,
    style: GlyphStyle,
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
            font: style.font_family,
            weight: glyph_weight(style.bold),
            italic: style.italic,
            underline: style.underline,
        },
    }]
}

pub(super) fn render_chord_symbol(
    elem: &AbsoluteElement,
    s: &str,
    dots: &DotState,
    base_font_size: &f32,
    style: GlyphStyle,
) -> Vec<SvgElement> {
    // `elem.x` is already corrected for this chord's own root character's
    // left-side bearing (see
    // `coordinate_resolver::resolve::flush_left_padding`), so the string can
    // draw flush-left at `elem.x` directly, exactly like `render_lyric` —
    // no renderer-side recentering needed. The augmentation dot(s), if any,
    // are drawn separately as circles (see `augmentation_dot_glyphs`) rather
    // than appended onto the chord text itself.
    let mut results = vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::ChordSymbol),
        kind: SvgKind::Text {
            content: s.to_string(),
            font_size: *base_font_size,
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            font: style.font_family,
            weight: glyph_weight(style.bold),
            italic: style.italic,
            underline: style.underline,
        },
    }];

    results.extend(augmentation_dot_glyphs(
        &AugmentationDotParams {
            x: elem.x,
            y: elem.y,
            base_content: s,
            font_size: *base_font_size,
            family: style.font_family,
            variant: SvgVariant::ChordSymbol,
        },
        dots,
    ));

    results
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
