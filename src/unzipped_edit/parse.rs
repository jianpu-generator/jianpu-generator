//! Parses Unzipped Edit text (as submitted by [`super::merge_unzipped_text`]
//! callers) into per-part blocks, ahead of `repack`'s measure re-barring.

use std::collections::HashMap;

use crate::ast::parsed::PartDecl;
use crate::ast::parsed::PartKind;

use super::UnzippedEditError;

/// A parsed unzipped-text block header: `[Abbrev]` (primary block, `verse_number: None`)
/// or `[Abbrev:lyrics:N]` (`verse_number: Some(N)`, `N >= 1`).
struct ParsedUnzippedHeader<'a> {
    abbreviation: &'a str,
    verse_number: Option<usize>,
}

/// Matches `^\[(\w+)(:lyrics:([1-9]\d*))?\]\s*$` (no leading/trailing content
/// besides the bracketed header) via manual `str` parsing, mirroring
/// `desugar::parse_key_prefix`'s style rather than pulling in a `regex`
/// dependency for one pattern.
fn parse_unzipped_header(line: &str) -> Option<ParsedUnzippedHeader<'_>> {
    let inner = line.strip_prefix('[')?;
    let close = inner.find(']')?;
    let (body, rest) = inner.split_at(close);
    let rest = &rest[1..]; // drop the ']'
    if !rest.trim().is_empty() {
        return None;
    }

    let mut segments = body.splitn(3, ':');
    let abbrev = segments.next()?;
    if abbrev.is_empty() || !abbrev.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }

    match (segments.next(), segments.next()) {
        (None, None) => Some(ParsedUnzippedHeader {
            abbreviation: abbrev,
            verse_number: None,
        }),
        (Some("lyrics"), Some(number)) => {
            let verse_number: usize = number.parse().ok()?;
            if verse_number == 0 {
                return None;
            }
            Some(ParsedUnzippedHeader {
                abbreviation: abbrev,
                verse_number: Some(verse_number),
            })
        }
        _ => None,
    }
}

/// Every declared part's parsed blocks from the unzipped text: primary
/// (`abbrev -> flat_text`) and tagged lyrics verses (`abbrev -> verse_number
/// -> flat_text`), with internal newlines within a block collapsed to single
/// spaces (newlines inside a block are purely cosmetic wrapping).
///
/// Splits `unzipped_text` into blank-line-delimited paragraphs the same way
/// `parser::score::measure_group::collect_groups` splits the `# score`
/// section into measure groups, but hand-rolled here on plain `&str` slices
/// (no byte-offset tracking needed, since unzipped text is never spliced
/// back verbatim — only re-tokenized) rather than reusing `collect_groups`
/// directly, which returns owned `(String, usize)` pairs keyed to source
/// byte offsets that Phase 3 has no use for.
pub(super) struct UnzippedBlocks {
    pub(super) primary: HashMap<String, String>,
    pub(super) lyrics_verses: HashMap<String, HashMap<usize, String>>,
}

pub(super) fn split_unzipped_blocks(
    unzipped_text: &str,
) -> Result<UnzippedBlocks, UnzippedEditError> {
    let mut paragraphs: Vec<Vec<&str>> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in unzipped_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !current.is_empty() {
                paragraphs.push(std::mem::take(&mut current));
            }
        } else {
            current.push(trimmed);
        }
    }
    if !current.is_empty() {
        paragraphs.push(current);
    }

    let mut primary = HashMap::with_capacity(paragraphs.len());
    let mut lyrics_verses: HashMap<String, HashMap<usize, String>> = HashMap::new();
    for paragraph in paragraphs {
        // Every pushed `paragraph` is non-empty by construction (see the loop
        // above); `split_first` still returns an `Option`, so skip rather
        // than index/panic on the unreachable empty case.
        let Some((header, body)) = paragraph.split_first() else {
            continue;
        };
        let parsed = parse_unzipped_header(header).ok_or(UnzippedEditError::MalformedHeader)?;
        let text = body.join(" ");
        match parsed.verse_number {
            None => {
                primary.insert(parsed.abbreviation.to_string(), text);
            }
            Some(verse_number) => {
                lyrics_verses
                    .entry(parsed.abbreviation.to_string())
                    .or_default()
                    .insert(verse_number, text);
            }
        }
    }
    Ok(UnzippedBlocks {
        primary,
        lyrics_verses,
    })
}

/// Validates that every abbreviation named by a unzipped-text block header
/// is declared, and that every tagged `[Abbrev:lyrics:N]` block names a part
/// whose kind actually has a Lyrics-role slot to target.
pub(super) fn validate_unzipped_blocks(
    blocks: &UnzippedBlocks,
    declarations: &[PartDecl],
) -> Result<(), UnzippedEditError> {
    for abbrev in blocks.primary.keys().chain(blocks.lyrics_verses.keys()) {
        if !declarations.iter().any(|decl| &decl.abbreviation == abbrev) {
            return Err(UnzippedEditError::UnknownPart);
        }
    }
    for abbrev in blocks.lyrics_verses.keys() {
        let Some(decl) = declarations
            .iter()
            .find(|decl| &decl.abbreviation == abbrev)
        else {
            return Err(UnzippedEditError::UnknownPart);
        };
        if !matches!(decl.kind, PartKind::NotesWithLyrics | PartKind::Lyrics) {
            return Err(UnzippedEditError::UnexpectedLyricsBlock);
        }
    }
    Ok(())
}
