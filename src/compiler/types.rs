use crate::ast::parsed::JianPuPitch;

#[derive(Debug, Clone, PartialEq)]
pub struct MeasureBlock {
    pub rows: Vec<MeasureRow>,
    pub decorations: Vec<Decoration>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasureRow {
    pub id: RowId,
    pub label: String,
    pub elements: Vec<ColumnElement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RowId(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnElement {
    pub column: u32,
    pub content: ElementContent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementContent {
    NoteHead {
        pitch: JianPuPitch,
        octave: i8,
        dotted: bool,
    },
    Rest {
        dotted: bool,
    },
    ChordSymbol(String),
    Underline {
        from_column: u32,
        to_column: u32,
        last_head_column: u32,
        level: u32,
    },
    TieOrSlur {
        from_column: u32,
        to_column: u32,
    },
    BarLine,
    Lyric(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decoration {
    Bpm(u32),
    TimeSignature { numerator: u32, denominator: u32 },
    SectionLabel(String),
    BarNumber(u32),
}
