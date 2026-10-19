//! Which parts can be stream-edited, and the grammar each one's stream uses.

use crate::ast::parsed::{PartDecl, PartKind};

use super::document::ScoreDocument;
use super::extract::extract_timed_chunks;
use super::lyrics::{extract_lyric_chunks, lyric_slot_counts, split_lyric_stream};
use super::split::{split_timed_stream, SplitStream};
use super::timing::{rest_fill, TimedLineKind};
use super::{join_chunks, PartStreamError, StreamDiagnosticSeverity, NO_LYRICS_MEASURE};

/// A notes / chords / percussion stream; a follow part's may also hold `@`.
#[derive(Debug, Clone, Copy)]
pub(super) struct TimedGrammar {
    pub(super) kind: TimedLineKind,
    pub(super) is_follow: bool,
}

pub(super) enum StreamGrammar {
    Timed {
        grammar: TimedGrammar,
    },
    /// A lyric part over a notes part: measure `m` takes `slot_counts[m]` syllables.
    Lyrics {
        slot_counts: Vec<usize>,
    },
}

impl StreamGrammar {
    pub(super) fn split(&self, stream: &str, document: &ScoreDocument) -> SplitStream {
        match self {
            Self::Timed { grammar } => split_timed_stream(stream, *grammar, &document.capacities()),
            Self::Lyrics { slot_counts } => split_lyric_stream(stream, slot_counts),
        }
    }

    pub(super) fn is_follow(&self) -> bool {
        matches!(self, Self::Timed { grammar } if grammar.is_follow)
    }

    /// What fills a measure the part must still write but has nothing for.
    pub(super) fn empty_measure(&self, capacity: u32) -> String {
        match self {
            Self::Timed { .. } => rest_fill(capacity),
            Self::Lyrics { .. } => NO_LYRICS_MEASURE.to_string(),
        }
    }
}

/// A part that passed every eligibility check, with its original chunks.
pub(super) struct EditablePart {
    pub(super) index: usize,
    pub(super) grammar: StreamGrammar,
    pub(super) chunks: Vec<String>,
}

pub(super) fn ineligible(reason: impl Into<String>) -> PartStreamError {
    PartStreamError::Ineligible {
        reason: reason.into(),
    }
}

/// Runs every eligibility check on the part at `index`, including a
/// self-check that its own extracted stream splits back into exactly the
/// same measure chunks without errors — the guarantee that applying an
/// untouched stream never changes the source.
pub(super) fn editable_part(
    document: &ScoreDocument,
    index: usize,
) -> Result<EditablePart, PartStreamError> {
    let declaration = document
        .declarations
        .get(index)
        .ok_or_else(|| ineligible("no such part"))?;
    let grammar = stream_grammar(document, declaration)?;
    let chunks = match &grammar {
        StreamGrammar::Timed { grammar } => extract_timed_chunks(document, index, *grammar),
        StreamGrammar::Lyrics { slot_counts } => extract_lyric_chunks(document, index, slot_counts),
    }
    .map_err(ineligible)?;
    let resplit = grammar.split(&join_chunks(&chunks).text, document);
    if let Some(error) = resplit
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == StreamDiagnosticSeverity::Error)
    {
        return Err(ineligible(format!(
            "its stream would not parse: {}",
            error.message
        )));
    }
    if let Some(measure) = (0..chunks.len().max(resplit.chunks.len()))
        .find(|&measure| chunks.get(measure) != resplit.chunks.get(measure))
    {
        return Err(ineligible(format!(
            "measure {} would not keep its content when re-barred",
            measure + 1
        )));
    }
    Ok(EditablePart {
        index,
        grammar,
        chunks,
    })
}

fn stream_grammar(
    document: &ScoreDocument,
    declaration: &PartDecl,
) -> Result<StreamGrammar, PartStreamError> {
    let PartKind::Lyrics { target_part_index } = declaration.kind else {
        let kind = TimedLineKind::of(declaration.kind)
            .ok_or_else(|| ineligible("this kind of part cannot be edited as a stream"))?;
        return Ok(StreamGrammar::Timed {
            grammar: TimedGrammar {
                kind,
                is_follow: declaration.follow_target.is_some(),
            },
        });
    };
    let target_kind = document
        .declarations
        .get(target_part_index)
        .map(|target| target.kind);
    match target_kind {
        Some(PartKind::Notes) => lyric_slot_counts(document, target_part_index)
            .map(|slot_counts| StreamGrammar::Lyrics { slot_counts })
            .map_err(ineligible),
        Some(PartKind::Chords) => Err(ineligible(
            "lyrics under a chords part cannot be edited as a stream",
        )),
        _ => Err(ineligible(
            "only lyrics under a notes part can be edited as a stream",
        )),
    }
}
