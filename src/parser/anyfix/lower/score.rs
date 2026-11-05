//! Lowering `sequence [ … ]`, `preset "Name" { … }` and `score """ … """`.

use anyfix_expr::{Node, NodeKind, StringLiteral};
use itertools::Itertools;

use super::super::source_map::MappedText;
use super::{as_bare_words, as_call, as_field, source_span, Lowerer, OriginLine};

/// The value ranges of a score string's lines, without the blank ones at
/// either end.
fn score_line_ranges(literal: &StringLiteral) -> Vec<std::ops::Range<usize>> {
    let mut start = 0;
    let lines = literal
        .value
        .split_inclusive('\n')
        .map(|line| {
            let line_start = start;
            start += line.len();
            let content = line.strip_suffix('\n').unwrap_or(line);
            let content = content.strip_suffix('\r').unwrap_or(content);
            line_start..line_start + content.len()
        })
        .collect_vec();
    let is_blank = |range: &std::ops::Range<usize>| {
        literal
            .value
            .get(range.clone())
            .is_none_or(|line| line.trim().is_empty())
    };
    let first = lines.iter().position(|range| !is_blank(range));
    let last = lines.iter().rposition(|range| !is_blank(range));
    match (first, last) {
        (Some(first), Some(last)) => {
            let mut kept = lines.get(first..=last).unwrap_or_default().to_vec();
            if let Some(final_line) = kept.last_mut() {
                let trimmed = literal
                    .value
                    .get(final_line.clone())
                    .map_or(0, |line| line.trim_end().len());
                final_line.end = final_line.start + trimmed;
            }
            kept
        }
        _ => Vec::new(),
    }
}

impl Lowerer<'_> {
    // ── sequence ────────────────────────────────────────────────────────

    pub(super) fn sequence(&mut self, node: &Node) -> Option<String> {
        let entries = self.elements(node, "[")?;
        Some(
            entries
                .iter()
                .filter_map(|entry| self.sequence_entry(entry))
                .join(", "),
        )
    }

    fn sequence_entry(&mut self, node: &Node) -> Option<String> {
        let filtered = as_call(node).and_then(|(call, name)| {
            match (name.as_str(), call.arguments.as_slice()) {
                ("_ WITHOUT _", [label, parts]) => Some((label, parts, true)),
                ("_ ONLY _", [label, parts]) => Some((label, parts, false)),
                _ => None,
            }
        });
        let Some((label, parts, omit)) = filtered else {
            return self.sequence_label(node);
        };
        let label = self.sequence_label(label)?;
        let parts = self.abbreviation_list(parts)?;
        let prefix = if omit { "-" } else { "" };
        Some(format!(
            "{label}({})",
            parts.iter().map(|part| format!("{prefix}{part}")).join(" ")
        ))
    }

    fn sequence_label(&mut self, node: &Node) -> Option<String> {
        let label = match &node.kind {
            NodeKind::String(literal) => Some(literal.value.clone()),
            _ => as_bare_words(node).map(|words| words.join(" ")),
        };
        match label {
            Some(label) if label.contains([',', '(', ')']) => {
                self.error(
                    node.span,
                    "a sequence label can't contain `,`, `(` or `)` yet",
                );
                None
            }
            Some(label) => Some(label),
            None => {
                self.error(
                    node.span,
                    "expected a label, optionally followed by `WITHOUT [parts]` or `ONLY [parts]`",
                );
                None
            }
        }
    }

    // ── presets ─────────────────────────────────────────────────────────

    /// The preset's `# preset "Name"` header line and its field lines.
    pub(super) fn preset(
        &mut self,
        name: &Node,
        fields: &Node,
    ) -> Option<(String, Vec<OriginLine>)> {
        let name = self.quoted_text(name, "a preset name")?;
        let fields = self.elements(fields, "{")?;
        let lines = fields
            .iter()
            .filter_map(|field| {
                let text = match as_field(field) {
                    Some((words, value))
                        if matches!(words.as_slice(), ["visible" | "soloed", "parts"]) =>
                    {
                        let parts = self.abbreviation_list(value)?;
                        format!("{} = {}", words.join("_"), parts.join(", "))
                    }
                    _ => self.metadata_field(field)?,
                };
                Some(OriginLine {
                    text,
                    origin: source_span(field.span),
                })
            })
            .collect();
        Some((format!("# preset {name}"), lines))
    }

    // ── score ───────────────────────────────────────────────────────────

    /// The score string, written as the `# score` body it is. Its lines are the
    /// string's own bytes, so they map exactly.
    pub(super) fn push_score(&mut self, output: &mut MappedText, body: &Node) -> Option<()> {
        let NodeKind::String(literal) = &body.kind else {
            self.error(
                body.span,
                "the score is one string of measures, like `score \"\"\" … \"\"\"`",
            );
            return None;
        };
        for line in score_line_ranges(literal) {
            output.push_string_value(literal, line);
            output.text.push('\n');
        }
        Some(())
    }
}
