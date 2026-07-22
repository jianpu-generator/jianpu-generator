/// A translation in points, applied to a rendered element after its layout
/// position has been resolved. Does not affect the position of any other
/// element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Offset {
    pub x: i32,
    pub y: i32,
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
