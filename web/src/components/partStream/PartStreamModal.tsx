import * as Dialog from '@radix-ui/react-dialog'
import { useEffect, useState } from 'react'
import {
  jianpuWasm,
  type PartStream,
  type StreamablePart,
} from '../../jianpuWasm'
import { ensureWasmInit } from '../../wasmInit'
import { PartStreamDiffEditor } from './PartStreamDiffEditor'
import { PartStreamPicker } from './PartStreamPicker'
import './PartStreamModal.css'

export interface PartStreamModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** The main editor's current source. */
  source: string
  /** Writes the applied source back through the main editor's model. */
  onApply: (newSource: string) => void
}

interface SelectedPartStream {
  part: StreamablePart
  stream: PartStream
}

/** Edit one part of the `# score` as a single stream: pick a part, edit its
 * stream in a diff view, then Apply it back. See `part_stream` in Rust. */
export function PartStreamModal({
  open,
  onOpenChange,
  source,
  onApply,
}: PartStreamModalProps) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="part-stream-modal__overlay" />
        <Dialog.Content
          className="part-stream-modal"
          data-testid="part-stream-modal"
          aria-describedby={undefined}
        >
          <PartStreamModalBody
            source={source}
            onClose={() => onOpenChange(false)}
            onApply={(newSource) => {
              onApply(newSource)
              onOpenChange(false)
            }}
          />
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}

interface PartStreamModalBodyProps {
  source: string
  onClose: () => void
  onApply: (newSource: string) => void
}

/** Mounted only while the dialog is open, so each opening starts afresh at
 * the part picker. */
function PartStreamModalBody({
  source,
  onClose,
  onApply,
}: PartStreamModalBodyProps) {
  const [parts, setParts] = useState<StreamablePart[] | null>(null)
  const [selected, setSelected] = useState<SelectedPartStream | null>(null)
  const [extractError, setExtractError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false
    void ensureWasmInit().then(() => {
      if (!cancelled) setParts(jianpuWasm().listStreamableParts(source))
    })
    return () => {
      cancelled = true
    }
  }, [source])

  const selectPart = (part: StreamablePart) => {
    const response = jianpuWasm().extractPartStream(source, part.abbreviation)
    if (response.tag === 'ok') {
      setExtractError(null)
      setSelected({ part, stream: response.val })
    } else {
      setExtractError(response.val.message)
    }
  }

  return (
    <>
      <div className="part-stream-modal__header">
        <Dialog.Title className="part-stream-modal__title">
          {selected === null
            ? 'Edit part as stream'
            : `Edit part as stream: ${selected.part.displayName} [${selected.part.abbreviation}]`}
        </Dialog.Title>
        <Dialog.Close className="part-stream-modal__close" aria-label="Close">
          ×
        </Dialog.Close>
      </div>
      {selected === null ? (
        <PartStreamPicker
          parts={parts}
          extractError={extractError}
          onSelect={selectPart}
        />
      ) : (
        <PartStreamDiffEditor
          source={source}
          part={selected.part}
          stream={selected.stream}
          onApply={onApply}
          onCancel={onClose}
        />
      )}
    </>
  )
}
