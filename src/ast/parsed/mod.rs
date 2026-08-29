use crate::error::{Diagnostic, RecoverableError, Span, Spanned, Warning};

mod pitch;
pub use pitch::*;

#[derive(Debug)]
pub enum ParsedMeasureSlot {
    EmptyNote { span: Span },
    Real { events: Vec<Spanned<ScoreEvent>> },
}

#[derive(Debug)]
pub struct ParsedLyrics {
    /// Measure -> verse -> syllables, in score order. Consecutive `[Part]` lyric
    /// lines after the notes line become verses 1..N; an empty inner vec = `_`
    /// (no lyrics) for that verse in that measure.
    pub measure_syllables: Vec<Vec<Vec<Syllable>>>,
    /// Byte offset of the start of the lyrics block (spanning all its verses)
    /// for each measure, in order.
    pub measure_starts: Vec<usize>,
    /// Byte offset of the end of the lyrics block (spanning all its verses)
    /// for each measure, in order. Used to extend the measure's source span to
    /// cover the lyrics lines.
    pub measure_ends: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Soundfont(pub u8);

impl Default for Soundfont {
    fn default() -> Self {
        Self(52) // Choir Aahs
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PartDecl {
    pub abbreviation: String,
    /// Byte span of the abbreviation token on its `# parts` declaration line,
    /// used by rename-symbol to locate the declaration site.
    pub abbreviation_span: Span,
    pub display_name: String,
    pub kind: PartKind,
    pub follow_target: Option<String>,
    pub soundfont: Soundfont,
    pub volume: u8,
    /// MIDI-only octave shift applied to every note in this part (−4..=+4).
    pub octave_offset: i8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PartKind {
    Chords,
    Notes,
    Percussion,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScoreLineRole {
    Chord,
    Notes,
    Lyrics,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScoreLineSlot {
    pub track_index: usize,
    pub role: ScoreLineRole,
}

impl PartDecl {
    pub fn score_line_roles(&self) -> &'static [ScoreLineRole] {
        match self.kind {
            PartKind::Chords => &[ScoreLineRole::Chord],
            PartKind::Notes => &[ScoreLineRole::Notes],
            PartKind::Percussion => &[ScoreLineRole::Notes],
        }
    }
}

#[derive(Debug)]
pub enum ParsedTrack {
    Timed(ParsedTimedTrack),
}

#[derive(Debug)]
pub struct ParsedTimedTrack {
    pub abbreviation: String,
    pub display_name: String,
    pub kind: PartKind,
    pub soundfont: Soundfont,
    pub volume: u8,
    pub octave_offset: i8,
    pub measure_slots: Vec<ParsedMeasureSlot>,
    pub lyrics: Option<ParsedLyrics>,
    /// Per-measure beat-overflow error (None = no overflow for that measure).
    pub per_measure_beat_errors: Vec<Option<Warning>>,
    /// Per-measure grouping diagnostics: dotted-eighth errors (RecoverableError) and
    /// half-bar-boundary warnings (Warning), mixed as Diagnostic.
    pub per_measure_dotted_eighth_errors: Vec<Vec<Diagnostic>>,
    /// Per-measure recoverable chord parse diagnostics (empty = no violations for that measure).
    pub per_measure_chord_errors: Vec<Vec<Diagnostic>>,
    /// Per-measure recoverable lex error from an unexpected character on the notes line.
    pub per_measure_lex_errors: Vec<Option<RecoverableError>>,
    /// Per-measure recoverable error on the lyrics line (e.g. empty lyrics line).
    pub per_measure_lyrics_errors: Vec<Option<RecoverableError>>,
}

/// One `[Abbrev]` key-prefix reference to a part or group abbreviation in the
/// `# score` section, with the abbreviation text's own byte span (excluding
/// the surrounding brackets and whitespace). Used by rename-symbol to locate
/// this reference site; kept separate from the wider bracket span used by
/// `RecoverableError::part_key_unknown`, which must keep covering `[Abbrev]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbbreviationReference {
    pub abbreviation: String,
    pub span: Span,
}

#[derive(Debug)]
pub struct ParsedDocument {
    pub metadata: ParsedMetadata,
    pub declarations: Vec<PartDecl>,
    pub tracks: Vec<ParsedTrack>,
    pub directive_events_per_measure: Vec<Vec<Spanned<ScoreEvent>>>,
    /// Every `[Abbrev]` key-prefix reference found in the `# score` section,
    /// in file order. Used by rename-symbol to find all reference sites.
    pub abbreviation_references: Vec<AbbreviationReference>,
    /// Per-measure recoverable errors from desugaring (e.g. missing lyrics line).
    pub per_measure_parse_errors: Vec<Option<RecoverableError>>,
    /// Recoverable errors from parsing the [metadata] section.
    pub metadata_parse_errors: Vec<RecoverableError>,
    /// Recoverable errors from parsing the [parts] section.
    pub parts_parse_errors: Vec<RecoverableError>,
    /// Recoverable errors from section structure validation (unknown/duplicate/missing sections).
    pub section_structure_errors: Vec<RecoverableError>,
    /// The parsed `# sequence` section, if present: an ordered list of
    /// section-label references defining explicit playback order.
    pub sequence: Option<crate::parser::sequence_parser::SequenceSection>,
    /// Recoverable errors from parsing the `# sequence` section (e.g. empty entries).
    pub sequence_parse_errors: Vec<RecoverableError>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TriadQuality {
    Major,
    Minor,
    Augmented,
    Diminished,
    Sus2,
    Sus4,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Extension {
    DominantSeventh,
    MajorSeventh,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BassDegree {
    pub degree: JianPuPitch,
    pub accidental: Accidental,
}

/// Parsed (all-optional) form of the three-component style object a single
/// `<kind> = { font_size: N, horizontal_padding_pt: N, vertical_padding_pt: N }`
/// metadata line resolves to. See `crate::ast::grouped::TextStyle` for the
/// fully-resolved (defaulted) counterpart and `syntax.md` for the
/// object-literal grammar.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TextStyle {
    pub font_size: Option<u32>,
    pub horizontal_padding_pt: Option<u32>,
    pub vertical_padding_pt: Option<u32>,
}

#[derive(Debug, Default)]
pub struct ParsedMetadata {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub row_height: Option<u32>,
    pub max_measures_per_system: Option<u32>,
    pub note_number_width: Option<u32>,
    pub parts_list_columns: Option<u32>,
    /// Fixed width (points) of the part-label column at the start of each
    /// system, shared by every system in the score (see
    /// `Metadata::part_label_width_pt`). A flat scalar field, not part of
    /// `part_label_style`, since it's a layout constant rather than a text
    /// style component.
    pub part_label_width_pt: Option<u32>,
    /// Title text style.
    pub title_style: TextStyle,
    /// Subtitle text style.
    pub subtitle_style: TextStyle,
    /// Author text style.
    pub author_style: TextStyle,
    /// `# sequence` summary line text style.
    pub sequence_style: TextStyle,
    /// Part-name legend entry text style.
    pub part_legend_style: TextStyle,
    /// Measure bar-number text style.
    pub measure_number_style: TextStyle,
    /// Inline section-label text style.
    pub section_label_style: TextStyle,
    /// Page-number footer text style.
    pub page_number_style: TextStyle,
    /// Part row-label text style (see `Metadata::part_label`).
    pub part_label_style: TextStyle,
    /// Lyric syllable text style, including the click-target's extra vertical
    /// padding (formerly `lyric_click_target_padding_pt`).
    pub lyrics_style: TextStyle,
    /// Note head/rest/percussion-hit text style.
    pub notes_style: TextStyle,
    /// Chord symbol text style.
    pub chords_style: TextStyle,
    /// Note-dash (sustain-beat `-` extension) text style.
    pub note_dash_style: TextStyle,
    /// When `false`, disables merging of identical measure rows that come from different
    /// parts (see `consolidator::consolidate`). Default: `true`.
    pub merge_duplicate_measures_across_parts: Option<bool>,
    /// When `false`, an all-rest part is no longer omitted from a measure that has other
    /// parts with real content (see `compiler::compile_measure`). Default: `true`.
    pub hide_resting_parts: Option<bool>,
    /// When `true`, the horizontal divider line drawn between systems is omitted (see
    /// `grid_layout::layout`). Default: `false`.
    pub hide_system_dividers: Option<bool>,
    /// Translation in points applied to every rendered directive row (bar number, section
    /// label, key, bpm, time signature, nav markers), after layout (see
    /// `renderer::new_renderer::render_directive_line`). Not applied to the `# sequence`
    /// summary header. Default: `(0, 0)`.
    pub directive_row_offset: Option<Offset>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScoreEvent {
    Note(ParsedNote),
    Chord(ParsedChordNote),
    PercussionHit(ParsedPercussionHit),
    Rest(ParsedRest),
    BpmChange(u32),
    KeyChange(KeyChange),
    TimeSignatureChange {
        numerator: u8,
        denominator: u8,
    },
    /// Internal or explicit padding: extends the previous note by one beat — one full beat
    /// (4 quarter-beats), one dotted beat (6 quarter-beats, written `-.`) for compound
    /// meters, or one double-dotted beat (7 quarter-beats, written `-..`).
    Extension {
        dotted: bool,
        double_dotted: bool,
    },
    /// Legacy tie marker retained for lyric-slot counting paths; use `(…)` groups in input.
    TieMarker,
    LabelChange(String),
    /// `merge_duplicate_measures_across_parts=` — in effect from this measure onward
    /// until the next occurrence.
    MergeDuplicateMeasuresAcrossPartsChange(bool),
    /// `hide_resting_parts=` — in effect from this measure onward until the next
    /// occurrence.
    HideRestingPartsChange(bool),
    /// `break` — forces a new system to start at this measure. Applies only
    /// to the measure it's written on; does not persist to later measures.
    SystemBreak,
}

/// Tuplet ratio tag attached to a parsed note/chord/rest/percussion-hit that falls inside
/// an open `{N:...}`/`{N:M:...}` bracket: `num` notes take the time of `den` notes of the
/// same written value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TupletInfo {
    pub num: u32,
    pub den: u32,
    /// Identifies which `{...}` bracket this tag came from, distinguishing
    /// directly-adjacent brackets that share the same `num`/`den` ratio (e.g.
    /// `3:{3 6 1} 3:{3 6 1}`) so they don't merge into a single tuplet span/bracket.
    /// Unique per opened bracket within a line; not meaningful beyond identity/equality.
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedNote {
    pub pitch: JianPuPitch,
    pub accidental: Accidental,
    /// Octave offset from the default octave. 0 = default, positive = up, negative = down.
    pub octave: i8,
    /// Duration in quarter-beats. For dotted notes this already includes the added half-value.
    pub duration: u32,
    /// Whether this note is tied/slurred to the next note (from a `(…)` group).
    pub slur: bool,
    /// Source span of the `~` suffix when this note is tied to the next note.
    pub tie_to_next_span: Option<Span>,
    /// Number of nested `(…)` groups this note belongs to.
    pub group_membership: u8,
    /// Number of those groups that continue past this note.
    pub group_continuation: u8,
    /// Whether `.` was present as a dotted-note suffix.
    pub dotted: bool,
    /// Whether `..` was present as a double-dotted-note suffix. Only ever `true` when
    /// `dotted` is also `true`.
    pub double_dotted: bool,
    /// When the slur group closes on an extension within this note (e.g. `(5 -)`),
    /// this holds the offset in quarter-beats from the note's start where the slur arc
    /// should end. `None` means the slur closes at the note's head position (normal case).
    pub slur_group_close_at_duration: Option<u32>,
    /// The innermost `{...}` tuplet bracket this note belongs to, if any.
    pub tuplet: Option<TupletInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedChordNote {
    pub degree: JianPuPitch,
    pub accidental: Accidental,
    pub triad: TriadQuality,
    pub extension: Option<Extension>,
    pub bass: Option<BassDegree>,
    pub duration: u32,
    pub slur: bool,
    pub tie_to_next_span: Option<Span>,
    pub group_membership: u8,
    pub group_continuation: u8,
    pub dotted: bool,
    pub double_dotted: bool,
    pub slur_group_close_at_duration: Option<u32>,
    /// The innermost `{...}` tuplet bracket this chord note belongs to, if any.
    pub tuplet: Option<TupletInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedPercussionHit {
    /// Duration in quarter-beats. For dotted hits this already includes the added half-value.
    pub duration: u32,
    /// Whether this hit is tied/slurred to the next hit (from a `(…)` group).
    pub slur: bool,
    /// Source span of the `~` suffix when this hit is tied to the next hit.
    pub tie_to_next_span: Option<Span>,
    /// Number of nested `(…)` groups this hit belongs to.
    pub group_membership: u8,
    /// Number of those groups that continue past this hit.
    pub group_continuation: u8,
    /// Whether `.` was present as a dotted-hit suffix.
    pub dotted: bool,
    /// Whether `..` was present as a double-dotted-hit suffix.
    pub double_dotted: bool,
    pub slur_group_close_at_duration: Option<u32>,
    /// The innermost `{...}` tuplet bracket this hit belongs to, if any.
    pub tuplet: Option<TupletInfo>,
}

impl ParsedPercussionHit {
    pub fn tie_to_next(&self) -> bool {
        self.tie_to_next_span.is_some()
    }
}

impl ParsedNote {
    pub fn tie_to_next(&self) -> bool {
        self.tie_to_next_span.is_some()
    }
}

impl ParsedChordNote {
    pub fn tie_to_next(&self) -> bool {
        self.tie_to_next_span.is_some()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedRest {
    /// Duration in quarter-beats. For dotted rests this already includes the added half-value.
    pub duration: u32,
    /// Whether `.` was present as a dotted-rest suffix.
    pub dotted: bool,
    /// Whether `..` was present as a double-dotted-rest suffix.
    pub double_dotted: bool,
    pub group_membership: u8,
    pub group_continuation: u8,
    /// The innermost `{...}` tuplet bracket this rest belongs to, if any.
    pub tuplet: Option<TupletInfo>,
    /// True when this rest was synthesized to fill a part not mentioned in
    /// this measure (see "Not-mentioned parts" in syntax.md), rather than
    /// written by the composer as an explicit `0`. Rendered with a distinct
    /// glyph — see `render_rest` in `src/renderer/new_renderer/glyph_renderers.rs`.
    pub implicit_fill: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Syllable {
    pub text: String,
    /// True if `-` follows this syllable in the lyrics section.
    pub held: bool,
    /// Source byte range of this syllable's own token, absolute within the
    /// whole document — lets the SVG preview map a clicked/dragged lyric
    /// syllable back to its source text, mirroring how `ParsedNote::event_span`
    /// does the same for notes (see `note_spans.rs`).
    pub span: Span,
}
