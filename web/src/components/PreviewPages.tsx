import type { SvgDocument } from '../jianpuWasm'
import { renderSvgDocument } from './PreviewSvgRenderer'

interface PreviewPagesProps {
  documents: SvgDocument[]
  /** When set, these Verovio-engraved staff-notation pages are shown
   * instead of `documents`. */
  staffPages: string[] | null
}

export function PreviewPages({ documents, staffPages }: PreviewPagesProps) {
  if (staffPages) {
    return staffPages.map((page, i) => (
      <div
        // biome-ignore lint/suspicious/noArrayIndexKey: pages have no stable identifier
        key={i}
        className="preview-page preview-page--staff"
        data-testid="staff-page"
        // biome-ignore lint/security/noDangerouslySetInnerHtml: Verovio's own SVG output, engraved from MEI that Rust escapes (`mei_export::escape`)
        dangerouslySetInnerHTML={{ __html: page }}
      />
    ))
  }
  return documents.map((doc, i) => (
    // biome-ignore lint/suspicious/noArrayIndexKey: pages have no stable identifier
    <div key={i} className="preview-page">
      {renderSvgDocument(doc, i)}
    </div>
  ))
}
