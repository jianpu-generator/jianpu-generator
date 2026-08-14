use crate::ast::parsed::{
    Accidental, BassDegree, Extension, JianPuPitch, TriadQuality, TupletInfo,
};
use crate::error::Span;

#[derive(Clone, PartialEq)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

#[derive(Clone)]
pub enum NoteEvent {
    Note(GroupedNote),
    Rest(GroupedRest),
    Chord(GroupedChordNote),
    Percussion(GroupedPercussionHit),
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
    pub tie_to_next_span: Option<Span>,
    pub event_span: Span,
    pub group_membership: u8,
    pub group_continuation: u8,
    pub dotted: bool,
    pub double_dotted: bool,
    pub slur_group_close_at_duration: Option<u32>,
    /// The innermost `{...}` tuplet bracket this chord note belonged to before
    /// rescaling, if any.
    pub tuplet: Option<TupletInfo>,
}

#[derive(Clone)]
pub struct GroupedNote {
    pub pitch: JianPuPitch,
    pub accidental: Accidental,
    pub octave: i8,
    /// Duration in quarter-beats, including any beats added by `-` extensions.
    pub duration: u32,
    /// True if this note is tied/slurred to the next note.
    pub slur: bool,
    /// Source span of the `~` suffix when this note is tied to the next note.
    pub tie_to_next_span: Option<Span>,
    /// Byte range of this note token in the original source.
    pub event_span: Span,
    /// Number of nested `(…)` groups this note belongs to.
    pub group_membership: u8,
    /// Number of those groups that continue past this note.
    pub group_continuation: u8,
    /// True if this note was written with `*` (dotted duration).
    pub dotted: bool,
    /// True if this note was written with a second dot (double-dotted duration).
    /// Only ever `true` when `dotted` is also `true`.
    pub double_dotted: bool,
    pub slur_group_close_at_duration: Option<u32>,
    /// The innermost `{...}` tuplet bracket this note belonged to before rescaling,
    /// if any.
    pub tuplet: Option<TupletInfo>,
}

impl GroupedNote {
    pub fn tie_to_next(&self) -> bool {
        self.tie_to_next_span.is_some()
    }
}

#[derive(Clone)]
pub struct GroupedPercussionHit {
    /// Duration in quarter-beats, including any beats added by `-` extensions.
    pub duration: u32,
    /// True if this hit is tied/slurred to the next hit.
    pub slur: bool,
    /// Source span of the `~` suffix when this hit is tied to the next hit.
    pub tie_to_next_span: Option<Span>,
    /// Byte range of this hit token in the original source.
    pub event_span: Span,
    /// Number of nested `(…)` groups this hit belongs to.
    pub group_membership: u8,
    /// Number of those groups that continue past this hit.
    pub group_continuation: u8,
    /// True if this hit was written with `*` (dotted duration).
    pub dotted: bool,
    /// True if this hit was written with a second dot (double-dotted duration).
    pub double_dotted: bool,
    pub slur_group_close_at_duration: Option<u32>,
    /// The innermost `{...}` tuplet bracket this hit belonged to before rescaling,
    /// if any.
    pub tuplet: Option<TupletInfo>,
}

impl GroupedPercussionHit {
    pub fn tie_to_next(&self) -> bool {
        self.tie_to_next_span.is_some()
    }
}

impl GroupedChordNote {
    pub fn tie_to_next(&self) -> bool {
        self.tie_to_next_span.is_some()
    }
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
            TriadQuality::Sus2 => "sus2",
            TriadQuality::Sus4 => "sus4",
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
    /// True if this rest was written with a second dot (double-dotted duration).
    pub double_dotted: bool,
    pub group_membership: u8,
    pub group_continuation: u8,
    /// The innermost `{...}` tuplet bracket this rest belonged to before rescaling,
    /// if any.
    pub tuplet: Option<TupletInfo>,
}
