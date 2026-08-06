//! Unzipped Edit: extract every declared part's score lines as one
//! whole-document, per-part-flattened editable text block, and merge edits to
//! that block back into the full `# score` section.
//!
//! [`merge_unzipped_text`] re-bars each part's flat token stream into
//! measures, then reassembles the `# score` section and runs one final
//! `desugar::desugar_groups` pass so the existing implicit-rest-fill
//! machinery produces correct rest tokens for parts that come up short.
//! Notes/Chords/Percussion (and the Notes half of `notes+lyrics`) re-bar by
//! greedily filling against each original measure's real, intrinsic beat
//! capacity; every Lyrics-role occurrence instead re-bars by diffing the
//! edited text against its own *original* per-measure tokens, since a
//! verse's per-measure token count has no intrinsic capacity of its own to
//! recompute (see [`repack`]'s doc comment for why). See the "Phase 3"
//! write-up this was implemented from for the full algorithm; do not
//! "simplify" the repack/reconcile arithmetic without re-reading it.
//!
//! **Multi-verse lyrics.** A `notes+lyrics` part's notes line and every verse
//! line share the same `track_index` (they're distinguished only by
//! `ScoreLineSlot::role`), and a standalone `lyrics` part's verses similarly
//! share one `track_index`. `[Abbrev]` always means slot occurrence 0 of that
//! part's first static role (Notes for `notes+lyrics`, Lyrics — i.e. verse 1
//! — for standalone `lyrics`); every additional Lyrics-role occurrence is its
//! own tagged block, `[Abbrev:lyrics:N]` (1-based verse number). Verse
//! assignment on disk is strictly positional per measure (see
//! `desugar::roles_for_group`): a measure can't have verse 3 without verses 1
//! and 2, so merge-back's reassembly force-fills any lower verse that's
//! implicitly missing with `desugar::implicit_fill(Lyrics, ..)` (`"_"`)
//! whenever a higher verse has real content in that same measure.
//!
//! This algorithm is split across submodules: [`capacity`] scans the
//! original document for per-measure beat capacities and (for Lyrics-role
//! content) original per-measure tokens, [`diff`] is the domain-free token
//! diff Lyrics-role repack is built on, [`repack`] implements both repack
//! mechanisms, [`extract`] implements [`extract_unzipped_text`], [`merge`]
//! implements [`merge_unzipped_text`], and [`format`] implements
//! [`format_unzipped_text`] (a readability-only pass built on the other two).
//! Shared setup (parsing `# parts`/`# groups`) and a few small helpers used
//! by more than one submodule live here.

use crate::ast::parsed::{PartDecl, ScoreEvent, ScoreLineRole, ScoreLineSlot};
use crate::desugar::SourceLine;
use crate::error::{Span, Spanned};
use crate::parser::{self, group_parser::ResolvedGroup};

mod capacity;
mod diff;
mod extract;
mod format;
mod merge;
mod parse;
mod reassemble;
mod repack;

pub use capacity::{scan_measure_capacities, scan_measure_token_counts};
pub use extract::extract_unzipped_text;
pub use format::format_unzipped_text;
pub use merge::merge_unzipped_text;

/// One declared part's tagged verse blocks (`[Abbrev:lyrics:N]`), in verse order.
#[derive(Debug)]
pub struct LyricsVerseRanges {
    /// 1-based verse number.
    pub verse_number: usize,
    /// Per original measure index: the byte range `[start, end)` within
    /// [`UnzippedExtractOutput::text`] covering that measure's tokens.
    pub measure_ranges: Vec<(usize, usize)>,
}

