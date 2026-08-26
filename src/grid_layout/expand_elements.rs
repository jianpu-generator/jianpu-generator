use crate::compiler::types::{ElementContent, MeasureRow, MULTI_MEASURE_REST_WIDTH};
use crate::grid_layout::layout::MUSIC_START_COL;
use crate::grid_layout::types::{GridContent, GridElement, GridRow, HAlign, VAlign};

pub(crate) fn grid_el(
    column: u32,
    content: GridContent,
    halign: HAlign,
    valign: VAlign,
) -> GridElement {
    GridElement {
        column,
        column_span: 1,
        halign,
        valign,
        content,
    }
}

pub(crate) fn push_head(
    sub_rows: &mut [GridRow],
    head_sub: usize,
    column: u32,
    content: GridContent,
) {
    if let Some(row) = sub_rows.get_mut(head_sub) {
        row.elements
            .push(grid_el(column, content, HAlign::Center, VAlign::Center));
    }
}

pub(crate) struct MeasureRenderParams {
    pub(crate) head_sub: usize,
    pub(crate) sub_count: usize,
    pub(crate) bar_height: f32,
    pub(crate) part_idx: usize,
    /// True when this measure is the last one in its system, so its closing
    /// `BarLine` should sit flush against the right edge of the column it
    /// occupies (`HAlign::End`) rather than centered within it — the same
    /// density-drift issue the leading barline has, mirrored at the right
    /// margin.
    pub(crate) is_last_block: bool,
}

/// The collapsed multi-measure-rest glyph gets a fixed wide `column_span`
/// (unlike `push_head`, which always spans a single column), so it's built
/// as its own `GridElement` rather than routed through `push_head`.
fn push_multi_measure_rest(sub_rows: &mut [GridRow], head_sub: usize, column: u32, count: u32) {
    if let Some(row) = sub_rows.get_mut(head_sub) {
        row.elements.push(GridElement {
            column,
            column_span: MULTI_MEASURE_REST_WIDTH,
            halign: HAlign::Center,
            valign: VAlign::Center,
            content: GridContent::MultiMeasureRest { count },
        });
    }
}

fn push_bar_line(sub_rows: &mut [GridRow], column: u32, bar_height: f32, halign: HAlign) {
    if let Some(row) = sub_rows.get_mut(0) {
        row.elements.push(grid_el(
            column,
            GridContent::BarLine {
                height_pt: bar_height,
            },
            halign,
            VAlign::Top,
        ));
    }
}

/// True for a `Rest` synthesized to fill a part not mentioned in this
/// measure (see `ElementContent::Rest::implicit_fill`) — the elements
/// [`expand_measure_elements`] consolidates into one centered whole-rest
/// glyph rather than laying out beat by beat.
fn is_implicit_fill_rest(content: &ElementContent) -> bool {
    matches!(
        content,
        ElementContent::Rest {
            implicit_fill: true,
            ..
        }
    )
}

/// A run of `implicit_fill` rests (see `is_implicit_fill_rest`) stands in for
/// a whole measure a part wasn't written for — normally several one-beat
/// `Rest` events (`0 0 0 0`), one per column, since the composer never wrote
/// a real whole rest to collapse them at the source level. Rendered
/// side-by-side they'd read as separate written rests; consolidated into a
/// single wide `GridElement` spanning from the run's first column to
/// whatever follows it (a bar line), they instead read as one glyph
/// centered between the measure's bar lines (see `resolve_implicit_fill_rest`
/// in `coordinate_resolver::resolve`), matching the conventional Western
/// whole-rest engraving.
fn push_implicit_fill_rest(
    sub_rows: &mut [GridRow],
    head_sub: usize,
    column: u32,
    column_span: u32,
    dotted: bool,
    double_dotted: bool,
) {
    if let Some(row) = sub_rows.get_mut(head_sub) {
        row.elements.push(GridElement {
            column,
            column_span,
            halign: HAlign::Center,
            valign: VAlign::Center,
            content: GridContent::Rest {
                dotted,
                double_dotted,
                implicit_fill: true,
            },
        });
    }
}

