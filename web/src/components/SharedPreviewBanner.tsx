interface SharedPreviewBannerProps {
  filename: string
  onImport: () => void
  onDiscard: () => void
}

export function SharedPreviewBanner({
  filename,
  onImport,
  onDiscard,
}: SharedPreviewBannerProps) {
  return (
    <div className="shared-preview-banner">
      <p>
        Viewing a shared score: <strong>{filename}</strong>
      </p>
      <div className="shared-preview-actions">
        <button
          type="button"
          className="shared-preview-import-btn"
          onClick={onImport}
        >
          Import to my scores
        </button>
        <button
          type="button"
          className="shared-preview-discard-btn"
          onClick={onDiscard}
        >
          Discard
        </button>
      </div>
    </div>
  )
}
