use crate::error::{RecoverableError, Span};

/// One `# groups` declaration: `<display-name> [<abbreviation>] = <members>`.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupDef {
    pub display_name: String,
    pub abbreviation: String,
    /// Raw member abbreviations (parts or earlier groups).
    pub members: Vec<String>,
    pub span: Span,
}

/// The parsed `# groups` section: an ordered list of group declarations.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupSection {
    pub groups: Vec<GroupDef>,
}

/// Parses a `# groups` section's raw content into a list of group declarations.
///
/// Returns `None` when the section is absent or blank.
pub fn parse_group(content: &str, offset: usize) -> (Option<GroupSection>, Vec<RecoverableError>) {
    if content.trim().is_empty() {
        return (None, Vec::new());
    }

    let mut errors = Vec::new();
    let mut groups = Vec::new();
    let mut byte_offset = offset;

    for line in content.lines() {
        let trimmed = line.trim();
        let line_start = byte_offset;
        byte_offset += line.len() + 1;
        if trimmed.is_empty() {
            continue;
        }
        let line_span = Span::new(line_start, line_start + line.len());
        match parse_group_line(trimmed, line_span) {
            Ok(group_def) => groups.push(group_def),
            Err(error) => errors.push(error),
        }
    }

    (Some(GroupSection { groups }), errors)
}

fn parse_group_line(line: &str, span: Span) -> Result<GroupDef, RecoverableError> {
    let Some((lhs, rhs)) = line.split_once('=') else {
        return Err(RecoverableError::general(
            span,
            format!("expected 'Name [Abbrev] = members', got: {line}"),
        ));
    };

    let (display_name, abbreviation) = parse_lhs(lhs.trim(), span)?;

    let members: Vec<String> = rhs.split_whitespace().map(str::to_string).collect();
    if members.is_empty() {
        return Err(RecoverableError::general(
            span,
            "group must list at least one member",
        ));
    }

    Ok(GroupDef {
        display_name,
        abbreviation,
        members,
        span,
    })
}

fn parse_lhs(lhs: &str, span: Span) -> Result<(String, String), RecoverableError> {
    if lhs.is_empty() {
        return Err(RecoverableError::general(
            span,
            "group name cannot be empty",
        ));
    }

    let Some(bracket_start) = lhs.find('[') else {
        return Ok((lhs.to_string(), lhs.to_string()));
    };

    let display_name = lhs[..bracket_start].trim();
    let bracketed = lhs[bracket_start..].trim();
    let Some(abbreviation) = bracketed
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    else {
        return Err(RecoverableError::general(
            span,
            format!("expected 'Name [Abbrev]', got: {lhs}"),
        ));
    };
    let abbreviation = abbreviation.trim();

    if display_name.is_empty() {
        return Err(RecoverableError::general(
            span,
            "group display name cannot be empty",
        ));
    }
    if abbreviation.is_empty() {
        return Err(RecoverableError::general(
            span,
            "group abbreviation cannot be empty",
        ));
    }

    Ok((display_name.to_string(), abbreviation.to_string()))
}

#[cfg(test)]
mod tests;
