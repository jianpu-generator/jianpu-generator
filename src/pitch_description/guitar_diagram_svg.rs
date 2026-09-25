use itertools::Itertools;

use super::guitar_voicing::GuitarVoicing;

const STRING_COUNT: usize = 6;
const FRET_ROWS: u8 = 4;
const STRING_SPACING: f64 = 16.0;
const FRET_SPACING: f64 = 20.0;
// Wide enough for a two-digit base-fret label, e.g. "12fr".
const LEFT_MARGIN: f64 = 40.0;
const TOP_MARGIN: f64 = 24.0;
const RIGHT_MARGIN: f64 = 12.0;
const BOTTOM_MARGIN: f64 = 8.0;
const DOT_RADIUS: f64 = 6.0;
const OPEN_STRING_RADIUS: f64 = 4.0;
const BASE_FRET_LABEL_FONT_SIZE: f64 = 14.0;

/// Renders a chord-box diagram: vertical lines are strings (low E on the
/// left), horizontal lines are frets, dots are fingered frets, `×`/`○` above
/// the nut mark muted/open strings. Every stroke and fill is `currentColor`
/// so the host page's text color themes it.
///
/// The root `<svg>` carries `data-guitar-frets` (e.g. `"x 3 1 3 4 x"`, in
/// absolute frets) and `data-guitar-base-fret` for tests to read without
/// parsing geometry.
pub(crate) fn render_guitar_diagram_svg(voicing: &GuitarVoicing, chord_name: &str) -> String {
    let width = LEFT_MARGIN + STRING_SPACING * (STRING_COUNT - 1) as f64 + RIGHT_MARGIN;
    let height = TOP_MARGIN + FRET_SPACING * f64::from(FRET_ROWS) + BOTTOM_MARGIN;
    let body = [
        render_nut_or_base_fret_label(voicing.base_fret),
        render_grid(),
        render_barres(voicing),
        render_string_markers(voicing),
    ]
    .concat();
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}" role="img" aria-label="Guitar chord diagram for {chord_name}" data-guitar-frets="{frets}" data-guitar-base-fret="{base_fret}" fill="currentColor" stroke="currentColor">{body}</svg>"#,
        frets = absolute_frets_label(voicing),
        base_fret = voicing.base_fret,
    )
}

fn absolute_frets_label(voicing: &GuitarVoicing) -> String {
    voicing
        .frets
        .iter()
        .map(|&fret| match fret {
            -1 => "x".to_string(),
            0 => "0".to_string(),
            relative => (i16::from(relative) + i16::from(voicing.base_fret) - 1).to_string(),
        })
        .join(" ")
}

fn string_x(string_index: usize) -> f64 {
    LEFT_MARGIN + STRING_SPACING * string_index as f64
}

fn fret_line_y(fret_row: u8) -> f64 {
    TOP_MARGIN + FRET_SPACING * f64::from(fret_row)
}

/// Centre of the space between fret lines `relative_fret - 1` and `relative_fret`.
fn fret_dot_y(relative_fret: u8) -> f64 {
    TOP_MARGIN + FRET_SPACING * (f64::from(relative_fret) - 0.5)
}

fn render_nut_or_base_fret_label(base_fret: u8) -> String {
    if base_fret <= 1 {
        format!(
            r#"<line x1="{x1}" y1="{y}" x2="{x2}" y2="{y}" stroke-width="4"/>"#,
            x1 = string_x(0),
            x2 = string_x(STRING_COUNT - 1),
            y = TOP_MARGIN,
        )
    } else {
        format!(
            r#"<text x="{x}" y="{y}" font-size="{BASE_FRET_LABEL_FONT_SIZE}" text-anchor="end" dominant-baseline="middle" stroke="none">{base_fret}fr</text>"#,
            x = string_x(0) - DOT_RADIUS - 2.0,
            y = fret_dot_y(1),
        )
    }
}

fn render_grid() -> String {
    let strings = (0..STRING_COUNT).map(|string_index| {
        format!(
            r#"<line x1="{x}" y1="{y1}" x2="{x}" y2="{y2}" stroke-width="1"/>"#,
            x = string_x(string_index),
            y1 = fret_line_y(0),
            y2 = fret_line_y(FRET_ROWS),
        )
    });
    let frets = (0..=FRET_ROWS).map(|fret_row| {
        format!(
            r#"<line x1="{x1}" y1="{y}" x2="{x2}" y2="{y}" stroke-width="1"/>"#,
            x1 = string_x(0),
            x2 = string_x(STRING_COUNT - 1),
            y = fret_line_y(fret_row),
        )
    });
    strings.chain(frets).collect()
}

/// A barre spans from the lowest to the highest string fretted at its row.
fn render_barres(voicing: &GuitarVoicing) -> String {
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
            Some(format!(
                r#"<rect x="{x}" y="{y}" width="{width}" height="{height}" rx="{DOT_RADIUS}" stroke="none"/>"#,
                x = string_x(first) - DOT_RADIUS,
                y = fret_dot_y(barre) - DOT_RADIUS,
                width = string_x(last) - string_x(first) + 2.0 * DOT_RADIUS,
                height = 2.0 * DOT_RADIUS,
            ))
        })
        .collect()
}

fn render_string_markers(voicing: &GuitarVoicing) -> String {
    let marker_y = TOP_MARGIN - 10.0;
    voicing
        .frets
        .iter()
        .enumerate()
        .map(|(string_index, &fret)| {
            let x = string_x(string_index);
            match u8::try_from(fret) {
                Err(_) => format!(
                    r#"<text x="{x}" y="{marker_y}" font-size="12" text-anchor="middle" dominant-baseline="middle" stroke="none">×</text>"#
                ),
                Ok(0) => format!(
                    r#"<circle cx="{x}" cy="{marker_y}" r="{OPEN_STRING_RADIUS}" fill="none" stroke-width="1"/>"#
                ),
                Ok(relative_fret) => format!(
                    r#"<circle cx="{x}" cy="{y}" r="{DOT_RADIUS}" stroke="none"/>"#,
                    y = fret_dot_y(relative_fret),
                ),
            }
        })
        .collect()
}
