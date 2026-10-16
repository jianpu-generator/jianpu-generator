interface SharedPreviewBannerProps {
  /** Name of the shared folder, when the link is a static folder link. */
  folderName?: string
  onImport: () => void
  onDiscard: () => void
}

export function SharedPreviewBanner({
  folderName,
  onImport,
  onDiscard,
}: SharedPreviewBannerProps) {
  return (
    <div className="shared-preview-banner">
      {folderName ? (
        <p>
          Shared folder: <strong>{folderName}</strong>
        </p>
      ) : null}
      <div className="shared-preview-actions">
        <button
          type="button"
          className="shared-preview-import-btn"
          onClick={onImport}
        >
          {folderName ? 'Import this folder' : 'Import this score'}
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
