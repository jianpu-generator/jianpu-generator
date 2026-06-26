use crate::ast::parsed::{
    Accidental, BassDegree, Extension, JianPuPitch, KeyChange, Syllable, TriadQuality,
};
use crate::error::{Diagnostic, RecoverableError, Span, Warning};

// ── Public final types ────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Metadata {
    pub title: String,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    /// Row height in points. Controls font sizes, dot radii, and all vertical spacing. Default: 24.
    pub row_height: u32,
    /// Maximum logical columns per row before wrapping. Default: 28.
    pub max_columns: u32,
    /// Left margin reserved for part labels in points. Default: 40.
    pub label_width: u32,
    /// Estimated rendered width of a single digit note number (0–9) in points. Default: 8.
    pub note_number_width: u32,
}

#[derive(Clone)]
pub struct Notes {
    pub events: Vec<NoteEvent>,
}

#[derive(Clone)]
pub struct Lyrics {
    pub syllables: Vec<Syllable>,
}

#[derive(Clone)]
pub struct PartSlice {
    pub name: Option<String>,
    pub kind: crate::ast::parsed::PartKind,
    pub soundfont: crate::ast::parsed::Soundfont,
    pub notes: Notes,
    pub lyrics: Option<Lyrics>,
    /// True when this slice's source measure had at least one `Diagnostic::Error`.
    /// The compiler uses this to drop incoming cross-measure tie/slur arcs.
    pub has_error: bool,
}

#[derive(Clone)]
pub struct MultiPartMeasure {
    pub time_signature: Option<TimeSignature>,
    pub bpm: Option<u32>,
    // TODO: key-change rendering (1=X label) is not yet implemented in layout/renderer
    pub key: Option<KeyChange>,
    pub label: Option<String>,
    pub parts: Vec<PartRow>,
    /// Byte range of this measure's note events in the original source.
    /// Used to map editor cursor position to a measure index.
    pub source_span: Span,
    /// Diagnostics collected during grouping for this measure.
    /// Non-empty triggers a colored overlay in the SVG renderer.
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone)]
pub enum PartRow {
    Timed(PartSlice),
}

impl PartRow {
    pub fn name(&self) -> Option<&String> {
        self.slice().name.as_ref()
    }

    pub fn slice(&self) -> &PartSlice {
        match self {
            PartRow::Timed(s) => s,
        }
    }

    pub fn slice_mut(&mut self) -> &mut PartSlice {
        match self {
            PartRow::Timed(s) => s,
        }
    }
}

pub(crate) enum GroupedTrack {
    Timed(GroupedPart),
}

impl GroupedTrack {
    pub(crate) fn measure_count(&self) -> usize {
        match self {
            GroupedTrack::Timed(part) => part.measures.len(),
        }
    }

    pub(crate) fn track_name(&self) -> &Option<String> {
        match self {
            GroupedTrack::Timed(part) => &part.name,
        }
    }
}

#[derive(Clone)]
pub struct Score {
    pub metadata: Metadata,
    pub measures: Vec<MultiPartMeasure>,
    /// Document-level diagnostics (e.g. metadata parse errors), not tied to any measure.
    pub document_diagnostics: Vec<Diagnostic>,
}

// ── Intermediate grouper types (not part of the public API) ─────────────────

pub(crate) struct MeasureDirectives {
    pub(crate) time_signature: Option<TimeSignature>,
    pub(crate) bpm: Option<u32>,
    pub(crate) key: Option<KeyChange>,
    pub(crate) label: Option<String>,
}

pub(crate) struct GroupedScore {
    pub(crate) measure_directives: Vec<MeasureDirectives>,
    pub(crate) parts: Vec<GroupedTrack>,
    pub(crate) per_measure_parse_errors: Vec<Option<RecoverableError>>,
}

