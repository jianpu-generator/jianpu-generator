use crate::renderer::new_types::{SvgElement, SvgKind, TransparentRectRole};

/// The rect-shaped half of [`super::serialize_element`]'s dispatch, split
/// out to stay under the file's line-count cap per function.
pub(super) fn serialize_rect_element(el: &SvgElement, out: &mut String, kind: &SvgKind) {
    match kind {
        SvgKind::Rect { width, height } => {
            out.push_str(&format!(
                r#"<rect data-testid="measure-highlight" x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="rgba(255,200,0,0.25)" rx="2"/>"#,
                el.x, el.y, width, height
            ));
        }
        SvgKind::ErrorRect { width, height } => {
            out.push_str(&format!(
                r#"<rect data-testid="error-highlight" x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="rgba(255,0,0,0.15)" rx="2"/>"#,
                el.x, el.y, width, height
            ));
        }
        SvgKind::PlaybackCursorRect { width, height } => {
            // No `rx`: adjacent notes' rects are laid out edge-to-edge
            // (`compute_all_playback_cursor_targets`), and a rounded corner
            // here would carve a visible sliver out of each rect's shared
            // edge, leaving a gap between the two fills during playback even
            // though their `x`/`width` line up exactly.
            out.push_str(&format!(
                r#"<rect data-variant="playback-cursor-rect" x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="transparent"/>"#,
                el.x, el.y, width, height
            ));
        }
        SvgKind::TransparentRect {
            width,
            height,
            role,
        } => {
            let stroke = match role {
                TransparentRectRole::SectionLabelBackground => {
                    r#" stroke="black" stroke-width="1""#
                }
                TransparentRectRole::MeasureClickTarget
                | TransparentRectRole::BarNumberClickTarget
                | TransparentRectRole::SectionLabelClickTarget
                | TransparentRectRole::NoteClickTarget
                | TransparentRectRole::PartLabelClickTarget
                | TransparentRectRole::LyricClickTarget
                | TransparentRectRole::LyricLabelClickTarget
                | TransparentRectRole::BarLineClickTarget => "",
            };
            out.push_str(&format!(
                r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" data-variant="{}" fill="transparent" rx="2"{} style="cursor:pointer"/>"#,
                el.x, el.y, width, height, role.as_str(), stroke
            ));
        }
        _ => {}
    }
}
