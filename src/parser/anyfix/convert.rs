//! Writes a `.poentaro` document in the current syntax as an `.anyfix`
//! document (see `anyfix-syntax.md`).

use itertools::Itertools;

use super::words::{abbreviation, is_name_word, label, quoted, snake_case_as_words};
use crate::parser::parts_parser::{
    collect_source_raw_declarations, SourcePartMode, SourceRawPartDecl,
};
use crate::parser::section_splitter::{split_sections, RawSection, SectionKind};
use crate::parser::sequence_parser::{parse_sequence, PartFilterKind};

/// The metadata field the current syntax writes as a pair, `x y`.
const DIRECTIVE_ROW_OFFSET_KEY: &str = crate::ast::parsed::DIRECTIVE_ROW_OFFSET_KEY;

/// The preset fields that hold a list of part abbreviations.
const PART_LIST_KEYS: [&str; 2] = ["visible_parts", "soloed_parts"];

/// Converts a document in the current syntax to anyfix expressions, in
/// anyfix-expr's canonical layout. Sections keep their order. Comments
/// outside `# score` are dropped.
pub fn convert(source: &str) -> String {
    let converted = convert_sections(source);
    anyfix_expr::format(&converted).unwrap_or(converted)
}

fn convert_sections(source: &str) -> String {
    let (sections, _) = split_sections(source);
    sections
        .iter()
        .map(|section| match section.kind {
            SectionKind::Metadata => {
                format!(
                    "metadata {},\n",
                    record(&key_value_fields(&section.content))
                )
            }
            SectionKind::Parts => parts(section, source),
            SectionKind::Sequence => sequence(section),
            SectionKind::Preset => preset(section),
            SectionKind::Score => score(section, source),
        })
        .join("\n")
}

/// Fields as a multi-line `{ … }` record.
fn record(fields: &[String]) -> String {
    if fields.is_empty() {
        return "{}".to_string();
    }
    let lines = fields
        .iter()
        .map(|field| match field.starts_with('#') {
            true => format!("  {field}\n"),
            false => format!("  {field},\n"),
        })
        .join("");
    format!("{{\n{lines}}}")
}

/// Elements as a multi-line `[ … ]` list.
fn list(elements: &[String]) -> String {
    if elements.is_empty() {
        return "[]".to_string();
    }
    let lines = elements
        .iter()
        .map(|element| match element.starts_with('#') {
            true => format!("  {element}\n"),
            false => format!("  {element},\n"),
        })
        .join("");
    format!("[\n{lines}]")
}

/// A snake_case key as words, or quoted when it can't be (the lowering then
/// reports it as an unknown field, as the current parser would).
fn key_words(key: &str) -> String {
    snake_case_as_words(key).unwrap_or_else(|| quoted(key))
}

// ── metadata and presets ─────────────────────────────────────────────────

/// Every `key = value` line of a metadata or preset section, as fields.
fn key_value_fields(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| match line.split_once('=') {
            Some((key, value)) => key_value_field(key.trim(), value.trim()),
            // The current parser reports this line; keep it visible.
            None => format!("# could not convert: {}", line.trim()),
        })
        .collect()
}

fn key_value_field(key: &str, value: &str) -> String {
    if PART_LIST_KEYS.contains(&key) {
        let parts = value
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .map(abbreviation)
            .join(", ");
        return format!("{} : [{parts}]", key_words(key));
    }
    if let Some(inner) = value
        .strip_prefix('{')
        .and_then(|rest| rest.strip_suffix('}'))
    {
        let fields = inner
            .split(',')
            .filter(|field| !field.trim().is_empty())
            .map(|field| match field.split_once(':') {
                Some((name, value)) => {
                    format!("{} : {}", key_words(name.trim()), scalar(value.trim()))
                }
                None => quoted(field.trim()),
            })
            .join(", ");
        return format!("{} style : {{ {fields} }}", key_words(key));
    }
    if key == DIRECTIVE_ROW_OFFSET_KEY {
        if let Some((x, y)) = value.trim_matches('"').split_whitespace().collect_tuple() {
            return format!("{} : ({x}, {y})", key_words(key));
        }
    }
    format!("{} : {}", key_words(key), scalar(value))
}

/// A value: quoted text stays quoted, numbers and signed numbers stay as
/// written, snake_case words become runs (`sans_serif` → `sans serif`).
fn scalar(value: &str) -> String {
    if let Some(text) = value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    {
        return quoted(text);
    }
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    let is_number = unsigned.starts_with(|c: char| c.is_ascii_digit())
        && unsigned
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.');
    if is_number {
        return value.to_string();
    }
    match snake_case_as_words(value) {
        Some(words) if is_name_word(value) => words,
        _ => quoted(value),
    }
}

fn preset(section: &RawSection) -> String {
    let name = section
        .preset_name
        .as_ref()
        .map_or_else(|| quoted(""), |name| quoted(&name.text));
    format!(
        "preset {name} {},\n",
        record(&key_value_fields(&section.content))
    )
}

