use super::depth_event::{annotate_slur_close_via_extension, DepthEvent};
use super::duration::parse_duration_suffixes;
use super::groups::{
    apply_closed_group_depth, apply_open_group_depth, validate_group_note_count, GroupStack,
};
use super::timed_lexer::TimedLexToken;
use super::{ParseHeadError, TimedUnitHead};
use crate::ast::parsed::ScoreEvent;
use crate::error::{
    Diagnostic, IrrecoverableError, IrrecoverableErrorKind, RecoverableError, Span, Spanned,
};

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

pub struct TimedRecursiveDescentParser<'a, H: TimedUnitHead> {
    source: &'a str,
    base_offset: usize,
    tokens: &'a [Spanned<TimedLexToken>],
    pos: usize,
    stack: &'a mut GroupStack,
    /// Staging area: events with their pending depth accumulators.
    staging: Vec<DepthEvent>,
    dash_after_rest_error: Option<RecoverableError>,
    chord_errors: Vec<Diagnostic>,
    head: std::marker::PhantomData<H>,
}

type TimedLineParseResult = (
    Vec<Spanned<ScoreEvent>>,
    Option<RecoverableError>,
    Vec<Diagnostic>,
);

impl<'a, H: TimedUnitHead> TimedRecursiveDescentParser<'a, H> {
    pub fn parse_line(
        source: &'a str,
        base_offset: usize,
        tokens: &'a [Spanned<TimedLexToken>],
        stack: &'a mut GroupStack,
    ) -> Result<TimedLineParseResult, IrrecoverableError> {
        // Frames carried over from a previous bar have segment_start values that
        // refer to the old staging vec.  Reset them to 0 so they cover all events
        // produced in this new call.
        for frame in stack.frames.iter_mut() {
            frame.segment_start = 0;
        }

        let mut parser = Self {
            source,
            base_offset,
            tokens,
            pos: 0,
            stack,
            staging: Vec::new(),
            dash_after_rest_error: None,
            chord_errors: Vec::new(),
            head: std::marker::PhantomData,
        };
        std::hint::black_box(parser.head);
        parser.parse_atoms(false)?;
        parser.finalize_open_frames()?;
        let events = parser
            .staging
            .into_iter()
            .map(|d| d.into_spanned())
            .collect();
        Ok((events, parser.dash_after_rest_error, parser.chord_errors))
    }

    // -----------------------------------------------------------------------
    // Token stream helpers
    // -----------------------------------------------------------------------

    fn peek(&self) -> Option<&TimedLexToken> {
        self.tokens.get(self.pos).map(|s| &s.value)
    }

    fn peek_span(&self) -> Option<&Span> {
        self.tokens.get(self.pos).map(|s| &s.span)
    }

    fn bump(&mut self) -> Option<&Spanned<TimedLexToken>> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn current_span(&self) -> Span {
        self.peek_span()
            .cloned()
            .unwrap_or_else(|| Span::new(self.base_offset, self.base_offset))
    }

    // -----------------------------------------------------------------------
    // Core recursive methods
    // -----------------------------------------------------------------------

    fn parse_atoms(&mut self, stop_at_rparen: bool) -> Result<(), IrrecoverableError> {
        loop {
            match self.peek() {
                None => return Ok(()),
                Some(TimedLexToken::RParen) => {
                    if stop_at_rparen {
                        return Ok(());
                    }
                    self.close_group()?;
                }
                Some(TimedLexToken::LParen) => {
                    self.open_group()?;
                }
                Some(TimedLexToken::Extension) => {
                    let span = self.current_span();
                    self.bump();
                    self.staging
                        .push(DepthEvent::new(Spanned::new(ScoreEvent::Extension, span)));
                }
                Some(TimedLexToken::Tilde) => {
                    self.parse_tilde()?;
                }
                Some(TimedLexToken::HeadStart { offset }) => {
                    let offset = *offset;
                    self.parse_timed_unit(offset)?;
                }
                Some(TimedLexToken::Repeat { offset }) => {
                    let offset = *offset;
                    self.parse_repeat_unit(offset)?;
                }
                Some(TimedLexToken::Bpm(bpm)) => {
                    let bpm = *bpm;
                    let span = self.current_span();
                    self.bump();
                    self.staging.push(DepthEvent::new(Spanned::new(
                        ScoreEvent::BpmChange(bpm),
                        span,
                    )));
                }
                Some(TimedLexToken::KeyChange(key)) => {
                    let key = key.clone();
                    let span = self.current_span();
                    self.bump();
                    self.staging.push(DepthEvent::new(Spanned::new(
                        ScoreEvent::KeyChange(key),
                        span,
                    )));
                }
                Some(TimedLexToken::TimeSignature { num, den }) => {
                    let numerator = *num;
                    let denominator = *den;
                    let span = self.current_span();
                    self.bump();
                    self.staging.push(DepthEvent::new(Spanned::new(
                        ScoreEvent::TimeSignatureChange {
                            numerator,
                            denominator,
                        },
                        span,
                    )));
                }
            }
        }
    }

