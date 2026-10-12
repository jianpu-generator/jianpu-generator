import * as AlertDialog from '@radix-ui/react-alert-dialog'
import type { ReactElement } from 'react'

export type DiscardConfirmDialogProps = {
  open: boolean
  fileName: string
  onDownloadFirst: () => void
  onConfirm: () => void
  onCancel: () => void
}

export function DiscardConfirmDialog(
  props: DiscardConfirmDialogProps,
): ReactElement {
  return (
    <AlertDialog.Root
      open={props.open}
      onOpenChange={(next) => {
        if (!next) props.onCancel()
      }}
    >
      <AlertDialog.Portal>
        <AlertDialog.Overlay className="discard-confirm__overlay" />
        <AlertDialog.Content className="discard-confirm">
          <AlertDialog.Title>
            Discard changes to {props.fileName}?
          </AlertDialog.Title>
          <AlertDialog.Description>
            {props.fileName} will revert to the server's version. Any changes
            that have not been synced will be lost.
          </AlertDialog.Description>
          <div className="discard-confirm__actions">
            <AlertDialog.Cancel asChild>
              <button type="button">Cancel</button>
            </AlertDialog.Cancel>
            <button type="button" onClick={props.onDownloadFirst}>
              Download my copy first
            </button>
            <button type="button" onClick={props.onConfirm}>
              Discard my changes
            </button>
          </div>
        </AlertDialog.Content>
      </AlertDialog.Portal>
    </AlertDialog.Root>
  )
}
