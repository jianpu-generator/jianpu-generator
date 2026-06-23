type SourceLine = (String, usize);

pub(super) fn is_directive_line(line: &str) -> bool {
    let inner = if line.starts_with('(') && line.ends_with(')') {
        &line[1..line.len() - 1]
    } else {
        line
    };
    inner.split_whitespace().any(|t| {
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

/// Byte offset in the full source of the first line in each measure group.
///
/// When a group begins with a directive row, that line's offset is returned;
/// otherwise the first data line's offset is returned. Used to place editor
/// view zones above the full measure block (directives included).
pub fn view_zone_starts(content: &str, base_offset: usize) -> Vec<usize> {
    collect_groups(content)
        .into_iter()
        .filter_map(|group| {
            group
                .first()
                .map(|(_, line_offset)| base_offset + line_offset)
        })
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
