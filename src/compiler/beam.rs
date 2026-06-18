use crate::compiler::types::{ColumnElement, ElementContent};

// ── Per-part beam state ───────────────────────────────────────────────────────

pub(super) struct BeamEntry {
    pub(super) column: u32,
    pub(super) underline_count: u32,
    pub(super) duration: u32,
}

pub(super) fn flush_beam_buffer(buffer: &mut Vec<BeamEntry>, elements: &mut Vec<ColumnElement>) {
    if buffer.is_empty() {
        return;
    }
    let underlines = compute_underline_levels(buffer);
    elements.extend(underlines);
    buffer.clear();
}

fn compute_underline_levels(buffer: &[BeamEntry]) -> Vec<ColumnElement> {
    let (Some(first), Some(last)) = (buffer.first(), buffer.last()) else {
        return Vec::new();
    };
    let mut result = Vec::new();

    result.push(ColumnElement {
        column: first.column,
        content: ElementContent::Underline {
            from_column: first.column,
            to_column: last.column + last.duration,
            last_head_column: last.column,
            level: 0,
        },
    });

    let mut run_start: Option<u32> = None;
    let mut run_end: u32 = 0;
    let mut run_last_head: u32 = 0;
    for entry in buffer {
        if entry.underline_count >= 2 {
            if run_start.is_none() {
                run_start = Some(entry.column);
            }
            run_end = entry.column + entry.duration;
            run_last_head = entry.column;
        } else if let Some(start) = run_start.take() {
            result.push(ColumnElement {
                column: start,
                content: ElementContent::Underline {
                    from_column: start,
                    to_column: run_end,
                    last_head_column: run_last_head,
                    level: 1,
                },
            });
        }
    }
    if let Some(start) = run_start {
        result.push(ColumnElement {
            column: start,
            content: ElementContent::Underline {
                from_column: start,
                to_column: run_end,
                last_head_column: run_last_head,
                level: 1,
            },
        });
    }

    result
}
