use crate::layout::types::{
    GridContent, GridElement, HorizontalAlignment, Page, VerticalAlignment,
};

/// Must match PAGE_MARGIN in layout/mod.rs — padding applied on every edge.
const PAGE_MARGIN: f32 = 25.0;

pub fn render(pages: &[Page], row_height: u32, note_number_width: u32) -> Vec<String> {
    pages
        .iter()
        .map(|page| render_page(page, row_height, note_number_width))
        .collect()
}

struct PageRenderContext {
    row_height: f32,
    note_number_width: f32,
    base_font_size: f32,
    cjk_font_size: f32,
    page_width: f32,
    page_height: f32,
    usable_width: f32,
}

impl PageRenderContext {
    fn new(page: &Page, row_height: u32, note_number_width: u32) -> Self {
        let row_height = row_height as f32;
        let note_number_width = note_number_width as f32;
        let page_width = page.page_width_pt;
        Self {
            row_height,
            note_number_width,
            base_font_size: row_height * 0.6,
            cjk_font_size: row_height * 0.6 * 1.2,
            page_width,
            page_height: 842.0,
            usable_width: page_width - 2.0 * PAGE_MARGIN,
        }
    }
}

fn render_page(page: &Page, row_height: u32, note_number_width: u32) -> String {
    let ctx = PageRenderContext::new(page, row_height, note_number_width);
    let mut elements = String::new();
    render_page_header(page, &ctx, &mut elements);
    render_row_groups(page, &ctx, &mut elements);
    render_page_footer(page, &ctx, &mut elements);

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="210mm" height="297mm" viewBox="0 0 595 842">{elements}</svg>"#
    )
}

fn render_page_header(page: &Page, ctx: &PageRenderContext, elements: &mut String) {
    let title_y = PAGE_MARGIN + ctx.row_height * 0.75;
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="sans-serif">{}</text>"#,
        ctx.page_width / 2.0,
        title_y,
        ctx.row_height * 1.5,
        escape_xml(&page.header.title)
    ));

    let subtitle_author_y = PAGE_MARGIN + ctx.row_height * 2.25;
    if let Some(subtitle) = &page.header.subtitle {
        elements.push_str(&format!(
            r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="sans-serif">{}</text>"#,
            ctx.page_width / 2.0,
            subtitle_author_y,
            ctx.base_font_size,
            escape_xml(subtitle)
        ));
    }
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="end" dominant-baseline="middle" font-family="sans-serif">{}</text>"#,
        ctx.page_width - PAGE_MARGIN,
        subtitle_author_y,
        ctx.base_font_size,
        escape_xml(&page.header.author)
    ));
}

fn render_row_groups(page: &Page, ctx: &PageRenderContext, elements: &mut String) {
    for row_group in &page.row_groups {
        let column_width = ctx.usable_width / row_group.width_in_columns as f32;
        for element in row_group.elements.iter() {
            render_grid_element(element, column_width, ctx, elements);
        }
    }
}

fn render_page_footer(page: &Page, ctx: &PageRenderContext, elements: &mut String) {
    let footer_y = ctx.page_height - PAGE_MARGIN - ctx.row_height * 0.5;
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="sans-serif">{}/{}</text>"#,
        ctx.page_width / 2.0,
        footer_y,
        ctx.row_height * 0.75,
        page.footer.page,
        page.footer.total
    ));
}

#[derive(Copy, Clone)]
struct ElementCoords {
    x: f32,
    y: f32,
    base_x: f32,
    base_y: f32,
}

fn element_position(element: &GridElement, column_width: f32, row_height: f32) -> ElementCoords {
    let col = element.position.column as f32;
    let row = element.position.row as f32;
    let base_x = col * column_width + PAGE_MARGIN;
    let base_y = PAGE_MARGIN + row * row_height;

    let x = match element.horizontal_alignment {
        HorizontalAlignment::Left => base_x,
        HorizontalAlignment::Center => base_x + column_width / 2.0,
        HorizontalAlignment::Right => base_x + column_width,
    };
    let y = match element.vertical_alignment {
        VerticalAlignment::Top => base_y,
        VerticalAlignment::Center => base_y + row_height / 2.0,
        VerticalAlignment::Bottom => base_y + row_height,
    };

    ElementCoords {
        x,
        y,
        base_x,
        base_y,
    }
}

fn render_grid_element(
    element: &GridElement,
    column_width: f32,
    ctx: &PageRenderContext,
    elements: &mut String,
) {
    let coords = element_position(element, column_width, ctx.row_height);
    render_grid_content(&element.content, coords, column_width, ctx, elements);
}

