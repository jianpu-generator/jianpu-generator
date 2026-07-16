mod declaration_parsing;
mod instrument_matching;
mod lexer;

use crate::ast::parsed::{PartDecl, PartKind, Soundfont};
use crate::error::{RecoverableError, Span};

pub use instrument_matching::InstrumentInfo;
use lexer::lex_line;

use declaration_parsing::parse_declaration_line;

#[cfg(test)]
mod lexer_tests;
#[cfg(test)]
mod percussion_tests;
#[cfg(test)]
mod tests;

pub fn parse_parts(
    content: &str,
    base_offset: usize,
    instruments: &[InstrumentInfo],
) -> (Vec<PartDecl>, Vec<RecoverableError>) {
    let mut errors = Vec::new();
    let raw = collect_raw_declarations(content, base_offset, &mut errors, instruments);
    let declarations = resolve_declarations(raw, &mut errors);
    if declarations.is_empty() {
        let section_span = Span::new(base_offset, base_offset + content.len().max(1));
        errors.push(RecoverableError::parts_empty_section(section_span));
    }
    (declarations, errors)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourcePartMode {
    Chords,
    Notes,
    NotesLyrics,
    Percussion,
    Follow,
}

/// Source-level part declaration before follow inheritance is applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRawPartDecl {
    pub display_name: String,
    pub abbreviation: String,
    pub line_number: u32,
    pub mode: SourcePartMode,
    pub follow_target: Option<String>,
    pub soundfont: Option<Soundfont>,
    pub volume: Option<u8>,
    pub octave_offset: Option<i8>,
}

struct RawDecl {
    display_name: String,
    abbreviation: String,
    span: Span,
    kind: RawKind,
    /// `None` when omitted on the declaration line (follow parts inherit from target).
    soundfont: Option<Soundfont>,
    /// `None` when omitted on the declaration line (follow parts inherit from target).
    volume: Option<u8>,
    /// `None` when omitted on the declaration line (follow parts inherit from target).
    octave_offset: Option<i8>,
}

enum RawKind {
    Concrete(PartKind),
    Follow { target: String, target_span: Span },
}

fn byte_offset_to_line_number(source: &str, byte_offset: usize) -> u32 {
    source
        .as_bytes()
        .iter()
        .take(byte_offset.min(source.len()))
        .filter(|&&byte| byte == b'\n')
        .count() as u32
        + 1
}

fn raw_kind_to_source_mode(kind: &RawKind) -> (SourcePartMode, Option<String>) {
    match kind {
        RawKind::Concrete(PartKind::Chords) => (SourcePartMode::Chords, None),
        RawKind::Concrete(PartKind::Notes) => (SourcePartMode::Notes, None),
        RawKind::Concrete(PartKind::NotesWithLyrics) => (SourcePartMode::NotesLyrics, None),
        RawKind::Concrete(PartKind::Percussion) => (SourcePartMode::Percussion, None),
        RawKind::Follow { target, .. } => (SourcePartMode::Follow, Some(target.clone())),
    }
}

/// Collect source-level part declarations from a `# parts` section body.
pub fn collect_source_raw_declarations(
    content: &str,
    base_offset: usize,
    full_source: &str,
    errors: &mut Vec<RecoverableError>,
    instruments: &[InstrumentInfo],
) -> Vec<SourceRawPartDecl> {
    collect_raw_declarations(content, base_offset, errors, instruments)
        .into_iter()
        .map(|raw_decl| {
            let (mode, follow_target) = raw_kind_to_source_mode(&raw_decl.kind);
            SourceRawPartDecl {
                display_name: raw_decl.display_name,
                abbreviation: raw_decl.abbreviation,
                line_number: byte_offset_to_line_number(full_source, raw_decl.span.start),
                mode,
                follow_target,
                soundfont: raw_decl.soundfont,
                volume: raw_decl.volume,
                octave_offset: raw_decl.octave_offset,
            }
        })
        .collect()
}

struct ParsedPartRhs {
    kind: RawKind,
    soundfont: Option<Soundfont>,
    volume: Option<u8>,
    octave_offset: Option<i8>,
}

struct LhsParsed {
    display_name: String,
    abbreviation: String,
}

struct RhsSuffixes {
    soundfont: Option<Soundfont>,
    volume: Option<u8>,
    octave_offset: Option<i8>,
}

fn collect_raw_declarations(
    content: &str,
    base_offset: usize,
    errors: &mut Vec<RecoverableError>,
    instruments: &[InstrumentInfo],
) -> Vec<RawDecl> {
    let mut raw_declarations = Vec::new();
    let mut seen_abbreviations = std::collections::HashSet::new();
    let mut byte_offset = base_offset;

    for line in content.lines() {
        let trimmed = line.trim();
        let line_start = byte_offset;
        byte_offset += line.len() + 1;
        if trimmed.is_empty() {
            continue;
        }
        let line_span = Span::new(line_start, line_start + line.len());
        let trimmed_start = line_start + (line.len() - line.trim_start().len());

        let tokens = match lex_line(trimmed, trimmed_start, line_span) {
            Ok(tokens) => tokens,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };

        let Some(raw_decl) = parse_declaration_line(&tokens, line_span, errors, instruments) else {
            continue;
        };

        if !seen_abbreviations.insert(raw_decl.abbreviation.clone()) {
            errors.push(RecoverableError::parts_duplicate_abbreviation(
                line_span,
                &raw_decl.abbreviation,
            ));
            continue;
        }

        raw_declarations.push(raw_decl);
    }

    raw_declarations
}

fn resolve_declarations(raw: Vec<RawDecl>, errors: &mut Vec<RecoverableError>) -> Vec<PartDecl> {
    let mut declarations = Vec::new();
    for (index, raw_decl) in raw.into_iter().enumerate() {
        let RawDecl {
            display_name,
            abbreviation,
            span,
            kind,
            soundfont,
            volume,
            octave_offset,
        } = raw_decl;
        match kind {
            RawKind::Follow {
                target,
                target_span,
            } => {
                if index == 0 {
                    errors.push(RecoverableError::parts_first_part_cannot_follow(span));
                    continue;
                }
                let found = declarations
                    .iter()
                    .find(|declaration: &&PartDecl| declaration.abbreviation == target);
                match found {
                    None => {
                        errors.push(RecoverableError::parts_follow_unknown_target(
                            target_span,
                            &target,
                        ));
                        continue;
                    }
                    Some(target_decl) => declarations.push(PartDecl {
                        abbreviation,
                        display_name,
                        kind: target_decl.kind,
                        follow_target: Some(target),
                        soundfont: soundfont.unwrap_or(target_decl.soundfont),
                        volume: volume.unwrap_or(target_decl.volume),
                        octave_offset: octave_offset.unwrap_or(target_decl.octave_offset),
                    }),
                }
            }
            RawKind::Concrete(kind) => declarations.push(PartDecl {
                abbreviation,
                display_name,
                kind,
                follow_target: None,
                soundfont: soundfont.unwrap_or_default(),
                volume: volume.unwrap_or(100),
                octave_offset: octave_offset.unwrap_or(0),
            }),
        }
    }
    declarations
}