/// Result of [`extract_unzipped_text`].
#[derive(Debug)]
pub struct UnzippedExtractOutput {
    pub text: String,
    /// Per declared part (declaration order), per measure index: the byte
    /// range `[start, end)` within `text` covering that measure's tokens, for
    /// the part's primary (`[Abbrev]`) block.
    pub part_measure_ranges: Vec<Vec<(usize, usize)>>,
    /// Per declared part (declaration order): that part's tagged
    /// `[Abbrev:lyrics:N]` verse blocks, in verse order. Empty for any part
    /// that isn't `NotesWithLyrics`/`Lyrics`-kind.
    pub lyrics_verse_ranges: Vec<Vec<LyricsVerseRanges>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnzippedEditError {
    /// A block header in the unzipped text names an abbreviation that isn't
    /// declared in `# parts`.
    UnknownPart,
    /// The first non-blank line of a paragraph in the unzipped text doesn't
    /// match the `^\[(\w+)(:lyrics:\d+)?\]\s*$` header shape at all (e.g.
    /// missing brackets, stray trailing text after `]`, a malformed verse
    /// tag). Distinct from `UnknownPart`, which is for a well-formed header
    /// naming an undeclared part.
    MalformedHeader,
    /// A `[Abbrev:lyrics:N]` header names a declared part whose kind is
    /// neither `NotesWithLyrics` nor `Lyrics` (no Lyrics-role slot to target).
    UnexpectedLyricsBlock,
    ParseFailed,
}

/// A raw, not-yet-desugared score line paired with its byte offset within
/// its containing section, matching `measure_group::collect_groups`'s output.
type RawSourceLine = (String, usize);

struct UnzippedDocumentContext {
    declarations: Vec<PartDecl>,
    resolved_groups: Vec<ResolvedGroup>,
    score_content: String,
    score_offset: usize,
}

/// Shared setup for whole-document callers: parses `# parts`/`# groups` once
/// and returns everything needed to work with the `# score` section, without
/// requiring a target part.
fn resolve_document_context(source: &str) -> UnzippedDocumentContext {
    let (sections, _section_errors) = parser::load_document_sections(source);
    let (parts_content, parts_offset) = sections.parts;
    let (score_content, score_offset) = sections.score;

    let (declarations, _parts_errors) =
        parser::parts_parser::parse_parts(&parts_content, parts_offset, &[]);

    let resolved_groups = match sections.group {
        Some((group_content, group_offset)) => {
            let (group_section, _group_errors) =
                parser::group_parser::parse_group(&group_content, group_offset);
            match group_section {
                Some(group_section) => {
                    let (resolved, _errors) = parser::group_parser::resolve_and_validate_groups(
                        &group_section,
                        &declarations,
                    );
                    resolved
                }
                None => Vec::new(),
            }
        }
        None => Vec::new(),
    };

    UnzippedDocumentContext {
        declarations,
        resolved_groups,
        score_content,
        score_offset,
    }
}

/// This part's resolved content line within a single desugared measure
/// group, or an empty string if this `(role, occurrence)` slot doesn't exist
/// in the group at all.
///
/// `group`/`slots` are the parallel outputs of `desugar::desugar_groups` for
/// one measure group: `group` is `[directive_line?] ++ data_lines`, while
/// `slots` covers only `data_lines`, so `data_lines` starts at
/// `group.len() - slots.len()`. `occurrence` is 0-based among the slots
/// matching `(target_index, role)` — e.g. verse 2 of a Lyrics-role part is
/// `role = Lyrics, occurrence = 1`.
fn extract_part_line(
    group: &[SourceLine],
    slots: &[ScoreLineSlot],
    target_index: usize,
    role: ScoreLineRole,
    occurrence: usize,
) -> String {
    let data_start = group.len().saturating_sub(slots.len());
    group
        .get(data_start..)
        .unwrap_or(&[])
        .iter()
        .zip(slots.iter())
        .filter(|(_, slot)| slot.track_index == target_index && slot.role == role)
        .nth(occurrence)
        .map(|(line, _)| line.content.clone())
        .unwrap_or_default()
}

/// A measure's beat capacity by index, extending past `capacities`'s length
/// with its last entry (or `u32::MAX` if `capacities` is empty), matching the
/// "shift content, auto re-bar" growth behavior `merge::repack_into_measures`
/// relies on for the same reason.
fn capacity_at(capacities: &[u32], index: usize) -> u32 {
    capacities
        .get(index)
        .or_else(|| capacities.last())
        .copied()
        .unwrap_or(u32::MAX)
}

/// Folds `Extension`/`TieMarker` events into the previous cluster (they add
/// beats to whichever timed event preceded them rather than starting a new
/// one), mirroring `PartGrouper::handle_extension`
/// (`src/grouper/part_grouper.rs:180`) without depending on that later
/// pipeline stage's `NoteEvent` type. Directive `ScoreEvent` variants never
/// appear here (a per-part flat-text repack only ever contains the six timed
/// variants), so they're ignored rather than matched exhaustively.
fn fold_extensions(events: Vec<Spanned<ScoreEvent>>) -> Vec<(Span, u32)> {
    let mut clusters: Vec<(Span, u32)> = Vec::new();
    for spanned in events {
        match spanned.value {
            ScoreEvent::Note(note) => clusters.push((spanned.span, note.duration)),
            ScoreEvent::Chord(chord) => clusters.push((spanned.span, chord.duration)),
            ScoreEvent::PercussionHit(hit) => clusters.push((spanned.span, hit.duration)),
            ScoreEvent::Rest(rest) => clusters.push((spanned.span, rest.duration)),
            ScoreEvent::Extension { dotted } => {
                let beats = if dotted { 6 } else { 4 };
                if let Some(last) = clusters.last_mut() {
                    last.0.end = last.0.end.max(spanned.span.end);
                    last.1 += beats;
                }
            }
            ScoreEvent::TieMarker => {
                if let Some(last) = clusters.last_mut() {
                    last.0.end = last.0.end.max(spanned.span.end);
                }
            }
            _ => {}
        }
    }
    clusters
}

#[cfg(test)]
mod tests_capacity;
#[cfg(test)]
mod tests_extract;
#[cfg(test)]
mod tests_format;
#[cfg(test)]
mod tests_merge;
#[cfg(test)]
mod tests_quickcheck;
