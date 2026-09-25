use crate::ast::grouped::Metadata;
use crate::ast::parsed::{ParsedMetadata, TextStyle, TextStyleKind};

use super::resolve_metadata;

/// What each `# metadata` field resolves to when left unset, given the rest
/// of `metadata` as written: e.g. `notes`' default `font_size` follows an
/// explicit `lyrics` font size, and the `row_height`-scaled font sizes follow
/// an explicit `row_height`. Every style default is found by clearing that
/// kind's style and resolving, so it can't drift from `resolve_metadata`.
/// The text fields (`title`/`subtitle`/`author`) have no default and are `None`.
pub fn metadata_defaults(metadata: &ParsedMetadata) -> Metadata {
    let mut defaults = resolve_metadata(ParsedMetadata::default());
    for &kind in TextStyleKind::ALL {
        let mut unset = metadata.clone();
        *unset.style_mut(kind) = TextStyle::default();
        *defaults.style_mut(kind) = *resolve_metadata(unset).style(kind);
    }
    defaults
}
