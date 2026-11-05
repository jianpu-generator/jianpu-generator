//! The text a user edits, and what the rest of the pipeline reads.
//!
//! A document is written in anyfix expressions (see `anyfix-syntax.md`). The
//! parser, source edits and part streams read its lowered form, in the section
//! syntax of `syntax.md`, so a [`Document`] holds both and translates between
//! them: offsets through a [`SourceMap`], and edits of the lowered form back
//! into the user's text with [`Document::with_lowered`].
//!
//! Text in the section syntax is its own lowered form, so files not yet
//! migrated keep working.

use itertools::Itertools;

use crate::error::Span;
use crate::format_anyfix::format_anyfix;
use crate::format_source::format_score;
use crate::parser::anyfix::{self, LoweringError, SourceMap};
use crate::parser::parts_parser::InstrumentInfo;
use crate::parser::section_splitter::split_sections;

/// A document's text and its lowered form.
#[derive(Debug, Clone)]
pub struct Document {
    text: String,
    lowered: String,
    /// `None` when the text is in the section syntax, so is its own lowered form.
    source_map: Option<SourceMap>,
    errors: Vec<LoweringError>,
}

/// One replacement in a document's text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextReplacement {
    pub span: Span,
    pub replacement: String,
}

/// Whether `text` is written in anyfix expressions. In anyfix a `#` line is a
/// comment, so a `# section` header only marks the section syntax when the text
/// doesn't open with an anyfix section.
pub fn is_anyfix(text: &str) -> bool {
    if text.trim().is_empty() {
        return false;
    }
    split_sections(text).0.is_empty() || opens_with_anyfix_section(text)
}

fn opens_with_anyfix_section(text: &str) -> bool {
    let nodes = anyfix_expr::parse(text).syntax.nodes();
    let Some(anyfix_expr::NodeKind::Call(call)) = nodes.first().map(|node| &node.kind) else {
        return false;
    };
    let name = call.name_text();
    [
        "metadata _",
        "parts _",
        "sequence _",
        "preset _ _",
        "score _",
    ]
    .iter()
    .any(|section| name.starts_with(section))
}