    /// Handle a `~` token: it ties the most recently pushed event to the timed unit that
    /// immediately follows it (if any), then, unless that pair forms a note/chord tie (which
    /// carries its own arc via `tie_to_next_span`), applies closed-group depth to the pair.
    fn parse_tilde(&mut self) -> Result<(), IrrecoverableError> {
        if self.staging.is_empty() {
            self.bump();
            return Ok(());
        }
        let group_start = self.staging.len() - 1;
        let tilde_span = self.current_span();
        self.bump();
        if let Some(TimedLexToken::HeadStart { offset }) = self.peek() {
            let offset = *offset;
            self.parse_timed_unit(offset)?;
        }
        if let Some(slice) = self.staging.get_mut(group_start..) {
            // Notes and chords use tie_to_next_span (set by duration parser); applying
            // group depth here would create a spurious slur arc in addition to the tie arc.
            let is_note_tie = slice
                .iter()
                .any(|e| matches!(e.spanned.value, ScoreEvent::Note(_)));
            let is_chord_tie = slice
                .iter()
                .any(|e| matches!(e.spanned.value, ScoreEvent::Chord(_)));
            if is_note_tie || is_chord_tie {
                // A `~` glued directly after a repeat atom (`r`/`_`/`=`) never goes through
                // `parse_duration_suffixes` — that's where `tie_to_next_span` normally gets
                // recorded while scanning a note's own suffix characters — so the tied event
                // here may not carry a tie span yet. Fill it in from the `~` token itself.
                if let Some(tied) = slice.first_mut() {
                    match &mut tied.spanned.value {
                        ScoreEvent::Note(note) if note.tie_to_next_span.is_none() => {
                            note.tie_to_next_span = Some(tilde_span);
                        }
                        ScoreEvent::Chord(chord) if chord.tie_to_next_span.is_none() => {
                            chord.tie_to_next_span = Some(tilde_span);
                        }
                        _ => {}
                    }
                }
            } else {
                apply_closed_group_depth(slice);
            }
        }
        Ok(())
    }

    /// Collect the recoverable diagnostics attached to a parsed `DurationParse` into
    /// `self.chord_errors` / `self.dash_after_rest_error`.
    fn collect_duration_suffix_diagnostics(&mut self, duration_meta: &super::DurationParse) {
        if let Some(error) = duration_meta.unexpected_char_error.clone() {
            self.chord_errors.push(Diagnostic::Error(error));
        }
        if let Some(error) = duration_meta.mixed_octave_markers_error.clone() {
            self.chord_errors.push(Diagnostic::Error(error));
        }
        if let Some(error) = duration_meta.cannot_dot_quarter_beat_error.clone() {
            self.chord_errors.push(Diagnostic::Error(error));
        }
        if duration_meta.dash_after_rest_error.is_some() && self.dash_after_rest_error.is_none() {
            self.dash_after_rest_error = duration_meta.dash_after_rest_error.clone();
        }
        if let Some(error) = duration_meta.tie_on_rest_error.clone() {
            self.chord_errors.push(Diagnostic::Error(error));
        }
    }

