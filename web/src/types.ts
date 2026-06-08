export interface ByteSpan {
  start: number
  end: number
}

export interface RenderError {
  message: string
  span: ByteSpan
  report?: string
}

export interface EditorSelection {
  start: number
  end: number
}

export interface EditorHandle {
  /** Insert text at the current cursor, replacing any selection. */
  insertAtCursor: (text: string) => void
  getSelection: () => EditorSelection
  setSelection: (start: number, end: number) => void
  focus: () => void
  getTextarea: () => HTMLTextAreaElement | null
}
