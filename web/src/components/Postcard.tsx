import * as Dialog from '@radix-ui/react-dialog'
import { useEffect, useState } from 'react'
import { POSTCARD_SOURCE } from '../postcardSource'
import { getPostcardSvg, onPostcardUpdate } from '../postcardSvg'
import './Postcard.css'

interface PostcardDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function PostcardDialog({ open, onOpenChange }: PostcardDialogProps) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="postcard-overlay" />
        <Dialog.Content className="postcard-content">
          <Dialog.Title className="postcard-title">
            Syntax Postcard
          </Dialog.Title>
          <Dialog.Close className="postcard-close" aria-label="Close">
            ✕
          </Dialog.Close>
          <PostcardBody />
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}

function PostcardBody() {
  const [, forceUpdate] = useState(0)
  useEffect(() => {
    return onPostcardUpdate(() => forceUpdate((n) => n + 1))
  }, [])

  const svg = getPostcardSvg()

  return (
    <div className="postcard-body">
      <pre className="postcard-source">{POSTCARD_SOURCE}</pre>
      <div className="postcard-svg-pane">
        {svg != null ? (
          <span
            // biome-ignore lint/security/noDangerouslySetInnerHtml: trusted SVG from local WASM renderer
            dangerouslySetInnerHTML={{ __html: svg }}
          />
        ) : (
          <div className="postcard-svg-placeholder" />
        )}
      </div>
    </div>
  )
}
