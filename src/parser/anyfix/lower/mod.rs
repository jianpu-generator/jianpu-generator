//! Reads an `.anyfix` document and writes the same document in the current
//! `.poentaro` syntax, section by section.

mod parts;
mod score;

use anyfix_expr::{Call, NamePart, Node, NodeKind, Span};
use itertools::Itertools;

use super::source_map::{MappedText, SourceMap};
use crate::error::Span as SourceSpan;
use crate::parser::parts_parser::InstrumentInfo;

/// A problem found while lowering, located in the `.anyfix` source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringError {
    pub span: Span,
    pub message: String,
}

/// The lowered document, and every problem found on the way. A part of the
/// source with a problem is left out of `source`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lowered {
    pub source: String,
    pub errors: Vec<LoweringError>,
    /// Maps offsets between `source` and the `.anyfix` text it was lowered from.
    pub source_map: SourceMap,
}

/// Lowers an `.anyfix` document to the current `.poentaro` syntax.
/// `instruments` names the sound a `sound : N` option picks.
pub fn lower(source: &str, instruments: &[InstrumentInfo]) -> Lowered {
    let parsed = anyfix_expr::parse(source);
    let mut lowerer = Lowerer {
        anyfix_source: source,
        instruments,
        errors: parsed
            .errors
            .iter()
            .map(|error| LoweringError {
                span: error.span,
                message: error.message.clone(),
            })
            .collect(),
    };
    let mut output = MappedText::default();
    let sections = parsed
        .syntax
        .nodes()
        .iter()
        .filter_map(|node| lowerer.section(node))
        .collect_vec();
    for (index, section) in sections.into_iter().enumerate() {
        if index > 0 {
            output.text.push('\n');
        }
        output.append(section);
    }
    Lowered {
        source: output.text,
        errors: lowerer.errors,
        source_map: SourceMap {
            segments: output.segments,
        },
    }
}

pub(super) struct Lowerer<'a> {
    pub(super) anyfix_source: &'a str,
    pub(super) instruments: &'a [InstrumentInfo],
    pub(super) errors: Vec<LoweringError>,
}

/// A lowered line and the `.anyfix` node it was rewritten from.
pub(super) struct OriginLine {
    pub(super) text: String,
    pub(super) origin: SourceSpan,
}

pub(super) fn source_span(span: Span) -> SourceSpan {
    SourceSpan::new(span.start, span.end)
}

/// A call's first name word (`metadata` in `metadata { … }`).
fn keyword_span(call: &Call) -> Option<SourceSpan> {
    call.name.iter().find_map(|part| match part {
        NamePart::Word { span, .. } => Some(source_span(*span)),
        NamePart::Slot => None,
    })
}

/// The call `node` is, with its name text (`parts _`).
pub(super) fn as_call(node: &Node) -> Option<(&Call, String)> {
    match &node.kind {
        NodeKind::Call(call) => Some((call, call.name_text())),
        _ => None,
    }
}

/// The name words of a call that ends in one slot (`max measures per system _`),
/// and that slot's argument: one `key : value` field.
pub(super) fn as_field(node: &Node) -> Option<(Vec<&str>, &Node)> {
    let (call, _) = as_call(node)?;
    let (last, words) = call.name.split_last()?;
    let words: Vec<&str> = words
        .iter()
        .map(|part| match part {
            NamePart::Word { text, .. } => Some(text.as_str()),
            NamePart::Slot => None,
        })
        .collect::<Option<_>>()?;
    match (last, call.arguments.as_slice()) {
        (NamePart::Slot, [value]) if !words.is_empty() => Some((words, value)),
        _ => None,
    }
}

/// A call with no slots, as its words (`break`, `sans serif`).
pub(super) fn as_bare_words(node: &Node) -> Option<Vec<&str>> {
    let (call, _) = as_call(node)?;
    call.name
        .iter()
        .map(|part| match part {
            NamePart::Word { text, .. } => Some(text.as_str()),
            NamePart::Slot => None,
        })
        .collect()
}