pub(crate) struct GroupedMeasure {
    pub(crate) notes: Notes,
    pub(crate) source_span: Span,
    /// Tie-aware syllables paired to this measure's lyric slots. Set for
    /// `NotesWithLyrics` parts during grouping.
    pub(crate) paired_lyrics: Option<Vec<Syllable>>,
    /// Recoverable lyrics underflow for this measure, if any.
    pub(crate) lyrics_error: Option<Warning>,
    /// Recoverable beat overflow for this measure (notes trimmed), if any.
    pub(crate) beat_overflow_error: Option<Warning>,
    /// Recoverable error from `-` used after a rest in this measure, if any.
    pub(crate) dash_after_rest_error: Option<RecoverableError>,
    /// Grouping diagnostics: dotted-eighth RecoverableErrors and half-bar-boundary Warnings.
    pub(crate) dotted_eighth_errors: Vec<Diagnostic>,
    /// Chord parse diagnostics: promoted kinds are Error, others are Warning.
    pub(crate) chord_errors: Vec<Diagnostic>,
    /// Recoverable lex error from an unexpected character on the notes line, if any.
    pub(crate) lex_error: Option<RecoverableError>,
    /// Recoverable error from a malformed lyrics line (e.g. empty lyrics line), if any.
    pub(crate) lyrics_parse_error: Option<RecoverableError>,
    /// Recoverable error from `-` at the start of a measure with no preceding event, if any.
    pub(crate) extension_no_preceding_event_error: Option<RecoverableError>,
}

pub(crate) struct GroupedPart {
    pub(crate) name: Option<String>,
    pub(crate) kind: crate::ast::parsed::PartKind,
    pub(crate) soundfont: crate::ast::parsed::Soundfont,
    pub(crate) measures: Vec<GroupedMeasure>,
}

// ── Shared note types ─────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

#[derive(Clone)]
pub enum NoteEvent {
    Note(GroupedNote),
    Rest(GroupedRest),
    Chord(GroupedChordNote),
}

#[derive(Clone)]
pub struct GroupedChordNote {
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

#[derive(Clone)]
pub struct GroupedNote {
    pub pitch: JianPuPitch,
    pub octave: i8,
    /// Duration in quarter-beats, including any beats added by `-` extensions.
    pub duration: u32,
    /// True if this note is tied/slurred to the next note.
    pub slur: bool,
    /// True if `~` appeared after the octave modifier, requesting a tie to the next note.
    pub tie_to_next: bool,
    /// Number of nested `(…)` groups this note belongs to.
    pub group_membership: u8,
    /// Number of those groups that continue past this note.
    pub group_continuation: u8,
    /// True if this note was written with `*` (dotted duration).
    pub dotted: bool,
    pub slur_group_close_at_duration: Option<u32>,
}

impl GroupedChordNote {
    pub fn format_symbol(&self) -> String {
        use crate::ast::parsed::{Accidental, Extension, JianPuPitch, TriadQuality};

        let degree = match self.degree {
            JianPuPitch::One => '1',
            JianPuPitch::Two => '2',
            JianPuPitch::Three => '3',
            JianPuPitch::Four => '4',
            JianPuPitch::Five => '5',
            JianPuPitch::Six => '6',
            JianPuPitch::Seven => '7',
        };
        let accidental = match self.accidental {
            Accidental::Sharp => "♯",
            Accidental::Flat => "♭",
            Accidental::Natural => "",
        };
        let triad = match self.triad {
            TriadQuality::Major => "",
            TriadQuality::Minor => "m",
            TriadQuality::Diminished => "°",
            TriadQuality::Augmented => "⁺",
        };
        let extension = match &self.extension {
            Some(Extension::DominantSeventh) => "⁷",
            Some(Extension::MajorSeventh) => "△⁷",
            None => "",
        };
        let mut result = format!("{degree}{accidental}{triad}{extension}");

        if let Some(bass) = &self.bass {
            let bass_degree = match bass.degree {
                JianPuPitch::One => '1',
                JianPuPitch::Two => '2',
                JianPuPitch::Three => '3',
                JianPuPitch::Four => '4',
                JianPuPitch::Five => '5',
                JianPuPitch::Six => '6',
                JianPuPitch::Seven => '7',
            };
            let bass_acc = match bass.accidental {
                Accidental::Sharp => "♯",
                Accidental::Flat => "♭",
                Accidental::Natural => "",
            };
            result.push('/');
            result.push(bass_degree);
            result.push_str(bass_acc);
        }

        result
    }
}

#[derive(Clone)]
pub struct GroupedRest {
    /// Duration in quarter-beats, including any beats added by `-` extensions.
    pub duration: u32,
    /// True if this rest was written with `*` (dotted duration).
    pub dotted: bool,
    pub group_membership: u8,
    pub group_continuation: u8,
}
