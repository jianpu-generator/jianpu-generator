mod declaration_parsing;
mod instrument_matching;
mod lexer;
mod setting_limits;

use crate::ast::parsed::{PartDecl, PartKind, Soundfont};
use crate::error::{RecoverableError, Span};

pub use instrument_matching::InstrumentInfo;
use lexer::lex_line;
pub use setting_limits::{
    ClampedSetting, PartSettingLimits, PartSettingRange, PART_SETTING_LIMITS,
};

use declaration_parsing::parse_declaration_line;

#[cfg(test)]
mod lexer_tests;
#[cfg(test)]
mod lyrics_part_tests;
#[cfg(test)]
mod percussion_tests;
#[cfg(test)]
mod setting_limits_tests;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourcePartMode {
    Chords,
    Notes,
    Percussion,
    Follow,
    Lyrics,
}

/// Source-level part declaration before follow inheritance is applied.
/// A part's volume when its `# parts` line has no `N%` suffix.
pub const DEFAULT_PART_VOLUME: u8 = 100;
/// A part's octave offset when its `# parts` line has no `+N`/`-N` suffix.
pub const DEFAULT_PART_OCTAVE_OFFSET: i8 = 0;

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
    abbreviation_span: Span,
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
    Lyrics { target: String, target_span: Span },
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
        RawKind::Concrete(PartKind::Percussion) => (SourcePartMode::Percussion, None),
        // A lyric part is only ever built as `RawKind::Lyrics`.
        RawKind::Concrete(PartKind::Lyrics { .. }) => (SourcePartMode::Lyrics, None),
        RawKind::Follow { target, .. } => (SourcePartMode::Follow, Some(target.clone())),
        RawKind::Lyrics { target, .. } => (SourcePartMode::Lyrics, Some(target.clone())),
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
    abbreviation_span: Span,
}

struct RhsSuffixes {
    soundfont: Option<Soundfont>,
    volume: Option<u8>,
    octave_offset: Option<i8>,
}

/// One non-blank line of a `# parts` section.
struct PartsLine<'a> {
    trimmed: &'a str,
    trimmed_start: usize,
    line_span: Span,
}

fn parts_lines(content: &str, base_offset: usize) -> impl Iterator<Item = PartsLine<'_>> {
    content
        .lines()
        .scan(base_offset, |next_offset, line| {
            let line_start = *next_offset;
            *next_offset += line.len() + 1;
            Some(PartsLine {
                trimmed: line.trim(),
                trimmed_start: line_start + (line.len() - line.trim_start().len()),
                line_span: Span::new(line_start, line_start + line.len()),
            })
        })
        .filter(|line| !line.trimmed.is_empty())
}

/// Spans of every part-kind keyword (`notes`, `chords`, `percussion`,
/// `follow`) in a `# parts` section's `content`, from the parts lexer.
pub(crate) fn part_kind_spans(content: &str, base_offset: usize) -> Vec<Span> {
    parts_lines(content, base_offset)
        .filter_map(|line| lex_line(line.trimmed, line.trimmed_start, line.line_span).ok())
        .flatten()
        .filter_map(|token| lexer::kind_keyword_span(&token))
        .collect()
}

fn collect_raw_declarations(
    content: &str,
    base_offset: usize,
    errors: &mut Vec<RecoverableError>,
    instruments: &[InstrumentInfo],
) -> Vec<RawDecl> {
    let mut raw_declarations = Vec::new();
    let mut seen_abbreviations = std::collections::HashSet::new();

    for line in parts_lines(content, base_offset) {
        let PartsLine {
            trimmed,
            trimmed_start,
            line_span,
        } = line;
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
            abbreviation_span,
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
                        abbreviation_span,
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
                abbreviation_span,
                display_name,
                kind,
                follow_target: None,
                soundfont: soundfont.unwrap_or_default(),
                volume: volume.unwrap_or(DEFAULT_PART_VOLUME),
                octave_offset: octave_offset.unwrap_or(DEFAULT_PART_OCTAVE_OFFSET),
            }),
            RawKind::Lyrics {
                target,
                target_span,
            } => {
                let declaration = resolve_lyrics_declaration(
                    &declarations,
                    LyricsDeclarationHeader {
                        abbreviation,
                        abbreviation_span,
                        display_name,
                    },
                    &target,
                    target_span,
                    errors,
                );
                declarations.extend(declaration);
            }
        }
    }
    lyric_parts_after_their_targets(declarations)
}

/// The name fields of a `lyrics[X]` declaration, before its target is resolved.
struct LyricsDeclarationHeader {
    abbreviation: String,
    abbreviation_span: Span,
    display_name: String,
}

/// Resolves a `lyrics[target]` declaration against the parts declared so far,
/// reporting an error (and yielding nothing) when `target` isn't an earlier
/// notes or chords part.
fn resolve_lyrics_declaration(
    declarations: &[PartDecl],
    header: LyricsDeclarationHeader,
    target: &str,
    target_span: Span,
    errors: &mut Vec<RecoverableError>,
) -> Option<PartDecl> {
    let target_part_index = declarations.iter().position(|declaration| {
        declaration.abbreviation == target
            && matches!(declaration.kind, PartKind::Notes | PartKind::Chords)
    });
    let Some(target_part_index) = target_part_index else {
        errors.push(RecoverableError::parts_lyrics_invalid_target(
            target_span,
            target,
        ));
        return None;
    };
    Some(PartDecl {
        abbreviation: header.abbreviation,
        abbreviation_span: header.abbreviation_span,
        display_name: header.display_name,
        kind: PartKind::Lyrics { target_part_index },
        follow_target: None,
        soundfont: Soundfont::default(),
        volume: DEFAULT_PART_VOLUME,
        octave_offset: DEFAULT_PART_OCTAVE_OFFSET,
    })
}

/// Reorders `declarations` so each lyric part sits directly after its target
/// (and after earlier lyric parts of the same target), keeping declaration
/// order otherwise, then re-points every `target_part_index` at the new order.
/// A lyric part's row is drawn directly under its target's, so part order is
/// row order.
fn lyric_parts_after_their_targets(declarations: Vec<PartDecl>) -> Vec<PartDecl> {
    let lyric_target = |declaration: &PartDecl| match declaration.kind {
        PartKind::Lyrics { target_part_index } => Some(target_part_index),
        _ => None,
    };
    let old_order: Vec<usize> = declarations
        .iter()
        .enumerate()
        .filter(|(_, declaration)| lyric_target(declaration).is_none())
        .flat_map(|(index, _)| {
            std::iter::once(index).chain(
                declarations
                    .iter()
                    .enumerate()
                    .filter(move |(_, declaration)| lyric_target(declaration) == Some(index))
                    .map(|(lyric_index, _)| lyric_index),
            )
        })
        .collect();
    let new_index_of = |old_index: usize| {
        old_order
            .iter()
            .position(|&candidate| candidate == old_index)
            .unwrap_or(old_index)
    };
    let mut slots: Vec<Option<PartDecl>> = declarations.into_iter().map(Some).collect();
    old_order
        .iter()
        .filter_map(|&old_index| slots.get_mut(old_index).and_then(Option::take))
        .map(|mut declaration| {
            if let PartKind::Lyrics { target_part_index } = &mut declaration.kind {
                *target_part_index = new_index_of(*target_part_index);
            }
            declaration
        })
        .collect()
}
