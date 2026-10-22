//! Chord degrees for tests that build `ChordSymbol`s by hand, where only the
//! formatted text matters.

use super::ChordDegrees;
use crate::ast::parsed::{Accidental, JianPuPitch, TriadQuality};

/// The tonic major chord, `1`.
pub(crate) fn tonic_major() -> ChordDegrees {
    ChordDegrees {
        degree: JianPuPitch::One,
        accidental: Accidental::Natural,
        triad: TriadQuality::Major,
        extension: None,
        bass: None,
    }
}
