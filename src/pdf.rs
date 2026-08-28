use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span};
use base64::Engine;
use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, TextStr};
use std::collections::HashMap;

pub struct PdfFonts {
    /// The `title` role's font bytes (currently Zhuque Fangsong — see
    /// `fonts/fonts.json`, the single source of truth for which file backs
    /// each role) — despite the field's name, no longer Source Han Sans SC
    /// or TW-Kai. Backs `FontFamily::Title` (the song title, subtitle,
    /// author, and lyric syllables/lines, see its doc comment in
    /// `src/compositor/types.rs`), addressed by its own literal family name
    /// in the SVG's `font-family` (`TITLE_FONT_FAMILY` in
    /// `src/serializer/mod.rs`) rather than via `set_sans_serif_family` —
    /// `fontdb` resolves a literal name by matching a loaded font's own
    /// name-table family, so whatever font backs this role just needs to be
    /// loaded here, not bound to a generic alias (see `fonts/fonts.json`'s
    /// comment on this).
    pub sans_serif_sc: Vec<u8>,
    /// The `sansSerif` role's font bytes (currently Source Han Sans SC —
    /// see `fonts/fonts.json`) — the default/body CJK font PDF export
    /// resolves `sans-serif` to (see `set_sans_serif_family` below),
    /// covering everything except `FontFamily::Title`'s text (directive
    /// line, part legend, footer). Loaded separately from `sans_serif_sc`
    /// since the two roles can be backed by different font files (and
    /// currently are — see `fonts/fonts.json`'s comment on why the split
    /// exists).
    pub sans_serif_tc: Vec<u8>,
    pub monospace: Vec<u8>,
}

/// Writes PDF bytes for the given rendered SVG pages, optionally embedding
/// `source` (the original `.jianpu` text) as a base64-encoded `/JianpuSource`
/// key in the PDF's `/Info` dictionary. `usvg` (used to parse `svgs` before
/// PDF conversion) strips `<metadata>` tags, so unlike SVG output the source
/// can't ride along inside the page content — see `extract_embedded_source_from_pdf`
/// for the matching extraction side.
pub fn write_pdf(
    svgs: &[String],
    fonts: &PdfFonts,
    source: Option<&str>,
) -> Result<Vec<u8>, IrrecoverableError> {
    if svgs.is_empty() {
        return Ok(Vec::new());
    }

    let mut options = svg2pdf::usvg::Options::default();
    {
        let db = options.fontdb_mut();
        db.load_font_data(fonts.sans_serif_sc.clone());
        db.load_font_data(fonts.sans_serif_tc.clone());
        db.load_font_data(fonts.monospace.clone());
        db.set_sans_serif_family(crate::fonts::SANS_SERIF_FONT_NAME);
        db.set_monospace_family(crate::fonts::MONOSPACE_FONT_NAME);
    }

    let conversion_options = svg2pdf::ConversionOptions::default();
    let mut alloc = Ref::new(1);

    let catalog_id = alloc.bump();
    let page_tree_id = alloc.bump();

    let num_pages = svgs.len();
    let page_ids: Vec<Ref> = (0..num_pages).map(|_| alloc.bump()).collect();
    let content_ids: Vec<Ref> = (0..num_pages).map(|_| alloc.bump()).collect();

    let mut pdf = Pdf::new();
    pdf.catalog(catalog_id).pages(page_tree_id);
    pdf.pages(page_tree_id)
        .count(num_pages as i32)
        .kids(page_ids.iter().copied());

    let svg_name = Name(b"Svg");

    for ((svg_str, page_id), content_id) in svgs.iter().zip(page_ids.iter()).zip(content_ids.iter())
    {
        let tree = svg2pdf::usvg::Tree::from_str(svg_str, &options).map_err(|e| {
            IrrecoverableError::new(IrrecoverableErrorKind::PdfSvgParseFailed {
                span: Span::new(0, 0),
                detail: e.to_string(),
            })
        })?;

        let page_width = tree.size().width();
        let page_height = tree.size().height();

        let (svg_chunk, svg_ref) = svg2pdf::to_chunk(&tree, conversion_options).map_err(|e| {
            IrrecoverableError::new(IrrecoverableErrorKind::PdfSvgConversionFailed {
                span: Span::new(0, 0),
                detail: e.to_string(),
            })
        })?;

        // Renumber the chunk's internal refs so they don't conflict with our allocator.
        let mut map = HashMap::new();
        let svg_chunk = svg_chunk.renumber(|old| *map.entry(old).or_insert_with(|| alloc.bump()));
        let svg_ref_new = map.get(&svg_ref).copied().ok_or_else(|| {
            IrrecoverableError::new(IrrecoverableErrorKind::internal_invariant(
                Span::new(0, 0),
                "internal invariant: SVG chunk ref missing after renumber",
            ))
        })?;

        pdf.extend(&svg_chunk);

        // Content stream: scale the 1×1 XObject to fill the page.
        let mut content = Content::new();
        content.transform([page_width, 0.0, 0.0, page_height, 0.0, 0.0]);
        content.x_object(svg_name);
        let content_bytes = content.finish();

        pdf.stream(*content_id, &content_bytes).finish();

        let mut page = pdf.page(*page_id);
        page.media_box(Rect::new(0.0, 0.0, page_width, page_height));
        page.parent(page_tree_id);
        page.contents(*content_id);
        let mut resources = page.resources();
        resources.x_objects().pair(svg_name, svg_ref_new);
        resources.finish();
        page.finish();
    }

    if let Some(source) = source {
        let info_id = alloc.bump();
        let encoded = base64::engine::general_purpose::STANDARD.encode(source);
        pdf.document_info(info_id)
            .pair(Name(b"JianpuSource"), TextStr(&encoded));
    }

    Ok(pdf.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pdf(score_str: &str, lyrics_str: &str) -> Vec<u8> {
        let input = format!(
            "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n# score\ntime=4/4 key=C4 bpm=120\n{score_str}\n{lyrics_str}\n"
        );
        let svgs = crate::render_svgs_from_source(&input, "test.jianpu", &[])
            .unwrap()
            .svgs;
        let fonts = PdfFonts {
            sans_serif_sc: crate::fonts::TITLE_FONT_BYTES.to_vec(),
            sans_serif_tc: crate::fonts::SANS_SERIF_FONT_BYTES.to_vec(),
            monospace: crate::fonts::MONOSPACE_FONT_BYTES.to_vec(),
        };
        write_pdf(&svgs, &fonts, None).unwrap()
    }

    #[test]
    fn produces_non_empty_pdf_bytes() {
        let bytes = make_pdf("1 2 3 4", "a b c d");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn pdf_starts_with_pdf_header() {
        let bytes = make_pdf("1 2 3 4", "a b c d");
        assert!(bytes.starts_with(b"%PDF"));
    }
}