    /// Parse one timed unit (note/rest/chord head + duration suffixes) starting at `digit_offset`
    /// (which is an absolute byte offset into `self.source`).
    fn parse_timed_unit(&mut self, digit_offset: usize) -> Result<(), IrrecoverableError> {
        // Relative byte offset from the start of `source`.
        let rel = digit_offset - self.base_offset;

        // Slice from the head offset to the end of the current non-whitespace word.
        // Duration suffixes are never whitespace, so the unit ends at the first whitespace char.
        let raw_text = self.source.get(rel..).unwrap_or_default();
        let text = raw_text
            .find(|c: char| c.is_whitespace() || c == '(' || c == ')')
            .map(|ws_pos| &raw_text[..ws_pos])
            .unwrap_or(raw_text);

        let chars: Vec<char> = text.chars().collect();

        // Build a span that covers the single head character (will be refined after suffixes).
        let head_span = Span::new(digit_offset, digit_offset + 1);

        // Parse the head (note digit / chord symbol).
        let (head, head_end, is_rest, head_errors) = match H::parse_head(&chars, 0, &head_span) {
            Ok(parsed) => parsed,
            Err(ParseHeadError::Recoverable(maybe_diagnostic)) => {
                if let Some(diagnostic) = maybe_diagnostic {
                    self.chord_errors.push(diagnostic);
                }
                self.bump();
                self.skip_head_starts_before(self.unit_end_abs(digit_offset));
                return Ok(());
            }
            Err(ParseHeadError::Irrecoverable(error)) => return Err(error),
        };
        self.chord_errors.extend(head_errors);

        // Parse duration suffixes.
        let duration_meta = parse_duration_suffixes::<H>(&chars, 0, head_end, is_rest, &head_span)?;
        self.collect_duration_suffix_diagnostics(&duration_meta);

        let octave = if duration_meta.octave_up > 0 {
            duration_meta.octave_up
        } else {
            -duration_meta.octave_down
        };

        // Calculate the actual byte length covered by this unit.
        let unit_byte_len: usize = chars
            .get(..duration_meta.next_index)
            .unwrap_or_default()
            .iter()
            .map(|c| c.len_utf8())
            .sum();
        let unit_end_abs = digit_offset + unit_byte_len;
        let unit_span = Span::new(digit_offset, unit_end_abs);

        let mut event = H::to_event(
            &head,
            duration_meta.duration,
            duration_meta.dotted,
            octave,
            0,
            0,
        );
        if let Some(tie_span) = duration_meta.tie_to_next_span {
            if let ScoreEvent::Note(ref mut note) = event {
                note.tie_to_next_span = Some(tie_span);
            }
            if let ScoreEvent::Chord(ref mut chord) = event {
                chord.tie_to_next_span = Some(tie_span);
            }
        }
        if matches!(event, ScoreEvent::Note(_) | ScoreEvent::Chord(_)) {
            self.stack.last_pitched_event = Some(event.clone());
        }

        self.staging
            .push(DepthEvent::new(Spanned::new(event, unit_span)));

        // Increment note count in the innermost open group frame.
        self.stack.increment_note_count();

        // Consume the HeadStart token for this unit.
        self.bump();

        // Skip any HeadStart tokens that fall within the byte range of the unit we just parsed.
        // This happens when the lexer emits a HeadStart for a digit inside a multi-char symbol
        // (e.g. the `7` in chord `1m7`).
        while let Some(TimedLexToken::HeadStart { offset }) = self.peek() {
            if *offset < unit_end_abs {
                self.bump();
            } else {
                break;
            }
        }

        Ok(())
    }

    /// Parse a repeat atom (`r`, bare `_`, or bare `=`) starting at `offset`, which repeats
    /// the last pitched note/chord (skipping rests) as a fresh, standalone attack.
    fn parse_repeat_unit(&mut self, offset: usize) -> Result<(), IrrecoverableError> {
        let rel = offset - self.base_offset;
        let ch = self.source[rel..].chars().next().unwrap_or('r');
        let len = ch.len_utf8();
        let span = Span::new(offset, offset + len);
        let duration = match ch {
            'r' => 4,
            '_' => 2,
            _ => 1, // '='
        };

        let Some(mut event) = self.stack.last_pitched_event.clone() else {
            self.chord_errors
                .push(Diagnostic::Error(RecoverableError::repeat_no_prior_note(
                    span,
                )));
            self.bump();
            return Ok(());
        };

        match &mut event {
            ScoreEvent::Note(note) => {
                note.duration = duration;
                note.dotted = false;
                note.tie_to_next_span = None;
                note.group_membership = 0;
                note.group_continuation = 0;
                note.slur = false;
                note.slur_group_close_at_duration = None;
            }
            ScoreEvent::Chord(chord) => {
                chord.duration = duration;
                chord.dotted = false;
                chord.tie_to_next_span = None;
                chord.group_membership = 0;
                chord.group_continuation = 0;
                chord.slur = false;
                chord.slur_group_close_at_duration = None;
            }
            _ => {}
        }

        self.stack.last_pitched_event = Some(event.clone());
        self.staging
            .push(DepthEvent::new(Spanned::new(event, span)));
        self.stack.increment_note_count();
        self.bump();

        Ok(())
    }

