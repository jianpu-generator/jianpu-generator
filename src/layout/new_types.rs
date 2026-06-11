use crate::compiler::types::{MeasureBlock, RowId};

#[derive(Debug, Clone)]
pub struct Page {
    pub header: Header,
    pub footer: Footer,
    pub systems: Vec<System>,
    pub page_width_pt: f32,
    pub page_height_pt: f32,
}

#[derive(Debug, Clone)]
pub struct System {
    pub row_labels: Vec<RowLabel>,
    pub measures: Vec<MeasureBlock>,
}

#[derive(Debug, Clone)]
pub struct RowLabel {
    pub id: RowId,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Header {
    pub title: String,
    pub subtitle: Option<String>,
    pub author: String,
}

#[derive(Debug, Clone)]
pub struct Footer {
    pub page: u32,
    pub total: u32,
}
