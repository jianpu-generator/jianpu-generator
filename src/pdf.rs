use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span};
use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref};
use std::collections::HashMap;

pub struct PdfFonts {
    pub sans_serif_sc: Vec<u8>,
    pub sans_serif_tc: Vec<u8>,
    pub monospace: Vec<u8>,
}

pub fn write_pdf(svgs: &[String], fonts: &PdfFonts) -> Result<Vec<u8>, IrrecoverableError> {
    if svgs.is_empty() {
        return Ok(Vec::new());
    }

    let mut options = svg2pdf::usvg::Options::default();
    {
        let db = options.fontdb_mut();
        db.load_font_data(fonts.sans_serif_sc.clone());
        db.load_font_data(fonts.sans_serif_tc.clone());
        db.load_font_data(fonts.monospace.clone());
        db.set_sans_serif_family("Source Han Sans SC");
        db.set_monospace_family("Noto Sans Mono");
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

    Ok(pdf.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pdf(score_str: &str, lyrics_str: &str) -> Vec<u8> {
        let input = format!(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[parts]\nMelody = notes lyrics\n\n[score]\ntime=4/4 key=C4 bpm=120\n{score_str}\n{lyrics_str}\n"
        );
        let svgs = crate::render_svgs_from_source(&input, "test.jianpu")
            .unwrap()
            .svgs;
        let fonts = PdfFonts {
            sans_serif_sc: include_bytes!("../fonts/SourceHanSansSC-Regular.otf").to_vec(),
            sans_serif_tc: include_bytes!("../fonts/SourceHanSansTC-Regular.otf").to_vec(),
            monospace: include_bytes!("../fonts/NotoSansMono-Regular.ttf").to_vec(),
        };
        write_pdf(&svgs, &fonts).unwrap()
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
