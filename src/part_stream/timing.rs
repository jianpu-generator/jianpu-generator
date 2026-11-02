//! Beat accounting shared by extraction and splitting.

use itertools::Itertools;

use crate::ast::parsed::{PartKind, ScoreEvent, TupletInfo};
use crate::error::{IrrecoverableError, Span, Spanned};
use crate::parser::score::interleaved_parser::timed_beats;
use crate::parser::score::timed_parser::{lex_line, GroupStack, LexContext, TimedLexToken};
use crate::parser::score::token_parser::{
    parse_chord_line, parse_notes_line, parse_percussion_line, TimedLineParse,
};
use crate::tuplet::{apply_resolution_multiplier, resolution_multiplier_of, tuplet_of};

/// The line grammar a stream-editable part is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TimedLineKind {
    Notes,
    Chords,
    Percussion,
}

impl TimedLineKind {
    pub(super) fn of(kind: PartKind) -> Option<Self> {
        match kind {
            PartKind::Notes => Some(Self::Notes),
            PartKind::Chords => Some(Self::Chords),
            PartKind::Percussion => Some(Self::Percussion),
            PartKind::Lyrics { .. } => None,
        }
    }

    pub(super) fn parse(
        self,
        line: &str,
        stack: &mut GroupStack,
    ) -> Result<TimedLineParse, IrrecoverableError> {
        match self {
            Self::Notes => parse_notes_line(line, 0, stack),
            Self::Chords => parse_chord_line(line, 0, stack),
            Self::Percussion => parse_percussion_line(line, 0, stack),
        }
    }

    pub(super) fn lex(self, line: &str) -> Result<Vec<Spanned<TimedLexToken>>, IrrecoverableError> {
        let context = match self {
            Self::Notes => LexContext::Notes,
            Self::Chords => LexContext::Chords,
            Self::Percussion => LexContext::Percussion,
        };
        lex_line(line, 0, context).map(|(tokens, _)| tokens)
    }
}

/// A timed event plus the `-` extensions that lengthen it: the smallest unit
/// that can't be cut by a barline.
#[derive(Debug, Clone, Copy)]
pub(super) struct TimedCluster {
    pub(super) start: usize,
    pub(super) end: usize,
    /// Duration on the rescaled grid (quarter-beats × the line's resolution multiplier).
    pub(super) duration: u32,
    pub(super) tuplet: Option<TupletInfo>,
    pub(super) has_extension: bool,
}

pub(super) struct TimedLayout {
    pub(super) clusters: Vec<TimedCluster>,
    /// Every duration in `clusters` is this many times its quarter-beat length.
    pub(super) multiplier: u32,
    /// `-` extensions with no note before them to extend; they count no beats.
    pub(super) orphan_extensions: Vec<Span>,
}

/// Groups `events` into clusters with durations on the line's rescaled grid.
pub(super) fn timed_layout(events: &[Spanned<ScoreEvent>]) -> TimedLayout {
    let multiplier = resolution_multiplier_of(events);
    let rescaled = apply_resolution_multiplier(events.to_vec(), multiplier);
    let mut clusters: Vec<TimedCluster> = Vec::new();
    let mut orphan_extensions = Vec::new();
    for event in &rescaled {
        let beats = timed_beats(&event.value);
        match event.value {
            ScoreEvent::Extension { .. } => {
                if let Some(last) = clusters.last_mut() {
                    last.duration += beats * multiplier;
                    last.end = event.span.end;
                    last.has_extension = true;
                } else {
                    orphan_extensions.push(event.span);
                }
            }
            _ if beats > 0 => clusters.push(TimedCluster {
                start: event.span.start,
                end: event.span.end,
                duration: beats,
                tuplet: tuplet_of(&event.value),
                has_extension: false,
            }),
            _ => {}
        }
    }
    TimedLayout {
        clusters,
        multiplier,
        orphan_extensions,
    }
}

/// A directive written inside a part line instead of on the measure's directive line.
pub(super) fn is_inline_directive(event: &ScoreEvent) -> bool {
    matches!(
        event,
        ScoreEvent::BpmChange(_)
            | ScoreEvent::KeyChange(_)
            | ScoreEvent::TimeSignatureChange { .. }
            | ScoreEvent::LabelChange(_)
            | ScoreEvent::MergeDuplicateMeasuresAcrossPartsChange(_)
            | ScoreEvent::HideRestingPartsChange(_)
            | ScoreEvent::SystemBreak
    )
}

/// Rests filling `quarter_beats`: whole-beat `0`s, then `0_` / `0=` for the rest.
pub(super) fn rest_fill(quarter_beats: u32) -> String {
    let whole_beats = std::iter::repeat_n("0", (quarter_beats / 4) as usize);
    let remainder = match quarter_beats % 4 {
        3 => vec!["0_", "0="],
        2 => vec!["0_"],
        1 => vec!["0="],
        _ => vec![],
    };
    whole_beats.chain(remainder).join(" ")
}

/// Collapses every whitespace run (including newlines) into one space.
pub(super) fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().join(" ")
}
