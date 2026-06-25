import MonacoEditor, { type Monaco, type OnMount } from '@monaco-editor/react'
import type { editor, ISelection } from 'monaco-editor'
import {
  forwardRef,
  type ReactNode,
  useCallback,
  useEffect,
  useImperativeHandle,
  useLayoutEffect,
  useRef,
} from 'react'
import type {
  Diagnostic,
  DiagnosticMessage,
  DiagnosticViewZone,
  EditorHandle,
  MeasureSpan,
} from '../types'
import { byteOffsetToStringIndex } from '../utils/byteSpan'

export interface EditorProps {
  value: string
  onChange: (value: string) => void
  readOnly?: boolean
  diagnostics?: Diagnostic[]
  diagnosticViewZones?: DiagnosticViewZone[]
  measureSpans?: MeasureSpan[]
  toolbar?: ReactNode
  onSelectionChange?: (startLine: number, endLine: number) => void
  onCursorLineChange?: (line: number) => void
  onPlayMeasure?: () => void
}

const MARKER_OWNER = 'jianpu'
const EDITOR_THEME = 'jianpu'
// Matches preview measure highlight (rgba(255, 200, 0, 0.25)); Monaco only accepts hex.
const MEASURE_HIGHLIGHT_COLOR = '#ffc80040'

function diagnosticRange(
  model: editor.ITextModel,
  source: string,
  diagnostic: Diagnostic,
  monacoApi: Monaco,
) {
  const startIndex = byteOffsetToStringIndex(source, diagnostic.span.start)
  const endIndex = Math.max(
    startIndex + 1,
    byteOffsetToStringIndex(source, diagnostic.span.end),
  )
  const startPos = model.getPositionAt(startIndex)
  const endPos = model.getPositionAt(endIndex)
  return new monacoApi.Range(
    startPos.lineNumber,
    startPos.column,
    endPos.lineNumber,
    endPos.column,
  )
}

const ERROR_ZONE_LINE_HEIGHT_PX = 21

function createDiagnosticViewZoneDomNode(
  severity: 'error' | 'warning',
  messages: DiagnosticMessage[],
): HTMLElement {
  const zoneClass =
    severity === 'warning' ? 'editor-warning-zone' : 'editor-error-zone'
  const messageClass =
    severity === 'warning'
      ? 'editor-warning-zone-message'
      : 'editor-error-zone-message'

  const domNode = document.createElement('div')
  domNode.className = zoneClass

  for (const [index, msg] of messages.entries()) {
    if (index > 0) {
      domNode.appendChild(document.createElement('hr'))
    }
    const messageEl = document.createElement('div')
    messageEl.className = messageClass
    messageEl.textContent = msg.message
    domNode.appendChild(messageEl)

    if (msg.report) {
      const report = document.createElement('pre')
      report.className = 'editor-error-zone-report'
      report.textContent = msg.report
      domNode.appendChild(report)
    }
  }

  return domNode
}

function errorViewZoneHeightInPx(
  domNode: HTMLElement,
  contentWidth: number,
): number {
  domNode.style.width = `${contentWidth}px`
  domNode.style.visibility = 'hidden'
  domNode.style.position = 'absolute'
  document.body.appendChild(domNode)
  const height = domNode.getBoundingClientRect().height
  domNode.remove()
  domNode.style.visibility = ''
  domNode.style.position = ''
  domNode.style.width = ''
  return Math.max(height, ERROR_ZONE_LINE_HEIGHT_PX)
}

