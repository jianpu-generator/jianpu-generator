interface PreviewProps {
  svgs: string[]
  rendering: boolean
  wavUrl?: string | null
  audioAvailable?: boolean
  emptyMessage?: string
}

export function Preview({
  svgs,
  rendering,
  wavUrl = null,
  audioAvailable = false,
  emptyMessage = 'No preview yet.',
}: PreviewProps) {
  return (
    <div className="preview">
      <div className="preview-header">
        <span>Preview</span>
        {rendering ? <span className="preview-status">Rendering…</span> : null}
      </div>
      {audioAvailable ? (
        <div className="preview-audio">
          {wavUrl ? (
            // biome-ignore lint/a11y/useMediaCaption: synthesized score preview has no captions track
            <audio className="preview-audio-player" controls src={wavUrl} />
          ) : (
            <span className="preview-audio-empty">
              {rendering ? 'Generating audio…' : 'No audio yet.'}
            </span>
          )}
        </div>
      ) : null}
      <div className="preview-pages">
        {svgs.length === 0 && !rendering ? (
          <p className="preview-empty">{emptyMessage}</p>
        ) : null}
        {svgs.map((svg) => (
          <div
            key={svg}
            className="preview-page"
            // biome-ignore lint/security/noDangerouslySetInnerHtml: trusted SVG from local WASM renderer
            dangerouslySetInnerHTML={{ __html: svg }}
          />
        ))}
      </div>
    </div>
  )
}
