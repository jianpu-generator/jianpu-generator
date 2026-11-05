//! Lowering `sequence [ … ]`, `preset "Name" { … }` and `score [ … ]`.

use anyfix_expr::{Node, NodeKind};
use itertools::Itertools;

use super::{as_bare_words, as_call, as_field, Lowerer};

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

    pub(super) fn preset(&mut self, name: &Node, fields: &Node) -> Option<String> {
        let name = self.quoted_text(name, "a preset name")?;
        let fields = self.elements(fields, "{")?;
        let lines: String = fields
            .iter()
            .filter_map(|field| match as_field(field) {
                Some((words, value))
                    if matches!(words.as_slice(), ["visible" | "soloed", "parts"]) =>
                {
                    let parts = self.abbreviation_list(value)?;
                    Some(format!("{} = {}", words.join("_"), parts.join(", ")))
                }
                _ => self.metadata_field(field),
            })
            .map(|line| format!("{line}\n"))
            .collect();
        Some(format!("# preset {name}\n{lines}"))
    }

    // ── score ───────────────────────────────────────────────────────────

    /// Score elements as measure groups: a `{ directives }` element becomes
    /// the directive line of the first measure in the string after it. With
    /// no string right after it, it is a measure of its own, as a directive
    /// line alone is in the current syntax.
    pub(super) fn score(&mut self, node: &Node) -> Option<String> {
        let elements = self.elements(node, "[")?;
        let mut chunks = Vec::new();
        let mut pending: Option<String> = None;
        for element in elements {
            match &element.kind {
                NodeKind::Braces(directives) => {
                    let directive_line = self.directive_line(directives);
                    chunks.extend(pending.replace(directive_line));
                }
                NodeKind::String(literal) => {
                    let measures = literal
                        .value
                        .lines()
                        .skip_while(|line| line.trim().is_empty())
                        .join("\n");
                    chunks.push(match pending.take() {
                        Some(directives) => format!("{directives}\n{measures}"),
                        None => measures,
                    });
                }
                _ => self.error(
                    element.span,
                    "expected `{ directives }` or a string of measures",
                ),
            }
        }
        chunks.extend(pending);
        Some(
            chunks
                .iter()
                .map(|chunk| format!("{}\n", chunk.trim_end()))
                .join("\n"),
        )
    }

    /// `{ bpm : 92, label : "Verse", break }` as `bpm=92 label="Verse" break`.
    fn directive_line(&mut self, directives: &[Node]) -> String {
        directives
            .iter()
            .filter_map(|directive| {
                if as_bare_words(directive).as_deref() == Some(&["break"]) {
                    return Some("break".to_string());
                }
                let Some((words, value)) = as_field(directive) else {
                    self.error(
                        directive.span,
                        "expected a directive, like `bpm : 92` or `break`",
                    );
                    return None;
                };
                let key = words.join("_");
                let value = if key == "label" {
                    self.quoted_text(value, "a label")?
                } else {
                    self.flat_text(value)?
                };
                Some(format!("{key}={value}"))
            })
            .join(" ")
    }
}
