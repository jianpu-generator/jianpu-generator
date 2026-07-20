use crate::ast::parsed::PartDecl;
use crate::error::{RecoverableError, Span};

/// One `# groups` declaration: `<display-name> [<abbreviation>] = <members>`.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupDef {
    pub display_name: String,
    pub abbreviation: String,
    /// Byte span of the abbreviation token on its declaration line, used by
    /// rename-symbol to locate the declaration site.
    pub abbreviation_span: Span,
    /// Raw member abbreviations (parts or earlier groups).
    pub members: Vec<String>,
    /// Byte span of each entry in `members`, parallel to it, used by
    /// rename-symbol to locate these reference sites.
    pub member_spans: Vec<Span>,
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
        let trimmed_start = line_start + (line.len() - line.trim_start().len());
        match parse_group_line(trimmed, trimmed_start, line_span) {
            Ok(group_def) => groups.push(group_def),
            Err(error) => errors.push(error),
        }
    }

    (Some(GroupSection { groups }), errors)
}

fn parse_group_line(
    line: &str,
    line_start: usize,
    span: Span,
) -> Result<GroupDef, RecoverableError> {
    let Some((lhs, rhs)) = line.split_once('=') else {
        return Err(RecoverableError::general(
            span,
            format!("expected 'Name [Abbrev] = members', got: {line}"),
        ));
    };

    let (display_name, abbreviation, abbreviation_span) = parse_lhs(lhs, line_start, span)?;

    let rhs_start = line_start + lhs.len() + 1;
    let mut members = Vec::new();
    let mut member_spans = Vec::new();
    for (member, offset) in split_whitespace_with_offsets(rhs) {
        members.push(member.to_string());
        let member_start = rhs_start + offset;
        member_spans.push(Span::new(member_start, member_start + member.len()));
    }
    if members.is_empty() {
        return Err(RecoverableError::general(
            span,
            "group must list at least one member",
        ));
    }

    Ok(GroupDef {
        display_name,
        abbreviation,
        abbreviation_span,
        members,
        member_spans,
        span,
    })
}

/// Yields `(word, byte_offset_in_s)` pairs for each whitespace-separated token in `s`.
fn split_whitespace_with_offsets(s: &str) -> impl Iterator<Item = (&str, usize)> {
    let mut cursor = 0;
    s.split_whitespace().map(move |token| {
        let offset = cursor + s[cursor..].find(token).unwrap_or(0);
        cursor = offset + token.len();
        (token, offset)
    })
}

fn parse_lhs(
    raw_lhs: &str,
    line_start: usize,
    span: Span,
) -> Result<(String, String, Span), RecoverableError> {
    let lhs = raw_lhs.trim();
    if lhs.is_empty() {
        return Err(RecoverableError::general(
            span,
            "group name cannot be empty",
        ));
    }
    let lhs_start = line_start + (raw_lhs.len() - raw_lhs.trim_start().len());

    let Some(bracket_start) = lhs.find('[') else {
        let name_span = Span::new(lhs_start, lhs_start + lhs.len());
        return Ok((lhs.to_string(), lhs.to_string(), name_span));
    };

    let display_name = lhs[..bracket_start].trim();
    let bracketed = &lhs[bracket_start..];
    let bracketed_trimmed = bracketed.trim();
    let Some(abbreviation) = bracketed_trimmed
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

    // Locate the abbreviation's own byte range within `lhs[bracket_start..]`,
    // accounting for the `[` and any inner whitespace before it.
    let abbrev_offset_in_bracketed = bracketed.find(abbreviation).unwrap_or(0);
    let abbreviation_start = lhs_start + bracket_start + abbrev_offset_in_bracketed;
    let abbreviation_span = Span::new(abbreviation_start, abbreviation_start + abbreviation.len());

    Ok((
        display_name.to_string(),
        abbreviation.to_string(),
        abbreviation_span,
    ))
}

/// Resolves a group's members to the set of concrete part abbreviations it ultimately
/// contains, expanding any member that names another group (transitively). A `visited`
/// guard against self-referential group cycles skips an already-visited group abbreviation.
pub fn resolve_group_members<'a>(
    group: &'a GroupDef,
    groups: &'a [GroupDef],
    visited: &mut Vec<&'a str>,
) -> Vec<&'a str> {
    if visited.contains(&group.abbreviation.as_str()) {
        return Vec::new();
    }
    visited.push(&group.abbreviation);
    group
        .members
        .iter()
        .flat_map(
            |member| match groups.iter().find(|g| &g.abbreviation == member) {
                Some(nested) => resolve_group_members(nested, groups, visited),
                None => vec![member.as_str()],
            },
        )
        .collect()
}

/// A group whose members have been resolved (transitively) to concrete part abbreviations,
/// all sharing the same part kind, and whose abbreviation does not collide with a part's.
/// Only groups satisfying these constraints may be used as a `[GroupAbbrev]` broadcast key
/// in the `# score` section.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedGroup {
    pub abbreviation: String,
    /// Concrete part abbreviations this group broadcasts to, in declaration order.
    pub members: Vec<String>,
}

/// Resolves and validates every group in `group_section` against the declared parts,
/// returning the groups usable for score broadcasting alongside any recoverable errors
/// (an invalid group is dropped from the returned list but still contributes an error).
pub fn resolve_and_validate_groups(
    group_section: &GroupSection,
    declarations: &[PartDecl],
) -> (Vec<ResolvedGroup>, Vec<RecoverableError>) {
    let mut errors = Vec::new();
    let mut resolved = Vec::new();

    for group in &group_section.groups {
        if declarations
            .iter()
            .any(|d| d.abbreviation == group.abbreviation)
        {
            errors.push(RecoverableError::groups_abbreviation_collides_with_part(
                group.span,
                &group.abbreviation,
            ));
            continue;
        }

        let members = resolve_group_members(group, &group_section.groups, &mut Vec::new());

        let Some(kinds) = members
            .iter()
            .map(|member| {
                declarations
                    .iter()
                    .find(|d| &d.abbreviation == member)
                    .map(|d| d.kind)
            })
            .collect::<Option<Vec<_>>>()
        else {
            let unknown = members
                .iter()
                .find(|member| !declarations.iter().any(|d| &d.abbreviation == *member))
                .copied()
                .unwrap_or_default();
            errors.push(RecoverableError::groups_unknown_member(
                group.span,
                &group.abbreviation,
                unknown,
            ));
            continue;
        };

        if kinds.windows(2).any(|w| w.first() != w.get(1)) {
            errors.push(RecoverableError::groups_member_kind_mismatch(
                group.span,
                &group.abbreviation,
            ));
            continue;
        }

        resolved.push(ResolvedGroup {
            abbreviation: group.abbreviation.clone(),
            members: members.into_iter().map(str::to_string).collect(),
        });
    }

    (resolved, errors)
}

#[cfg(test)]
mod tests;
