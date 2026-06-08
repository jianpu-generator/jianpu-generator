interface PreviewProps {
  svgs: string[]
  rendering: boolean
}

export function Preview({ svgs, rendering }: PreviewProps) {
  return (
    <div className="preview">
      <div className="preview-header">
        <span>Preview</span>
        {rendering ? <span className="preview-status">Rendering…</span> : null}
      </div>
      <div className="preview-pages">
        {svgs.length === 0 && !rendering ? (
          <p className="preview-empty">No preview yet.</p>
        ) : null}
        {svgs.map((svg, index) => (
          <div
            key={index}
            className="preview-page"
            // SVG is generated locally by our WASM module.
            dangerouslySetInnerHTML={{ __html: svg }}
          />
        ))}
      </div>
    </div>
  )
}