export const Editor = forwardRef<EditorHandle, EditorProps>(function Editor(
  {
    value,
    onChange,
    readOnly = false,
    diagnostics = [],
    diagnosticViewZones = [],
    measureSpans = [],
    toolbar,
    onSelectionChange,
    onCursorLineChange,
    onPlayMeasure,
  },
  ref,
) {
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null)
  const monacoRef = useRef<Monaco | null>(null)
  const measureViewZoneIdsRef = useRef<string[]>([])
  const diagnosticViewZoneIdsRef = useRef<string[]>([])
  const onSelectionChangeRef = useRef(onSelectionChange)
  const onCursorLineChangeRef = useRef(onCursorLineChange)
  const onPlayMeasureRef = useRef(onPlayMeasure)
  const savedSelectionRef = useRef<ISelection | null>(null)
  const isInternalChangeRef = useRef(false)
  useEffect(() => {
    onSelectionChangeRef.current = onSelectionChange
    onCursorLineChangeRef.current = onCursorLineChange
    onPlayMeasureRef.current = onPlayMeasure
  })

  const applyDiagnostics = useCallback(() => {
    const ed = editorRef.current
    const monacoApi = monacoRef.current
    const model = ed?.getModel()
    if (!ed || !monacoApi || !model) return

    const source = model.getValue()

    if (diagnostics.length === 0) {
      monacoApi.editor.setModelMarkers(model, MARKER_OWNER, [])
      return
    }

    const markers = diagnostics.map((d) => {
      const range = diagnosticRange(model, source, d, monacoApi)
      return {
        severity:
          d.severity === 'warning'
            ? monacoApi.MarkerSeverity.Warning
            : monacoApi.MarkerSeverity.Error,
        message: d.message,
        startLineNumber: range.startLineNumber,
        startColumn: range.startColumn,
        endLineNumber: range.endLineNumber,
        endColumn: range.endColumn,
      }
    })

    monacoApi.editor.setModelMarkers(model, MARKER_OWNER, markers)
  }, [diagnostics])

  const applyMeasureViewZones = useCallback(() => {
    const ed = editorRef.current
    const model = ed?.getModel()
    if (!ed || !model) return

    ed.changeViewZones((accessor) => {
      for (const id of measureViewZoneIdsRef.current) {
        accessor.removeZone(id)
      }
      measureViewZoneIdsRef.current = []

      const source = model.getValue()

      measureSpans.forEach((span, index) => {
        const stringIndex = byteOffsetToStringIndex(
          source,
          span.view_zone_start,
        )
        const lineNumber = model.getPositionAt(stringIndex).lineNumber

        const hasLabel = span.section_label != null
        const domNode = document.createElement('div')
        domNode.style.cssText = [
          'width: 100%',
          'height: 21px',
          hasLabel ? 'background: #dbeafe' : 'background: #f5f5f5',
          hasLabel ? 'color: #1e40af' : 'color: #666666',
          'font-family: var(--mono)',
          'font-size: 14px',
          'font-weight: bold',
          'display: flex',
          'align-items: center',
          'padding-left: 8px',
          'box-sizing: border-box',
        ].join(';')
        domNode.textContent = span.section_label ?? `${index + 1}`

        const id = accessor.addZone({
          afterLineNumber: lineNumber - 1,
          heightInLines: 1,
          domNode,
        })
        measureViewZoneIdsRef.current.push(id)
      })
    })
  }, [measureSpans])

  const applyDiagnosticViewZones = useCallback(() => {
    const ed = editorRef.current
    if (!ed) return

    ed.changeViewZones((accessor) => {
      for (const id of diagnosticViewZoneIdsRef.current) {
        accessor.removeZone(id)
      }
      diagnosticViewZoneIdsRef.current = []

      for (const zone of diagnosticViewZones) {
        const domNode = createDiagnosticViewZoneDomNode(
          zone.severity,
          zone.messages,
        )
        const heightInPx = errorViewZoneHeightInPx(
          domNode,
          ed.getLayoutInfo().contentWidth,
        )
        const id = accessor.addZone({
          afterLineNumber: zone.after_line_number,
          heightInPx,
          domNode,
        })
        diagnosticViewZoneIdsRef.current.push(id)
      }
    })
  }, [diagnosticViewZones])

  useImperativeHandle(ref, () => ({
    insertAtCursor(text: string) {
      const ed = editorRef.current
      const model = ed?.getModel()
      if (!ed || !model) return

      const selection = ed.getSelection()
      if (!selection) return

      ed.executeEdits('insertAtCursor', [
        {
          range: selection,
          text,
          forceMoveMarkers: true,
        },
      ])
      ed.focus()
    },
    getSelection() {
      const ed = editorRef.current
      const model = ed?.getModel()
      const selection = ed?.getSelection()
      if (!model || !selection) return { start: 0, end: 0 }

      return {
        start: model.getOffsetAt(selection.getStartPosition()),
        end: model.getOffsetAt(selection.getEndPosition()),
      }
    },
    setSelection(start: number, end: number) {
      const ed = editorRef.current
      const model = ed?.getModel()
      const monacoApi = monacoRef.current
      if (!ed || !model || !monacoApi) return

      const startPos = model.getPositionAt(start)
      const endPos = model.getPositionAt(end)
      ed.setSelection(
        new monacoApi.Selection(
          startPos.lineNumber,
          startPos.column,
          endPos.lineNumber,
          endPos.column,
        ),
      )
      ed.focus()
    },
    jumpToOffset(charOffset: number) {
      const ed = editorRef.current
      const model = ed?.getModel()
      if (!ed || !model) return
      const position = model.getPositionAt(charOffset)
      ed.setPosition(position)
      ed.revealPositionInCenter(position)
      ed.focus()
    },
    focus() {
      editorRef.current?.focus()
    },
    getEditor() {
      return editorRef.current
    },
  }))

  const handleMount: OnMount = (ed, monacoApi) => {
    editorRef.current = ed
    monacoRef.current = monacoApi
    applyDiagnostics()
    applyMeasureViewZones()
    applyDiagnosticViewZones()

    ed.addCommand(monacoApi.KeyMod.CtrlCmd | monacoApi.KeyCode.Enter, () =>
      onPlayMeasureRef.current?.(),
    )

    const notifyCursor = () => {
      const model = ed.getModel()
      if (!model) return
      const selection = ed.getSelection()
      if (!selection) return
      onSelectionChangeRef.current?.(
        selection.startLineNumber,
        selection.endLineNumber,
      )
      onCursorLineChangeRef.current?.(selection.startLineNumber)
    }
    ed.onDidChangeCursorPosition(notifyCursor)
    notifyCursor()
  }

  useEffect(() => {
    applyDiagnostics()
  }, [applyDiagnostics])

  useEffect(() => {
    applyMeasureViewZones()
  }, [applyMeasureViewZones])

  useEffect(() => {
    applyDiagnosticViewZones()
  }, [applyDiagnosticViewZones])

  // @monaco-editor/react calls model.setValue() (via useEffect) when the value
  // prop changes externally, which resets the cursor. The fix has two parts:
  //
  // 1. useLayoutEffect runs BEFORE the child's useEffect, so we snapshot the
  //    cursor position here before setValue has a chance to reset it.
  // 2. useEffect runs AFTER the child's useEffect (setValue + reset), so we
  //    restore the snapshotted position here.
  // biome-ignore lint/correctness/useExhaustiveDependencies: value is the trigger; refs don't need to be listed
  useLayoutEffect(() => {
    if (!isInternalChangeRef.current) {
      savedSelectionRef.current = editorRef.current?.getSelection() ?? null
    }
  }, [value])

  // biome-ignore lint/correctness/useExhaustiveDependencies: value is the trigger; refs don't need to be listed
  useEffect(() => {
    if (isInternalChangeRef.current) {
      isInternalChangeRef.current = false
      return
    }
    const ed = editorRef.current
    const saved = savedSelectionRef.current
    if (ed && saved) {
      ed.setSelection(saved)
    }
  }, [value])

  return (
    <div className="editor">
      {toolbar ? <div className="editor-toolbar">{toolbar}</div> : null}
      <div className="editor-surface">
        <MonacoEditor
          height="100%"
          language="plaintext"
          theme={EDITOR_THEME}
          value={value}
          onChange={(next) => {
            isInternalChangeRef.current = true
            onChange(next ?? '')
          }}
          beforeMount={(monacoApi) => {
            monacoApi.editor.defineTheme(EDITOR_THEME, {
              base: 'vs',
              inherit: true,
              rules: [],
              colors: {
                'editor.lineHighlightBackground': MEASURE_HIGHLIGHT_COLOR,
                'editor.lineHighlightBorder': '#00000000',
              },
            })
          }}
          onMount={handleMount}
          options={{
            readOnly,
            minimap: { enabled: false },
            fontFamily: 'var(--mono)',
            fontSize: 14,
            lineHeight: 21,
            padding: { top: 16, bottom: 16 },
            scrollBeyondLastLine: false,
            wordWrap: 'off',
            tabSize: 2,
            renderLineHighlight: 'line',
            renderValidationDecorations: 'on',
            overviewRulerLanes: 2,
            hideCursorInOverviewRuler: true,
            overviewRulerBorder: false,
            glyphMargin: false,
            folding: false,
            lineNumbers: 'on',
            lineNumbersMinChars: 3,
            scrollbar: {
              verticalScrollbarSize: 10,
              horizontalScrollbarSize: 10,
            },
          }}
        />
      </div>
    </div>
  )
})
