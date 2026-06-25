type SourceLine = (String, usize);

pub(super) fn is_directive_line(line: &str) -> bool {
    line.split_whitespace().any(|t| {
        t.starts_with("bpm=")
            || t.starts_with("key=")
            || t.starts_with("time=")
            || t.starts_with("label=")
    })
}

/// Number of leading directive lines in a raw measure group (0 or 1).
pub fn directive_line_count(group: &[SourceLine]) -> usize {
    usize::from(
        group
            .first()
            .map(|(line, _)| is_directive_line(line))
            .unwrap_or(false),
    )
}

/// Line number bounds for a single measure group, relative to the full source.
pub struct MeasureGroupBounds {
    /// Byte offset of the first line in this group, in the full source.
    pub view_zone_start: usize,
    /// 1-indexed first line of this group (inclusive), in the full source.
    pub start_line: usize,
    /// 1-indexed last line of this group (inclusive), in the full source.
    pub end_line: usize,
}

/// Returns the line-number bounds and view-zone start for every measure group
/// in `content`.
///
/// `base_byte_offset` is the byte offset of `content` within the full source
/// (used to compute `view_zone_start`).  `base_line` is the 1-indexed line
/// number of the first line of `content` within the full source (used to
/// compute `start_line` and `end_line`).
pub fn collect_group_bounds(
    content: &str,
    base_byte_offset: usize,
    base_line: usize,
) -> Vec<MeasureGroupBounds> {
    struct GroupAccum {
        view_zone_start: usize,
        start_line: usize,
        end_line: usize,
    }

    let mut groups: Vec<MeasureGroupBounds> = Vec::new();
    let mut current: Option<GroupAccum> = None;
    let mut byte_offset: usize = 0;
    let mut line_number: usize = base_line;

    for line in content.lines() {
        let leading = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if let Some(accum) = current.take() {
                groups.push(MeasureGroupBounds {
                    view_zone_start: accum.view_zone_start,
                    start_line: accum.start_line,
                    end_line: accum.end_line,
                });
            }
        } else {
            match &mut current {
                None => {
                    current = Some(GroupAccum {
                        view_zone_start: base_byte_offset + byte_offset + leading,
                        start_line: line_number,
                        end_line: line_number,
                    });
                }
                Some(accum) => {
                    accum.end_line = line_number;
                }
            }
        }
        byte_offset += line.len() + 1; // +1 for '\n'
        line_number += 1;
    }
    if let Some(accum) = current {
        groups.push(MeasureGroupBounds {
            view_zone_start: accum.view_zone_start,
            start_line: accum.start_line,
            end_line: accum.end_line,
        });
    }

    groups
}

/// Byte offset in the full source of the first line in each measure group.
///
/// When a group begins with a directive row, that line's offset is returned;
/// otherwise the first data line's offset is returned. Used to place editor
/// view zones above the full measure block (directives included).
pub fn view_zone_starts(content: &str, base_offset: usize) -> Vec<usize> {
    collect_group_bounds(content, base_offset, 1)
        .into_iter()
        .map(|b| b.view_zone_start)
        .collect()
}

/// Returns groups of `(trimmed_line, byte_offset_within_content)` pairs.
pub fn collect_groups(content: &str) -> Vec<Vec<SourceLine>> {
    let mut groups: Vec<Vec<SourceLine>> = Vec::new();
    let mut current: Vec<SourceLine> = Vec::new();
    let mut byte_offset: usize = 0;

    for line in content.lines() {
        let leading = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !current.is_empty() {
                groups.push(std::mem::take(&mut current));
            }
        } else {
            current.push((trimmed.to_string(), byte_offset + leading));
        }
        byte_offset += line.len() + 1; // +1 for '\n'
    }
    if !current.is_empty() {
        groups.push(current);
    }

    groups
}