    fn unit_end_abs(&self, digit_offset: usize) -> usize {
        let rel = digit_offset.saturating_sub(self.base_offset);
        let raw_text = self.source.get(rel..).unwrap_or("");
        raw_text
            .find(|c: char| c.is_whitespace() || c == '(' || c == ')')
            .map(|ws_pos| digit_offset + ws_pos)
            .unwrap_or_else(|| self.base_offset + self.source.len())
    }

    fn skip_head_starts_before(&mut self, unit_end_abs: usize) {
        while let Some(TimedLexToken::HeadStart { offset }) = self.peek() {
            if *offset < unit_end_abs {
                self.bump();
            } else {
                break;
            }
        }
    }

    /// Handle `(` — push a new frame and recurse into the inner atom sequence.
    fn open_group(&mut self) -> Result<(), IrrecoverableError> {
        self.bump(); // consume LParen

        let segment_start = self.staging.len();
        self.stack.push(segment_start);

        // Parse inner atoms until `)` or end of token stream.
        self.parse_atoms(true)?;

        // Now we should see `)` or end of stream.
        match self.peek() {
            Some(TimedLexToken::RParen) => {
                // Closed group — consume and apply closed-group depth.
                let rparen_span = self.current_span();
                self.bump();

                let frame = self.stack.pop().ok_or_else(|| {
                    IrrecoverableError::new(IrrecoverableErrorKind::internal_invariant(
                        rparen_span,
                        "open_group: stack empty after push",
                    ))
                })?;

                let note_count = frame.note_count;
                if let Some(warning) = validate_group_note_count(note_count, &rparen_span) {
                    self.chord_errors.push(Diagnostic::Warning(warning));
                } else if let Some(slice) = self.staging.get_mut(frame.segment_start..) {
                    apply_closed_group_depth(slice);
                }
            }
            _ => {
                // No closing paren — treat as an open (cross-line) group: apply open depth.
                // The frame stays on the stack for `finalize_open_frames`.
            }
        }

        Ok(())
    }

    /// Handle `)` when encountered outside of `parse_atoms(stop_at_rparen=true)`.
    /// This closes the innermost frame that was left open from a previous call.
    fn close_group(&mut self) -> Result<(), IrrecoverableError> {
        let rparen_span = self.current_span();
        self.bump(); // consume RParen

        let Some(frame) = self.stack.pop() else {
            self.chord_errors.push(Diagnostic::Error(RecoverableError {
                span: rparen_span,
                kind: crate::error::RecoverableErrorKind::GroupUnexpectedCloseParen,
            }));
            return Ok(());
        };

        let note_count = frame.note_count;
        if let Some(warning) = validate_group_note_count(note_count, &rparen_span) {
            self.chord_errors.push(Diagnostic::Warning(warning));
        } else if let Some(slice) = self.staging.get_mut(frame.segment_start..) {
            apply_closed_group_depth(slice);
            annotate_slur_close_via_extension(slice);
        }

        Ok(())
    }

    /// At end-of-line, any frames still on the stack represent open (cross-line) groups.
    /// Apply open-group depth to the events that belong to those frames.
    fn finalize_open_frames(&mut self) -> Result<(), IrrecoverableError> {
        // Iterate from outermost to innermost (bottom of stack to top).
        for frame in &self.stack.frames {
            if let Some(slice) = self.staging.get_mut(frame.segment_start..) {
                apply_open_group_depth(slice);
            }
        }
        Ok(())
    }
}
