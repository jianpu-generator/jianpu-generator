use itertools::Itertools;

use crate::compositor::types::{
    AbsoluteElement, DominantBaseline, FontFamily, FontWeight, TextAnchor,
};
use crate::font_metrics::guitar_diagram::{
    base_fret_label, grid_height_pt, grid_width_pt, BASE_FRET_LABEL_FONT_SIZE_PT,
    BASE_FRET_LABEL_GAP_PT, DOT_RADIUS_PT, FRET_ROWS, FRET_SPACING_PT, GAP_BELOW_PT, STRING_COUNT,
    STRING_MARKER_BAND_PT, STRING_MARKER_RADIUS_PT, STRING_SPACING_PT,
};
use crate::pitch_description::GuitarVoicing;
use crate::renderer::new_types::{SvgElement, SvgKind, SvgVariant, Tag};

const GRID_STROKE_WIDTH: f32 = 0.4;
const NUT_STROKE_WIDTH: f32 = 1.4;
const MARKER_STROKE_WIDTH: f32 = 0.5;

/// Where the diagram's grid sits: `elem.x` is the diagram's left edge and
/// `elem.y` its bottom edge (see `AbsoluteContent::GuitarDiagram`).
struct Grid {
    left: f32,
    top: f32,
    bottom: f32,
}

impl Grid {
    fn string_x(&self, string_index: usize) -> f32 {
        self.left + STRING_SPACING_PT * string_index as f32
    }

    fn fret_line_y(&self, fret_row: u8) -> f32 {
        self.top + FRET_SPACING_PT * f32::from(fret_row)
    }

    /// Centre of the space between fret lines `relative_fret - 1` and
    /// `relative_fret`.
    fn fret_dot_y(&self, relative_fret: u8) -> f32 {
        self.top + FRET_SPACING_PT * (f32::from(relative_fret) - 0.5)
    }

    fn marker_y(&self) -> f32 {
        self.top - STRING_MARKER_BAND_PT * 0.5
    }
}

/// A chord-box diagram, like the pitch drawer's (`render_guitar_diagram_svg`)
/// but sized for the score: vertical lines are strings (low E on the left),
/// horizontal lines are frets, dots are fingered frets, `×`/`○` above the nut
/// mark muted/open strings, and a diagram that starts above the nut is
/// labelled with its base fret (`5fr`) on its right. Grouped under
/// `Tag::GuitarDiagram`, which carries the absolute frets for tests.
pub(in crate::renderer::new_renderer) fn render_guitar_diagram(
    elem: &AbsoluteElement,
    voicing: &GuitarVoicing,
) -> Vec<SvgElement> {
    let bottom = elem.y - GAP_BELOW_PT;
    let grid = Grid {
        left: elem.x + DOT_RADIUS_PT,
        top: bottom - grid_height_pt(),
        bottom,
    };
    let children = [
        render_grid(&grid),
        render_nut_or_base_fret_label(&grid, voicing),
        render_barres(&grid, voicing),
        render_string_markers(&grid, voicing),
    ]
    .into_iter()
    .flatten()
    .collect();
    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: None,
        kind: SvgKind::Group {
            children,
            tag: Some(Tag::GuitarDiagram {
                frets: voicing.frets_label(),
                base_fret: voicing.base_fret,
            }),
        },
    }]
}

fn line(x1: f32, y1: f32, x2: f32, y2: f32, stroke_width: f32) -> SvgElement {
    SvgElement {
        x: x1,
        y: y1,
        variant: Some(SvgVariant::GuitarDiagram),
        kind: SvgKind::Line {
            x2,
            y2,
            stroke_width,
        },
    }
}

fn render_grid(grid: &Grid) -> Vec<SvgElement> {
    let strings = (0..STRING_COUNT).map(|string_index| {
        let x = grid.string_x(string_index);
        line(x, grid.top, x, grid.bottom, GRID_STROKE_WIDTH)
    });
    let frets = (0..=FRET_ROWS).map(|fret_row| {
        let y = grid.fret_line_y(fret_row);
        line(
            grid.left,
            y,
            grid.left + grid_width_pt(),
            y,
            GRID_STROKE_WIDTH,
        )
    });
    strings.chain(frets).collect()
}

fn render_nut_or_base_fret_label(grid: &Grid, voicing: &GuitarVoicing) -> Vec<SvgElement> {
    match base_fret_label(voicing) {
        None => vec![line(
            grid.left,
            grid.top,
            grid.left + grid_width_pt(),
            grid.top,
            NUT_STROKE_WIDTH,
        )],
        Some(label) => vec![SvgElement {
            x: grid.left + grid_width_pt() + DOT_RADIUS_PT + BASE_FRET_LABEL_GAP_PT,
            y: grid.fret_dot_y(1),
            variant: Some(SvgVariant::GuitarDiagram),
            kind: SvgKind::Text {
                content: label,
                font_size: BASE_FRET_LABEL_FONT_SIZE_PT,
                anchor: TextAnchor::Start,
                baseline: DominantBaseline::Middle,
                font: FontFamily::SansSerif,
                weight: FontWeight::Normal,
                italic: false,
                underline: false,
            },
        }],
    }
}

/// A barre spans from the lowest to the highest string fretted at its row.
fn render_barres(grid: &Grid, voicing: &GuitarVoicing) -> Vec<SvgElement> {
    voicing
        .barres
        .iter()
        .filter_map(|&barre| {
            let (first, last) = voicing
                .frets
                .iter()
                .positions(|&fret| i16::from(fret) == i16::from(barre))
                .minmax()
                .into_option()?;
            Some(SvgElement {
                x: grid.string_x(first) - DOT_RADIUS_PT,
                y: grid.fret_dot_y(barre) - DOT_RADIUS_PT,
                variant: Some(SvgVariant::GuitarDiagram),
                kind: SvgKind::RoundedBar {
                    width: grid.string_x(last) - grid.string_x(first) + 2.0 * DOT_RADIUS_PT,
                    height: 2.0 * DOT_RADIUS_PT,
                },
            })
        })
        .collect()
}

fn render_string_markers(grid: &Grid, voicing: &GuitarVoicing) -> Vec<SvgElement> {
    let marker_y = grid.marker_y();
    let r = STRING_MARKER_RADIUS_PT;
    voicing
        .frets
        .iter()
        .enumerate()
        .flat_map(|(string_index, &fret)| {
            let x = grid.string_x(string_index);
            match u8::try_from(fret) {
                Err(_) => vec![
                    line(
                        x - r,
                        marker_y - r,
                        x + r,
                        marker_y + r,
                        MARKER_STROKE_WIDTH,
                    ),
                    line(
                        x - r,
                        marker_y + r,
                        x + r,
                        marker_y - r,
                        MARKER_STROKE_WIDTH,
                    ),
                ],
                Ok(0) => vec![SvgElement {
                    x,
                    y: marker_y,
                    variant: Some(SvgVariant::GuitarDiagram),
                    kind: SvgKind::RingCircle {
                        r,
                        stroke_width: MARKER_STROKE_WIDTH,
                    },
                }],
                Ok(relative_fret) => vec![SvgElement {
                    x,
                    y: grid.fret_dot_y(relative_fret),
                    variant: Some(SvgVariant::GuitarDiagram),
                    kind: SvgKind::Circle { r: DOT_RADIUS_PT },
                }],
            }
        })
        .collect()
}