pub(crate) fn expand_measure_elements(
    row: &MeasureRow,
    measure_col_offset: u32,
    params: &MeasureRenderParams,
    sub_rows: &mut [GridRow],
) {
    let head_sub = params.head_sub;
    let sub_count = params.sub_count;
    let mut elements = row.elements.iter().peekable();
    while let Some(el) = elements.next() {
        // A run's first `Rest` is destructured here (rather than reusing
        // `is_implicit_fill_rest`, which only tests the content) so its own
        // `dotted`/`double_dotted` carry through without a redundant,
        // never-taken fallback arm for a shape `is_implicit_fill_rest`
        // already ruled out.
        if let ElementContent::Rest {
            dotted,
            double_dotted,
            implicit_fill: true,
        } = &el.content
        {
            let start_column = el.column;
            while elements
                .next_if(|next| is_implicit_fill_rest(&next.content))
                .is_some()
            {}
            let column_span = elements
                .peek()
                .map_or(1, |next| next.column - start_column)
                .max(1);
            push_implicit_fill_rest(
                sub_rows,
                head_sub,
                MUSIC_START_COL + measure_col_offset + start_column,
                column_span,
                *dotted,
                *double_dotted,
            );
            continue;
        }
        let grid_col = MUSIC_START_COL + measure_col_offset + el.column;
        match &el.content {
            ElementContent::Underline {
                from_column,
                last_head_column,
                level,
                ..
            } => {
                let span = last_head_column.saturating_sub(*from_column) + 1;
                let ul_sub = (sub_count - 2) + *level as usize;
                if let Some(row) = sub_rows.get_mut(ul_sub) {
                    row.elements.push(GridElement {
                        column: MUSIC_START_COL + measure_col_offset + from_column,
                        column_span: span,
                        halign: HAlign::Start,
                        valign: VAlign::Center,
                        content: GridContent::Underline { level: *level },
                    });
                }
            }
            ElementContent::BarLine => {
                if params.part_idx == 0 {
                    let halign = if params.is_last_block {
                        HAlign::End
                    } else {
                        HAlign::Center
                    };
                    push_bar_line(sub_rows, grid_col, params.bar_height, halign);
                }
            }
            ElementContent::Lyric { .. } | ElementContent::LyricLine { .. } => {} // handled in lyric-row branch above
            content => push_note_element(sub_rows, head_sub, grid_col, content),
        }
    }
}

/// The note/rest/chord-symbol half of [`expand_measure_elements`]'s dispatch,
/// split out to stay under the file's line-count cap per function.
fn push_note_element(
    sub_rows: &mut [GridRow],
    head_sub: usize,
    grid_col: u32,
    content: &ElementContent,
) {
    match content {
        ElementContent::NoteHead {
            pitch,
            accidental,
            octave,
            dotted,
            double_dotted,
        } => push_head(
            sub_rows,
            head_sub,
            grid_col,
            GridContent::NoteHead {
                pitch: pitch.clone(),
                accidental: accidental.clone(),
                octave: *octave,
                dotted: *dotted,
                double_dotted: *double_dotted,
            },
        ),
        ElementContent::Rest {
            dotted,
            double_dotted,
            implicit_fill,
        } => push_head(
            sub_rows,
            head_sub,
            grid_col,
            GridContent::Rest {
                dotted: *dotted,
                double_dotted: *double_dotted,
                implicit_fill: *implicit_fill,
            },
        ),
        ElementContent::MultiMeasureRest { count } => {
            push_multi_measure_rest(sub_rows, head_sub, grid_col, *count as u32);
        }
        ElementContent::NoteDash {
            dotted,
            double_dotted,
        } => push_head(
            sub_rows,
            head_sub,
            grid_col,
            GridContent::NoteDash {
                dotted: *dotted,
                double_dotted: *double_dotted,
            },
        ),
        ElementContent::PercussionHit => {
            push_head(sub_rows, head_sub, grid_col, GridContent::PercussionHit);
        }
        ElementContent::ChordSymbol {
            text,
            dotted,
            double_dotted,
        } => push_head(
            sub_rows,
            head_sub,
            grid_col,
            GridContent::ChordSymbol {
                text: text.clone(),
                dotted: *dotted,
                double_dotted: *double_dotted,
            },
        ),
        // Handled directly in `expand_measure_elements` before dispatching here.
        ElementContent::Underline { .. }
        | ElementContent::BarLine
        | ElementContent::Lyric { .. }
        | ElementContent::LyricLine { .. } => {}
    }
}
