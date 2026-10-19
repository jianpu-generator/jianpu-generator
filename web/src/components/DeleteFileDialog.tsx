import * as Dialog from '@radix-ui/react-dialog'
import './FileDialogs.css'

export interface DeleteFileDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** Full path of the file to delete. */
  file: string
  onConfirm: (file: string) => void
}

/** Asks before a file moves to the bin. */
export function DeleteFileDialog({
  open,
  onOpenChange,
  file,
  onConfirm,
}: DeleteFileDialogProps) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="file-dialog-overlay" />
        <Dialog.Content className="file-dialog" aria-describedby={undefined}>
          <div className="file-dialog-header">
            <Dialog.Title className="file-dialog-title">
              {`Delete file "${file}"?`}
            </Dialog.Title>
            <Dialog.Close className="file-dialog-close" aria-label="Close">
              ×
            </Dialog.Close>
          </div>
          <div className="file-dialog-body">
            <p className="file-dialog-warn-box">
              The file moves to the bin, where you can restore it.
            </p>
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
                onConfirm(file)
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
