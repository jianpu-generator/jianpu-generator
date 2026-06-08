import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  type ReactNode,
} from 'react'
import type { ByteSpan, EditorHandle } from '../types'

export interface EditorProps {
  value: string
  onChange: (value: string) => void
  /** Byte span from the WASM parser — used for scroll-to-error now, overlay later. */
  errorSpan?: ByteSpan | null
  /** Slot for a future formatting toolbar (WYSIWYG-style insert buttons). */
  toolbar?: ReactNode
}

function scrollToByteOffset(textarea: HTMLTextAreaElement, offset: number) {
  const before = textarea.value.slice(0, offset)
  const line = before.split('\n').length
  const lineHeight =
    Number.parseFloat(getComputedStyle(textarea).lineHeight) || 20
  textarea.scrollTop = Math.max(0, (line - 4) * lineHeight)
  textarea.focus()
}

export const Editor = forwardRef<EditorHandle, EditorProps>(function Editor(
  { value, onChange, errorSpan, toolbar },
  ref,
) {
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  useImperativeHandle(ref, () => ({
    insertAtCursor(text: string) {
      const el = textareaRef.current
      if (!el) return
      const start = el.selectionStart
      const end = el.selectionEnd
      const next = value.slice(0, start) + text + value.slice(end)
      onChange(next)
      const cursor = start + text.length
      requestAnimationFrame(() => {
        el.focus()
        el.setSelectionRange(cursor, cursor)
      })
    },
    getSelection() {
      const el = textareaRef.current
      return {
        start: el?.selectionStart ?? 0,
        end: el?.selectionEnd ?? 0,
      }
    },
    setSelection(start: number, end: number) {
      const el = textareaRef.current
      if (!el) return
      el.focus()
      el.setSelectionRange(start, end)
    },
    focus() {
      textareaRef.current?.focus()
    },
    getTextarea() {
      return textareaRef.current
    },
  }))

  useEffect(() => {
    if (!errorSpan || !textareaRef.current) return
    scrollToByteOffset(textareaRef.current, errorSpan.start)
  }, [errorSpan?.start, errorSpan?.end])

  const hasError = errorSpan != null

  return (
    <div className="editor">
      {toolbar ? <div className="editor-toolbar">{toolbar}</div> : null}
      <div
        className={`editor-surface${hasError ? ' editor-surface--error' : ''}`}
        data-error-start={errorSpan?.start}
        data-error-end={errorSpan?.end}
      >
        {/* Overlay layer reserved for future syntax highlights / error ranges */}
        <div className="editor-overlay" aria-hidden="true" />
        <textarea
          ref={textareaRef}
          className="editor-input"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          spellCheck={false}
          autoCapitalize="off"
          autoCorrect="off"
        />
      </div>
    </div>
  )
})
