use crate::ast::grouped::Metadata;

#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub row_height: u32,
    pub label_width: u32,
    pub note_number_width: u32,
    pub max_columns: u32,
}

impl RenderConfig {
    pub fn from_metadata(meta: &Metadata) -> Self {
        RenderConfig {
            row_height: meta.row_height,
            label_width: meta.label_width,
            note_number_width: meta.note_number_width,
            max_columns: meta.max_columns,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::grouped::Metadata;

    #[test]
    fn from_metadata_copies_fields() {
        let meta = Metadata {
            title: String::new(),
            subtitle: None,
            author: String::new(),
            row_height: 30,
            label_width: 20,
            note_number_width: 12,
            max_columns: 48,
        };
        let cfg = RenderConfig::from_metadata(&meta);
        assert_eq!(cfg.row_height, 30);
        assert_eq!(cfg.label_width, 20);
        assert_eq!(cfg.note_number_width, 12);
        assert_eq!(cfg.max_columns, 48);
    }
}
