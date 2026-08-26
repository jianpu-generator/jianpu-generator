use crate::grid_layout::types::{GridContent, PostArcGridContent};

/// Converts a resolved `GridContent` into the narrower `PostArcGridContent`
/// consumed by `content_conversion::grid_to_absolute`. Returns `None` for the
/// span-marking variants (`TieOrSlur*`, `TupletBracket`), which are resolved
/// directly by `resolve_span_marking` instead.
pub(super) fn to_post_arc_content(content: &GridContent) -> Option<PostArcGridContent> {
    match content {
        GridContent::TieOrSlur { .. }
        | GridContent::TieOrSlurTail { .. }
        | GridContent::TieOrSlurHead { .. }
        | GridContent::TupletBracket { .. } => None,
        GridContent::NoteHead {
            pitch,
            accidental,
            octave,
            dotted,
            double_dotted,
        } => Some(PostArcGridContent::NoteHead {
            pitch: pitch.clone(),
            accidental: accidental.clone(),
            octave: *octave,
            dotted: *dotted,
            double_dotted: *double_dotted,
        }),
        GridContent::Rest {
            dotted,
            double_dotted,
            implicit_fill,
        } => Some(PostArcGridContent::Rest {
            dotted: *dotted,
            double_dotted: *double_dotted,
            implicit_fill: *implicit_fill,
        }),
        GridContent::MultiMeasureRest { count } => {
            Some(PostArcGridContent::MultiMeasureRest { count: *count })
        }
        GridContent::NoteDash {
            dotted,
            double_dotted,
        } => Some(PostArcGridContent::NoteDash {
            dotted: *dotted,
            double_dotted: *double_dotted,
        }),
        GridContent::OctaveDot => Some(PostArcGridContent::OctaveDot),
        GridContent::ChordSymbol {
            text,
            dotted,
            double_dotted,
        } => Some(PostArcGridContent::ChordSymbol {
            text: text.clone(),
            dotted: *dotted,
            double_dotted: *double_dotted,
        }),
        GridContent::PercussionHit => Some(PostArcGridContent::PercussionHit),
        content => to_post_arc_text_content(content),
    }
}

/// The text/directive/label half of [`to_post_arc_content`]'s dispatch, split
/// out to stay under the file's line-count cap per function.
fn to_post_arc_text_content(content: &GridContent) -> Option<PostArcGridContent> {
    match content {
        GridContent::Underline { level } => Some(PostArcGridContent::Underline { level: *level }),
        GridContent::BarLine { height_pt } => Some(PostArcGridContent::BarLine {
            height_pt: *height_pt,
        }),
        GridContent::HorizontalLine => Some(PostArcGridContent::HorizontalLine),
        GridContent::RowLabel(s) => Some(PostArcGridContent::RowLabel(s.clone())),
        GridContent::LyricSyllable {
            text,
            source_part_index,
            note_id,
            verse,
        } => Some(PostArcGridContent::LyricSyllable {
            text: text.clone(),
            source_part_index: *source_part_index,
            note_id: *note_id,
            verse: *verse,
        }),
        GridContent::LyricLine(s) => Some(PostArcGridContent::LyricLine(s.clone())),
        GridContent::DirectiveLine {
            label,
            bar_number,
            key,
            bpm,
            time_signature,
        } => Some(PostArcGridContent::DirectiveLine {
            label: label.clone(),
            bar_number: *bar_number,
            key: key.clone(),
            bpm: *bpm,
            time_signature: *time_signature,
        }),
        GridContent::Text {
            content,
            font_size,
            bold,
            italic,
        } => Some(PostArcGridContent::Text {
            content: content.clone(),
            font_size: *font_size,
            bold: *bold,
            italic: *italic,
        }),
        GridContent::SequenceLine { entries, font_size } => {
            Some(PostArcGridContent::SequenceLine {
                entries: entries.clone(),
                font_size: *font_size,
            })
        }
        _ => None,
    }
}