fn render_grid_content(
    content: &GridContent,
    coords: ElementCoords,
    column_width: f32,
    ctx: &PageRenderContext,
    elements: &mut String,
) {
    let ElementCoords {
        x,
        y,
        base_x,
        base_y,
    } = coords;

    match content {
        GridContent::NoteHead {
            pitch,
            octave,
            dotted,
        } => render_note_head(pitch, *octave, *dotted, coords, column_width, ctx, elements),
        GridContent::Rest => render_rest(x, y, ctx, elements),
        GridContent::DurationUnderlines { levels } => {
            render_duration_underlines(levels, base_y, column_width, ctx, elements);
        }
        GridContent::LowerOctaveDots {
            count,
            underline_count,
        } => render_lower_octave_dots(*count, *underline_count, x, base_y, ctx, elements),
        GridContent::Lyric { text, is_cjk } => render_lyric(text, *is_cjk, x, y, ctx, elements),
        GridContent::TieOrSlurCurve {
            from_column,
            to_column,
        } => render_tie_or_slur_curve(
            *from_column,
            *to_column,
            y,
            base_y,
            column_width,
            ctx,
            elements,
        ),
        GridContent::Extension => render_extension(x, y, ctx, elements),
        GridContent::BarLine { height_in_rows } => {
            render_bar_line(x, base_y, *height_in_rows, ctx, elements);
        }
        GridContent::TimeSignatureLabel {
            numerator,
            denominator,
        } => render_time_signature_label(
            numerator,
            denominator,
            base_x,
            y,
            column_width,
            ctx,
            elements,
        ),
        GridContent::BpmLabel { bpm } => {
            render_bpm_label(*bpm, base_x, y, column_width, ctx, elements);
        }
        GridContent::PartLabel { text } => render_part_label(text, x, y, ctx, elements),
        GridContent::HorizontalBar {
            from_column,
            to_column,
        } => render_horizontal_bar(*from_column, *to_column, base_y, column_width, elements),
        GridContent::BarNumber { number } => render_bar_number(*number, x, y, ctx, elements),
        GridContent::SectionLabel { text } => render_section_label(text, x, y, ctx, elements),
        GridContent::ChordSymbol { text } => render_chord_symbol(text, x, y, ctx, elements),
    }
}

fn render_rest(x: f32, y: f32, ctx: &PageRenderContext, elements: &mut String) {
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="monospace">0</text>"#,
        x, y, ctx.base_font_size
    ));
}

fn render_lyric(
    text: &str,
    is_cjk: bool,
    x: f32,
    y: f32,
    ctx: &PageRenderContext,
    elements: &mut String,
) {
    let font_size = if is_cjk {
        ctx.cjk_font_size
    } else {
        ctx.base_font_size
    };
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="hanging" font-family="sans-serif">{}</text>"#,
        x, y, font_size, escape_xml(text)
    ));
}

fn render_extension(x: f32, y: f32, ctx: &PageRenderContext, elements: &mut String) {
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="monospace">-</text>"#,
        x, y, ctx.base_font_size
    ));
}

fn render_bar_line(
    x: f32,
    base_y: f32,
    height_in_rows: u32,
    ctx: &PageRenderContext,
    elements: &mut String,
) {
    let line_y2 = base_y + height_in_rows as f32 * ctx.row_height;
    elements.push_str(&format!(
        r#"<line x1="{x:.1}" y1="{base_y:.1}" x2="{x:.1}" y2="{line_y2:.1}" stroke="black" stroke-width="0.5"/>"#
    ));
}

fn render_part_label(text: &str, x: f32, y: f32, ctx: &PageRenderContext, elements: &mut String) {
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="start" dominant-baseline="middle" font-family="sans-serif">{}</text>"#,
        x, y, ctx.base_font_size * 0.8, escape_xml(text)
    ));
}

fn render_bar_number(number: u32, x: f32, y: f32, ctx: &PageRenderContext, elements: &mut String) {
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="start" dominant-baseline="ideographic" font-family="sans-serif">{}</text>"#,
        x, y, ctx.base_font_size * 0.6, number
    ));
}

fn render_section_label(
    text: &str,
    x: f32,
    y: f32,
    ctx: &PageRenderContext,
    elements: &mut String,
) {
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="start" dominant-baseline="ideographic" font-style="italic" font-weight="bold" font-family="sans-serif">{}</text>"#,
        x, y, ctx.base_font_size * 1.2, escape_xml(text)
    ));
}

fn render_chord_symbol(text: &str, x: f32, y: f32, ctx: &PageRenderContext, elements: &mut String) {
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="start" dominant-baseline="middle" font-family="monospace">{}</text>"#,
        x, y, ctx.base_font_size, escape_xml(text)
    ));
}

fn render_note_head(
    pitch: &crate::ast::parsed::JianPuPitch,
    octave: i8,
    dotted: bool,
    coords: ElementCoords,
    column_width: f32,
    ctx: &PageRenderContext,
    elements: &mut String,
) {
    let ElementCoords { x, y, base_y, .. } = coords;
    let digit = pitch_to_digit(pitch);
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="monospace">{}</text>"#,
        x, y, ctx.base_font_size, digit
    ));
    if dotted {
        let dot_radius = ctx.row_height * 0.06;
        let dot_x = x + column_width * 0.5;
        elements.push_str(&format!(
            r#"<circle cx="{dot_x:.1}" cy="{y:.1}" r="{dot_radius:.1}" fill="black"/>"#
        ));
    }
    let dot_radius = ctx.row_height * 0.08;
    let dot_spacing = dot_radius * 3.0;
    for i in 0..octave {
        let dot_y = base_y - dot_radius - (i as f32) * dot_spacing;
        elements.push_str(&format!(
            r#"<circle cx="{x:.1}" cy="{dot_y:.1}" r="{dot_radius:.1}" fill="black"/>"#
        ));
    }
}

