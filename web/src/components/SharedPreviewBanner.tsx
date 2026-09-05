interface SharedPreviewBannerProps {
  onImport: () => void
  onDiscard: () => void
}

export function SharedPreviewBanner({
  onImport,
  onDiscard,
}: SharedPreviewBannerProps) {
  return (
    <div className="shared-preview-banner">
      <div className="shared-preview-actions">
        <button
          type="button"
          className="shared-preview-import-btn"
          onClick={onImport}
        >
          Import this score
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
