use crate::ast::parsed::Offset;
use crate::compositor::types::{AbsoluteElement, DominantBaseline, TextAnchor, TextSpan};
use crate::renderer::new_types::{
    SvgElement, SvgKind, SvgVariant, Tag, TransparentRectRole, TspanData,
};

fn spans_to_tspans(spans: &[TextSpan]) -> Vec<TspanData> {
    spans
        .iter()
        .map(|s| TspanData {
            content: s.content.clone(),
            bold: s.bold,
            italic: s.italic,
            font_size: if (s.font_size - 12.0).abs() < 0.001 {
                None
            } else {
                Some(s.font_size)
            },
        })
        .collect()
}

pub(super) struct DirectiveLineArgs<'a> {
    pub bar_number: &'a Option<TextSpan>,
    pub label: &'a Option<String>,
    pub spans: &'a [TextSpan],
    pub spans_x_offset: f32,
    pub label_x_offset: f32,
    pub apply_row_offset: bool,
    pub directive_row_offset: Offset,
}

pub(super) fn render_directive_line(
    elem: &AbsoluteElement,
    args: &DirectiveLineArgs,
) -> Vec<SvgElement> {
    let (row_x, row_y) = if args.apply_row_offset {
        (
            elem.x + args.directive_row_offset.x as f32,
            elem.y + args.directive_row_offset.y as f32,
        )
    } else {
        (elem.x, elem.y)
    };

    let bar_number_element = args.bar_number.as_ref().map(|span| SvgElement {
        x: row_x,
        y: row_y,
        variant: Some(SvgVariant::DirectiveLine),
        kind: SvgKind::TextWithTspans {
            font_size: 12.0,
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            spans: spans_to_tspans(std::slice::from_ref(span)),
        },
    });

    let text_element = SvgElement {
        x: row_x + args.spans_x_offset,
        y: row_y,
        variant: Some(SvgVariant::DirectiveLine),
        kind: SvgKind::TextWithTspans {
            font_size: 12.0,
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            spans: spans_to_tspans(args.spans),
        },
    };

    match args.label {
        Some(label_str) => {
            let line_width = crate::font_metrics::directive_line_width(
                args.bar_number.as_ref(),
                Some(label_str),
                args.spans,
            );

            vec![render_section_label_group(
                elem,
                &SectionLabelGroupArgs {
                    label_str,
                    row_x,
                    row_y,
                    label_x: row_x + args.label_x_offset,
                    line_width,
                },
                SectionLabelSiblingElements {
                    bar_number_element,
                    text_element,
                },
            )]
        }
        None => bar_number_element
            .into_iter()
            .chain(std::iter::once(text_element))
            .collect(),
    }
}

/// The directive-line elements a section-label group wraps alongside the
/// label's own text/box, so they move together under `directive_row_offset`.
struct SectionLabelSiblingElements {
    bar_number_element: Option<SvgElement>,
    text_element: SvgElement,
}

/// Where a section label's own text/box and its full-line click target sit,
/// relative to the row.
struct SectionLabelGroupArgs<'a> {
    label_str: &'a str,
    row_x: f32,
    row_y: f32,
    label_x: f32,
    line_width: f32,
}

fn render_section_label_group(
    elem: &AbsoluteElement,
    args: &SectionLabelGroupArgs,
    siblings: SectionLabelSiblingElements,
) -> SvgElement {
    let bg_width = crate::font_metrics::section_label_box_width(args.label_str);
    let bg_height = crate::font_metrics::section_label_box_height();
    // Covers the whole directive line (bar number through trailing spans),
    // not just the label box, so the group has no unpainted gap for a click
    // to fall through — see `TransparentRectRole::SectionLabelClickTarget`.
    let click_target = SvgElement {
        x: args.row_x,
        y: args.row_y - bg_height / 2.0,
        variant: None,
        kind: SvgKind::TransparentRect {
            width: args.line_width,
            height: bg_height,
            role: TransparentRectRole::SectionLabelClickTarget,
        },
    };
    let label_element = SvgElement {
        x: args.label_x,
        y: args.row_y,
        variant: None,
        kind: SvgKind::TextWithTspans {
            font_size: crate::font_metrics::SECTION_LABEL_FONT_SIZE,
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            spans: vec![TspanData {
                content: args.label_str.to_string(),
                bold: true,
                italic: true,
                font_size: None,
            }],
        },
    };
    let mut children: Vec<SvgElement> = vec![click_target];
    children.extend(siblings.bar_number_element);
    children.push(SvgElement {
        x: args.label_x - crate::font_metrics::section_label_box_padding(),
        y: args.row_y - bg_height / 2.0,
        variant: None,
        kind: SvgKind::TransparentRect {
            width: bg_width,
            height: bg_height,
            role: TransparentRectRole::SectionLabelBackground,
        },
    });
    children.push(label_element);
    children.push(siblings.text_element);
    SvgElement {
        x: elem.x,
        y: elem.y,
        variant: None,
        kind: SvgKind::Group {
            tag: Some(Tag::SectionLabel {
                label: args.label_str.to_string(),
            }),
            children,
        },
    }
}