fn render_duration_underlines(
    levels: &[crate::layout::types::UnderlineSpan],
    base_y: f32,
    column_width: f32,
    ctx: &PageRenderContext,
    elements: &mut String,
) {
    for (i, span) in levels.iter().enumerate() {
        let line_x1 = span.from_column as f32 * column_width + column_width * 0.1 + PAGE_MARGIN;
        let line_x2 = span.last_head_column as f32 * column_width
            + column_width * 0.5
            + ctx.note_number_width * 0.5
            + PAGE_MARGIN;
        let line_y = base_y + ctx.row_height * 0.1 + (i as f32) * (ctx.row_height * 0.15);
        elements.push_str(&format!(
            r#"<line x1="{line_x1:.1}" y1="{line_y:.1}" x2="{line_x2:.1}" y2="{line_y:.1}" stroke="black" stroke-width="1"/>"#
        ));
    }
}

fn render_lower_octave_dots(
    count: u32,
    underline_count: u8,
    x: f32,
    base_y: f32,
    ctx: &PageRenderContext,
    elements: &mut String,
) {
    let dot_radius = ctx.row_height * 0.08;
    for i in 0..count {
        let slot = underline_count as f32 + i as f32;
        let dot_y = base_y + ctx.row_height * 0.1 + slot * (ctx.row_height * 0.15);
        elements.push_str(&format!(
            r#"<circle cx="{x:.1}" cy="{dot_y:.1}" r="{dot_radius:.1}" fill="black"/>"#
        ));
    }
}

fn render_tie_or_slur_curve(
    from_column: u32,
    to_column: u32,
    y: f32,
    base_y: f32,
    column_width: f32,
    ctx: &PageRenderContext,
    elements: &mut String,
) {
    let x1 = (from_column as f32 + 0.5) * column_width + PAGE_MARGIN;
    let x2 = (to_column as f32 + 0.5) * column_width + PAGE_MARGIN;
    let cy = base_y - ctx.row_height * 0.3;
    elements.push_str(&format!(
        r#"<path d="M {:.1} {:.1} Q {:.1} {:.1} {:.1} {:.1}" fill="none" stroke="black" stroke-width="1"/>"#,
        x1, y, (x1 + x2) / 2.0, cy, x2, y
    ));
}

fn render_time_signature_label(
    numerator: &u8,
    denominator: &u8,
    base_x: f32,
    y: f32,
    column_width: f32,
    ctx: &PageRenderContext,
    elements: &mut String,
) {
    let slot_width = 2.0 * column_width;
    let center_x = base_x + slot_width / 2.0;
    let numerator_y = y - ctx.row_height * 0.25;
    let rule_y = y;
    let denominator_y = y + ctx.row_height * 0.25;
    let rule_x1 = base_x + slot_width * 0.2;
    let rule_x2 = base_x + slot_width * 0.8;
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="sans-serif">{}</text>"#,
        center_x, numerator_y, ctx.base_font_size, numerator
    ));
    elements.push_str(&format!(
        r#"<line x1="{rule_x1:.1}" y1="{rule_y:.1}" x2="{rule_x2:.1}" y2="{rule_y:.1}" stroke="black" stroke-width="1"/>"#
    ));
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="sans-serif">{}</text>"#,
        center_x, denominator_y, ctx.base_font_size, denominator
    ));
}

fn render_bpm_label(
    bpm: u32,
    base_x: f32,
    y: f32,
    column_width: f32,
    ctx: &PageRenderContext,
    elements: &mut String,
) {
    let slot_width = 2.0 * column_width;
    let center_x = base_x + slot_width / 2.0;
    elements.push_str(&format!(
        r#"<text x="{center_x:.1}" y="{y:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="sans-serif">♩={bpm}</text>"#,
        ctx.base_font_size
    ));
}

fn render_horizontal_bar(
    from_column: u32,
    to_column: u32,
    base_y: f32,
    column_width: f32,
    elements: &mut String,
) {
    let x1 = from_column as f32 * column_width + PAGE_MARGIN;
    let x2 = to_column as f32 * column_width + PAGE_MARGIN;
    elements.push_str(&format!(
        r#"<line x1="{x1:.1}" y1="{base_y:.1}" x2="{x2:.1}" y2="{base_y:.1}" stroke="black" stroke-width="0.35"/>"#
    ));
}

fn pitch_to_digit(pitch: &crate::ast::parsed::JianPuPitch) -> char {
    use crate::ast::parsed::JianPuPitch::*;
    match pitch {
        One => '1',
        Two => '2',
        Three => '3',
        Four => '4',
        Five => '5',
        Six => '6',
        Seven => '7',
    }
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests;
