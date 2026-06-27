use crate::error::{Diagnostic, RecoverableError, Span, Spanned, Warning};

#[derive(Debug)]
pub enum ParsedMeasureSlot {
    EmptyNote { span: Span },
    Real { events: Vec<Spanned<ScoreEvent>> },
}

#[derive(Debug)]
pub struct ParsedLyrics {
    /// One syllable vec per measure, in score order. Empty inner vec = `_` (no lyrics).
    pub measure_syllables: Vec<Vec<Syllable>>,
    /// Byte offset of the start of the lyrics line for each measure, in order.
    pub measure_starts: Vec<usize>,
    /// Byte offset of the end of the lyrics line for each measure, in order.
    /// Used to extend the measure's source span to cover the lyrics line.
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
    pub display_name: String,
    pub kind: PartKind,
    pub follow_target: Option<String>,
    pub soundfont: Soundfont,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PartKind {
    Chords,
    Notes,
    NotesWithLyrics,
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
            PartKind::NotesWithLyrics => &[ScoreLineRole::Notes, ScoreLineRole::Lyrics],
        }
    }
}

pub fn flatten_score_line_slots(declarations: &[PartDecl]) -> Vec<ScoreLineSlot> {
    let mut slots = Vec::new();
    for (track_index, decl) in declarations.iter().enumerate() {
        for &role in decl.score_line_roles() {
            slots.push(ScoreLineSlot { track_index, role });
        }
    }
    slots
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
    pub measure_slots: Vec<ParsedMeasureSlot>,
    pub lyrics: Option<ParsedLyrics>,
    /// Per-measure beat-overflow error (None = no overflow for that measure).
    pub per_measure_beat_errors: Vec<Option<Warning>>,
    /// Per-measure grouping diagnostics: dotted-eighth errors (RecoverableError) and
    /// half-bar-boundary warnings (Warning), mixed as Diagnostic.
    pub per_measure_dotted_eighth_errors: Vec<Vec<Diagnostic>>,
    /// Per-measure dash-after-rest errors from suffix dashes on rests during token parse.
    pub per_measure_dash_after_rest_errors: Vec<Option<RecoverableError>>,
    /// Per-measure recoverable chord parse diagnostics (empty = no violations for that measure).
    pub per_measure_chord_errors: Vec<Vec<Diagnostic>>,
    /// Per-measure recoverable lex error from an unexpected character on the notes line.
    pub per_measure_lex_errors: Vec<Option<RecoverableError>>,
    /// Per-measure recoverable error on the lyrics line (e.g. empty lyrics line).
    pub per_measure_lyrics_errors: Vec<Option<RecoverableError>>,
}

#[derive(Debug)]
pub struct ParsedDocument {
    pub metadata: ParsedMetadata,
    pub declarations: Vec<PartDecl>,
    pub tracks: Vec<ParsedTrack>,
    pub directive_events_per_measure: Vec<Vec<Spanned<ScoreEvent>>>,
    /// Per-measure recoverable errors from desugaring (e.g. missing lyrics line).
    pub per_measure_parse_errors: Vec<Option<RecoverableError>>,
    /// Recoverable errors from parsing the [metadata] section.
    pub metadata_parse_errors: Vec<RecoverableError>,
    /// Recoverable errors from parsing the [parts] section.
    pub parts_parse_errors: Vec<RecoverableError>,
    /// Recoverable errors from section structure validation (unknown/duplicate/missing/out-of-order sections).
    pub section_structure_errors: Vec<RecoverableError>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TriadQuality {
    Major,
    Minor,
    Augmented,
    Diminished,
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

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedChordSymbol {
    pub degree: JianPuPitch,
    pub accidental: Accidental,
    pub triad: TriadQuality,
    pub extension: Option<Extension>,
    pub bass: Option<BassDegree>,
}

#[derive(Debug)]
pub struct ParsedMetadata {
    pub title: String,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub row_height: Option<u32>,
    pub max_columns: Option<u32>,
    pub label_width: Option<u32>,
    pub note_number_width: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScoreEvent {
    Note(ParsedNote),
    Chord(ParsedChordNote),
    Rest(ParsedRest),
    BpmChange(u32),
    KeyChange(KeyChange),
    TimeSignatureChange {
        numerator: u8,
        denominator: u8,
    },
    /// Internal or explicit padding: extends the previous note by one full beat (4 quarter-beats).
    Extension,
    /// Legacy tie marker retained for lyric-slot counting paths; use `(…)` groups in input.
    TieMarker,
    LabelChange(String),
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
    /// Whether `~` appeared after the octave modifier, requesting a tie to the next note.
    pub tie_to_next: bool,
    /// Number of nested `(…)` groups this note belongs to.
    pub group_membership: u8,
    /// Number of those groups that continue past this note.
    pub group_continuation: u8,
    /// Whether `.` was present as a dotted-note suffix.
    pub dotted: bool,
    /// When the slur group closes on an extension within this note (e.g. `(5 -)`),
    /// this holds the offset in quarter-beats from the note's start where the slur arc
    /// should end. `None` means the slur closes at the note's head position (normal case).
    pub slur_group_close_at_duration: Option<u32>,
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
    pub group_membership: u8,
    pub group_continuation: u8,
    pub dotted: bool,
    pub slur_group_close_at_duration: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedRest {
    /// Duration in quarter-beats. For dotted rests this already includes the added half-value.
    pub duration: u32,
    /// Whether `.` was present as a dotted-rest suffix.
    pub dotted: bool,
    pub group_membership: u8,
    pub group_continuation: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JianPuPitch {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyChange {
    pub note: Note,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub name: NoteName,
    pub octave: u8,
    pub accidental: Accidental,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NoteName {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Accidental {
    Flat,
    Sharp,
    Natural,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Syllable {
    pub text: String,
    /// True if `-` follows this syllable in the lyrics section.
    pub held: bool,
}