impl Document {
    /// Reads `text`, in either syntax. `instruments` names the sound a
    /// `sound : N` option picks.
    pub fn open(text: &str, instruments: &[InstrumentInfo]) -> Self {
        if !is_anyfix(text) {
            return Document {
                text: text.to_string(),
                lowered: text.to_string(),
                source_map: None,
                errors: Vec::new(),
            };
        }
        let lowered = anyfix::lower(text, instruments);
        Document {
            text: text.to_string(),
            lowered: lowered.source,
            source_map: Some(lowered.source_map),
            errors: lowered.errors,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn lowered(&self) -> &str {
        &self.lowered
    }

    /// Problems found while lowering, located in [`Self::text`].
    pub fn errors(&self) -> &[LoweringError] {
        &self.errors
    }

    pub fn is_anyfix(&self) -> bool {
        self.source_map.is_some()
    }

    /// A position in the lowered form as a position in the text.
    pub fn offset_to_text(&self, offset: usize) -> usize {
        self.source_map
            .as_ref()
            .map_or(offset, |map| map.to_anyfix(offset))
    }

    /// A position in the text as a position in the lowered form.
    pub fn offset_to_lowered(&self, offset: usize) -> usize {
        self.source_map
            .as_ref()
            .map_or(offset, |map| map.to_lowered(offset))
    }

    /// A range of the lowered form as a range of the text.
    pub fn span_to_text(&self, span: Span) -> Span {
        self.source_map
            .as_ref()
            .map_or(span, |map| map.span_to_anyfix(span))
    }

    /// A range of the text as a range of the lowered form.
    pub fn span_to_lowered(&self, span: Span) -> Span {
        self.source_map
            .as_ref()
            .map_or(span, |map| map.span_to_lowered(span))
    }

    /// The document whose lowered form is `lowered`, keeping as much of this
    /// document's text as it can: comments and layout survive outside what
    /// changed. A document with lowering errors keeps its text, since part of
    /// it isn't in the lowered form and rewriting would lose it.
    pub fn with_lowered(&self, lowered: &str, instruments: &[InstrumentInfo]) -> Document {
        if lowered == self.lowered || !self.errors.is_empty() {
            return self.clone();
        }
        if !self.is_anyfix() {
            return Document::open(lowered, instruments);
        }
        self.with_changed_lines(lowered, instruments)
            .unwrap_or_else(|| self.with_changed_sections(lowered, instruments))
    }

    /// The text formatted, in its own syntax.
    pub fn formatted(&self) -> String {
        if self.is_anyfix() {
            format_anyfix(&self.text)
        } else {
            format_score(&self.text)
        }
    }

    /// Applies `lowered`'s line changes to the matching places in the text.
    /// `None` when lines were added or removed, or the result doesn't lower to
    /// `lowered`.
    fn with_changed_lines(
        &self,
        lowered: &str,
        instruments: &[InstrumentInfo],
    ) -> Option<Document> {
        let old_lines = self.lowered.split_inclusive('\n').collect_vec();
        let new_lines = lowered.split_inclusive('\n').collect_vec();
        if old_lines.len() != new_lines.len() {
            return None;
        }
        let mut line_start = 0;
        let mut replacements = Vec::new();
        for (old_line, new_line) in old_lines.iter().zip(&new_lines) {
            if old_line != new_line {
                let changed = changed_range(old_line, new_line);
                let span = self.span_to_text(Span::new(
                    line_start + changed.old_start,
                    line_start + changed.old_end,
                ));
                replacements.push(TextReplacement {
                    span,
                    replacement: new_line
                        .get(changed.new_start..changed.new_end)?
                        .to_string(),
                });
            }
            line_start += old_line.len();
        }
        let candidate =
            Document::open(&apply_replacements(&self.text, &replacements)?, instruments);
        (candidate.errors.is_empty() && candidate.lowered == lowered).then_some(candidate)
    }

    /// Rewrites the top-level sections that lower differently, keeping the
    /// text of those that don't.
    fn with_changed_sections(&self, lowered: &str, instruments: &[InstrumentInfo]) -> Document {
        let rewritten = format_anyfix(&anyfix::convert(lowered));
        let kept = kept_sections(&self.text, &rewritten, instruments);
        let candidate = Document::open(&kept, instruments);
        if candidate.errors.is_empty() {
            candidate
        } else {
            Document::open(&rewritten, instruments)
        }
    }
}

/// The smallest range of a line that changed.
struct ChangedRange {
    old_start: usize,
    old_end: usize,
    new_start: usize,
    new_end: usize,
}

fn changed_range(old: &str, new: &str) -> ChangedRange {
    let prefix = old
        .char_indices()
        .zip(new.chars())
        .take_while(|((_, old_char), new_char)| old_char == new_char)
        .last()
        .map_or(0, |((index, character), _)| index + character.len_utf8());
    let old_rest = old.get(prefix..).unwrap_or_default();
    let new_rest = new.get(prefix..).unwrap_or_default();
    let suffix = old_rest
        .chars()
        .rev()
        .zip(new_rest.chars().rev())
        .take_while(|(old_char, new_char)| old_char == new_char)
        .map(|(character, _)| character.len_utf8())
        .sum::<usize>();
    ChangedRange {
        old_start: prefix,
        old_end: old.len() - suffix,
        new_start: prefix,
        new_end: new.len() - suffix,
    }
}

/// `text` with non-overlapping `replacements`, in document order.
fn apply_replacements(text: &str, replacements: &[TextReplacement]) -> Option<String> {
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0;
    for replacement in replacements {
        output.push_str(text.get(cursor..replacement.span.start)?);
        output.push_str(&replacement.replacement);
        cursor = replacement.span.end;
    }
    output.push_str(text.get(cursor..)?);
    Some(output)
}

/// One replacement turning `old` into `new`, or `None` when they are equal.
pub fn text_difference(old: &str, new: &str) -> Option<TextReplacement> {
    if old == new {
        return None;
    }
    let changed = changed_range(old, new);
    Some(TextReplacement {
        span: Span::new(changed.old_start, changed.old_end),
        replacement: new.get(changed.new_start..changed.new_end)?.to_string(),
    })
}

/// A top-level section: where it is, and what it lowers to.
struct SectionText {
    span: Span,
    lowered: String,
}

fn section_texts(text: &str, instruments: &[InstrumentInfo]) -> Vec<SectionText> {
    anyfix_expr::parse(text)
        .syntax
        .nodes()
        .iter()
        .filter_map(|node| {
            let section = text.get(node.span.start..node.span.end)?;
            Some(SectionText {
                span: Span::new(node.span.start, node.span.end),
                lowered: anyfix::lower(section, instruments).source,
            })
        })
        .collect()
}

/// `rewritten`, with the text of `old`'s unchanged leading and trailing
/// sections in place of their rewritten form.
fn kept_sections(old: &str, rewritten: &str, instruments: &[InstrumentInfo]) -> String {
    let old_sections = section_texts(old, instruments);
    let new_sections = section_texts(rewritten, instruments);
    let same = |first: &SectionText, second: &SectionText| first.lowered == second.lowered;
    let prefix = old_sections
        .iter()
        .zip(&new_sections)
        .take_while(|(first, second)| same(first, second))
        .count();
    let longest_suffix = old_sections.len().min(new_sections.len()) - prefix;
    let suffix = old_sections
        .iter()
        .rev()
        .zip(new_sections.iter().rev())
        .take(longest_suffix)
        .take_while(|(first, second)| same(first, second))
        .count();
    let changed_old = old_sections
        .get(prefix..old_sections.len() - suffix)
        .unwrap_or_default();
    let changed_new = new_sections
        .get(prefix..new_sections.len() - suffix)
        .unwrap_or_default();
    let new_text = match (changed_new.first(), changed_new.last()) {
        (Some(first), Some(last)) => rewritten.get(first.span.start..last.span.end),
        _ => Some(""),
    }
    .unwrap_or_default();
    let spliced = match (changed_old.first(), changed_old.last()) {
        (Some(first), Some(last)) => splice_changed_sections(old, first.span, last.span, new_text),
        _ => insert_sections(old, &old_sections, prefix, new_text),
    };
    spliced.unwrap_or_else(|| rewritten.to_string())
}

/// `old` with the sections from `first` to `last` replaced by `new_text`. With
/// nothing to put there, the separator before them goes too.
fn splice_changed_sections(old: &str, first: Span, last: Span, new_text: &str) -> Option<String> {
    let before = old.get(..first.start)?;
    let after = old.get(last.end..)?;
    if new_text.is_empty() {
        let after = after.trim_start_matches([',', ' ', '\t', '\r', '\n']);
        return Some(format!("{before}{after}"));
    }
    Some(format!("{before}{new_text}{after}"))
}

/// `old` with `new_text` inserted as new sections after its first `prefix`
/// sections.
fn insert_sections(
    old: &str,
    sections: &[SectionText],
    prefix: usize,
    new_text: &str,
) -> Option<String> {
    if new_text.is_empty() {
        return Some(old.to_string());
    }
    let insertion = prefix
        .checked_sub(1)
        .and_then(|index| sections.get(index))
        .map(|section| section.span.end);
    match insertion {
        Some(end) => {
            let before = old.get(..end)?;
            let after = old.get(end..)?;
            let after = after.strip_prefix(',').unwrap_or(after);
            Some(format!("{before},\n\n{new_text},{after}"))
        }
        None => Some(format!("{new_text},\n\n{old}")),
    }
}

#[cfg(test)]
#[path = "document_tests.rs"]
mod document_tests;
