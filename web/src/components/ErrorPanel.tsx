import type { RenderError } from '../types'

interface ErrorPanelProps {
  error: RenderError | null
}

export function ErrorPanel({ error }: ErrorPanelProps) {
  if (!error) return null

  return (
    <div className="error-panel" role="alert">
      <div className="error-panel-summary">
        <span className="error-panel-message">{error.message}</span>
        <span className="error-panel-span">
          bytes {error.span.start}–{error.span.end}
        </span>
      </div>
      {error.report ? (
        <pre className="error-panel-report">{error.report}</pre>
      ) : null}
    </div>
  )
}
