import * as Dialog from '@radix-ui/react-dialog'
import './FileDialogs.css'

export interface DeleteFolderDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** Full path of the folder to delete. */
  folder: string
  /** The sentence to show (file count and where the files go), from Rust. */
  message: string | undefined
  /** What deleting does to live links involved (from Rust), if any. */
  shareWarning?: string
  onConfirm: (folder: string) => void
}

/** Asks before a folder's files all move to the bin. */
export function DeleteFolderDialog({
  open,
  onOpenChange,
  folder,
  message,
  shareWarning,
  onConfirm,
}: DeleteFolderDialogProps) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="file-dialog-overlay" />
        <Dialog.Content className="file-dialog" aria-describedby={undefined}>
          <div className="file-dialog-header">
            <Dialog.Title className="file-dialog-title">
              {`Delete folder "${folder}"?`}
            </Dialog.Title>
            <Dialog.Close className="file-dialog-close" aria-label="Close">
              ×
            </Dialog.Close>
          </div>
          <div className="file-dialog-body">
            <p className="file-dialog-warn-box">{message}</p>
            {shareWarning ? (
              <p
                className="file-dialog-warn-box"
                data-testid="delete-folder-share-warning"
              >
                {shareWarning}
              </p>
            ) : null}
          </div>
          <div className="file-dialog-footer">
            <button
              type="button"
              className="file-dialog-button"
              onClick={() => onOpenChange(false)}
            >
              Cancel
            </button>
            <button
              type="button"
              className="file-dialog-button file-dialog-button--danger"
              onClick={() => {
                onOpenChange(false)
                onConfirm(folder)
              }}
            >
              Move to bin
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