// ── parts ────────────────────────────────────────────────────────────────

fn parts(section: &RawSection, source: &str) -> String {
    let declarations = collect_source_raw_declarations(
        &section.content,
        section.content_offset,
        source,
        &mut Vec::new(),
        &[],
    );
    let first_line = byte_offset_to_line_number(source, section.content_offset);
    let declared_lines = declarations
        .iter()
        .map(|declaration| declaration.line_number)
        .collect_vec();
    // A line the current parser rejects is kept visible rather than dropped.
    let rejected = section
        .content
        .lines()
        .zip(first_line..)
        .filter(|(line, number)| !line.trim().is_empty() && !declared_lines.contains(number))
        .map(|(line, _)| format!("# could not convert: {}", line.trim()));
    let elements = declarations.iter().map(part).chain(rejected).collect_vec();
    format!("parts {},\n", list(&elements))
}

/// The 1-based line number of `byte_offset` in `source`.
fn byte_offset_to_line_number(source: &str, byte_offset: usize) -> u32 {
    let before = source
        .get(..byte_offset.min(source.len()))
        .unwrap_or_default();
    before.matches('\n').count() as u32 + 1
}

fn part(declaration: &SourceRawPartDecl) -> String {
    let display = quoted(&declaration.display_name);
    let left = if declaration.abbreviation == declaration.display_name {
        display
    } else {
        format!("{display} [{}]", abbreviation(&declaration.abbreviation))
    };
    let target = || abbreviation(declaration.follow_target.as_deref().unwrap_or_default());
    let head = match declaration.mode {
        SourcePartMode::Notes => "notes".to_string(),
        SourcePartMode::Chords => "chords".to_string(),
        SourcePartMode::Percussion => "percussion".to_string(),
        SourcePartMode::Follow => format!("follow [{}]", target()),
        SourcePartMode::Lyrics => format!("lyrics [{}]", target()),
    };
    let options = [
        declaration
            .range_instrument
            .map(|instrument| format!("instrument : {}", instrument.keyword())),
        declaration
            .soundfont
            .map(|sound| format!("sound : {}", sound.0)),
        declaration
            .volume
            .map(|volume| format!("volume : {volume}%")),
        declaration
            .octave_offset
            .map(|octave| format!("octave : {octave:+}")),
    ]
    .into_iter()
    .flatten()
    .join(", ");
    if options.is_empty() {
        format!("{left} : {head}")
    } else {
        format!("{left} : {head} {{ {options} }}")
    }
}

// ── sequence ─────────────────────────────────────────────────────────────

fn sequence(section: &RawSection) -> String {
    let (sequence, _) = parse_sequence(&section.content, section.content_offset);
    let entries = sequence
        .map(|sequence| sequence.entries)
        .unwrap_or_default();
    let entries = entries
        .iter()
        .map(|entry| {
            let label = label(&entry.label);
            match &entry.part_filter {
                None => label,
                Some(filter) => {
                    let keyword = match filter.kind {
                        PartFilterKind::Omit => "WITHOUT",
                        PartFilterKind::Only => "ONLY",
                    };
                    let parts = filter
                        .parts
                        .iter()
                        .map(|part| abbreviation(part))
                        .join(", ");
                    format!("{label} {keyword} [{parts}]")
                }
            }
        })
        .join(", ");
    format!("sequence [ {entries} ],\n")
}

// ── score ────────────────────────────────────────────────────────────────

/// The score as one raw string: the `# score` body as it is, directive lines,
/// comments and blank lines kept.
fn score(section: &RawSection, source: &str) -> String {
    let end = (section.content_offset + section.content.len()).min(source.len());
    let original = source.get(section.content_offset..end).unwrap_or_default();
    let lines = original.lines().map(str::to_string).collect_vec();
    let body = raw_string(&lines).unwrap_or_else(|| "\"\"\"\n  \"\"\"".to_string());
    format!("score {body},\n")
}

/// Lines as an indented raw string, or `None` when they are all blank. The
/// quotes outnumber any run of quotes that starts a line, so no line closes
/// the string early.
fn raw_string(lines: &[String]) -> Option<String> {
    let first = lines.iter().position(|line| !line.trim().is_empty())?;
    let last = lines.iter().rposition(|line| !line.trim().is_empty())?;
    let lines = lines.get(first..=last)?;
    let longest_leading_quotes = lines
        .iter()
        .map(|line| line.trim_start().chars().take_while(|&c| c == '"').count())
        .max()
        .unwrap_or(0);
    let quotes = "\"".repeat((longest_leading_quotes + 1).max(3));
    let body = lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("  {}\n", line.trim_end())
            }
        })
        .map(|line| {
            if line.is_empty() {
                "\n".to_string()
            } else {
                line
            }
        })
        .join("");
    Some(format!("{quotes}\n{body}  {quotes}"))
}