impl Lowerer<'_> {
    pub(super) fn error(&mut self, span: Span, message: impl Into<String>) {
        self.errors.push(LoweringError {
            span,
            message: message.into(),
        });
    }

    /// One top-level section, as its `# header` and body.
    fn section(&mut self, node: &Node) -> Option<MappedText> {
        let (call, name) = as_call(node)?;
        let mut output = MappedText::default();
        let keyword = keyword_span(call).unwrap_or_else(|| source_span(node.span));
        match (name.as_str(), call.arguments.as_slice()) {
            ("metadata _", [fields]) => {
                let lines = self.metadata_fields(fields)?;
                self.push_line(&mut output, "# metadata", keyword);
                self.push_lines(&mut output, lines);
            }
            ("parts _", [parts]) => {
                let lines = self.parts(parts)?;
                self.push_line(&mut output, "# parts", keyword);
                self.push_lines(&mut output, lines);
            }
            ("sequence _", [entries]) => {
                let line = self.sequence(entries)?;
                self.push_line(&mut output, "# sequence", keyword);
                self.push_line(&mut output, &line, source_span(entries.span));
            }
            ("preset _ _", [preset_name, fields]) => {
                let (header, lines) = self.preset(preset_name, fields)?;
                let origin = SourceSpan::new(keyword.start, preset_name.span.end);
                self.push_line(&mut output, &header, origin);
                self.push_lines(&mut output, lines);
            }
            ("score _", [body]) => {
                self.push_line(&mut output, "# score", keyword);
                self.push_score(&mut output, body)?;
            }
            _ => {
                self.error(
                    node.span,
                    format!(
                        "found `{name}` at the top level; expected `metadata {{…}}`, \
                         `parts […]`, `sequence […]`, `preset \"Name\" {{…}}` or `score \"\"\"…\"\"\"`"
                    ),
                );
                return None;
            }
        }
        Some(output)
    }

    pub(super) fn push_line(&self, output: &mut MappedText, line: &str, origin: SourceSpan) {
        output.push_rewritten_line(line, origin, self.anyfix_source);
    }

    fn push_lines(&self, output: &mut MappedText, lines: Vec<OriginLine>) {
        for line in lines {
            self.push_line(output, &line.text, line.origin);
        }
    }

    pub(super) fn elements<'n>(&mut self, node: &'n Node, expected: &str) -> Option<&'n [Node]> {
        match &node.kind {
            NodeKind::Brackets(elements) if expected == "[" => Some(elements),
            NodeKind::Braces(elements) if expected == "{" => Some(elements),
            _ => {
                let closer = if expected == "[" { "]" } else { "}" };
                self.error(node.span, format!("expected `{expected} … {closer}` here"));
                None
            }
        }
    }

    // ── values ──────────────────────────────────────────────────────────

    /// A value as the current syntax spells it outside quotes: `80%`, `+1`,
    /// `3/4`, `sans_serif`, `Bb4`.
    pub(super) fn flat_text(&mut self, node: &Node) -> Option<String> {
        self.joined_text(node, "_")
    }

    /// Like [`Self::flat_text`], with `separator` between two name words.
    /// Built from the tree, not the source text, so formatting the document
    /// (which spaces out `A1&T` as `A1 & T`) reads the same.
    fn joined_text(&mut self, node: &Node, separator: &str) -> Option<String> {
        match &node.kind {
            NodeKind::Number(text) => Some(text.clone()),
            NodeKind::String(literal) => Some(literal.value.clone()),
            NodeKind::Call(call) => {
                let mut arguments = call.arguments.iter();
                let mut text = String::new();
                let mut previous_was_name_word = false;
                for part in &call.name {
                    match part {
                        NamePart::Word { text: word, .. } => {
                            let is_name = super::words::is_name_word(word);
                            if is_name && previous_was_name_word {
                                text.push_str(separator);
                            }
                            text.push_str(word);
                            previous_was_name_word = is_name;
                        }
                        NamePart::Slot => {
                            let argument = arguments.next()?;
                            text.push_str(&self.joined_text(argument, separator)?);
                            previous_was_name_word = false;
                        }
                    }
                }
                Some(text)
            }
            _ => {
                self.error(node.span, "expected a word, number or string here");
                None
            }
        }
    }

    /// A string's value, quoted the way the current syntax quotes it.
    pub(super) fn quoted_text(&mut self, node: &Node, what: &str) -> Option<String> {
        match &node.kind {
            NodeKind::String(literal) if literal.value.contains(['"', '\n']) => {
                self.error(
                    node.span,
                    format!("{what} can't contain `\"` or a line break yet"),
                );
                None
            }
            NodeKind::String(literal) => Some(format!("\"{}\"", literal.value)),
            _ => {
                self.error(node.span, format!("{what} must be a string"));
                None
            }
        }
    }

    /// An abbreviation: a string's value, or the element's words and symbols
    /// joined without spaces (`A1 & T` is `A1&T`), except that two name words
    /// keep one.
    pub(super) fn abbreviation(&mut self, node: &Node) -> Option<String> {
        self.joined_text(node, " ")
    }

    /// The one abbreviation inside `[ ]`.
    pub(super) fn bracketed_abbreviation(&mut self, node: &Node) -> Option<String> {
        match &node.kind {
            NodeKind::Brackets(elements) if elements.len() == 1 => elements
                .first()
                .and_then(|element| self.abbreviation(element)),
            _ => {
                self.error(
                    node.span,
                    "expected one part abbreviation in brackets, like `[S1]`",
                );
                None
            }
        }
    }

    /// Every abbreviation in a `[a, b]` list.
    pub(super) fn abbreviation_list(&mut self, node: &Node) -> Option<Vec<String>> {
        let elements = self.elements(node, "[")?;
        elements
            .iter()
            .map(|element| self.abbreviation(element))
            .collect()
    }

    // ── metadata ────────────────────────────────────────────────────────

    fn metadata_fields(&mut self, node: &Node) -> Option<Vec<OriginLine>> {
        let fields = self.elements(node, "{")?;
        Some(
            fields
                .iter()
                .filter_map(|field| {
                    Some(OriginLine {
                        text: self.metadata_field(field)?,
                        origin: source_span(field.span),
                    })
                })
                .collect(),
        )
    }

    /// One `key : value` field as a `key = value` line.
    pub(super) fn metadata_field(&mut self, node: &Node) -> Option<String> {
        let Some((words, value)) = as_field(node) else {
            self.error(node.span, "expected a `name : value` field");
            return None;
        };
        let key = words.join("_");
        match &value.kind {
            NodeKind::Braces(style_fields) => {
                let Some(kind) = key.strip_suffix("_style") else {
                    self.error(
                        value.span,
                        format!(
                            "only a text style takes `{{ … }}`; did you mean `{} style`?",
                            words.join(" ")
                        ),
                    );
                    return None;
                };
                let fields = style_fields
                    .iter()
                    .filter_map(|field| {
                        let Some((words, value)) = as_field(field) else {
                            self.error(field.span, "expected a `name : value` field");
                            return None;
                        };
                        Some(format!("{}: {}", words.join("_"), self.flat_text(value)?))
                    })
                    .join(", ");
                Some(format!("{kind} = {{ {fields} }}"))
            }
            NodeKind::String(_) => Some(format!(
                "{key} = {}",
                self.quoted_text(value, "a metadata text")?
            )),
            NodeKind::Parens(pair) => {
                let [x, y] = pair.as_slice() else {
                    self.error(value.span, "expected a pair, like `(0, -4)`");
                    return None;
                };
                Some(format!(
                    "{key} = {} {}",
                    self.flat_text(x)?,
                    self.flat_text(y)?
                ))
            }
            _ => Some(format!("{key} = {}", self.flat_text(value)?)),
        }
    }
}
