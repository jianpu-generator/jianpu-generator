import MonacoEditor, { type Monaco, type OnMount } from '@monaco-editor/react'
import type { editor, IDisposable, languages } from 'monaco-editor'
import {
  forwardRef,
  type ReactNode,
  useCallback,
  useEffect,
  useImperativeHandle,
  useRef,
} from 'react'
import type {
  Diagnostic,
  EditorHandle,
  MeasureSpan,
  ScoreLineHint,
} from '../types'
import {
  byteOffsetToStringIndex,
  stringIndexToByteOffset,
} from '../utils/byteSpan'

export interface EditorProps {
  value: string
  onChange: (value: string) => void
  readOnly?: boolean
  diagnostics?: Diagnostic[]
  measureSpans?: MeasureSpan[]
  scoreLineHints?: ScoreLineHint[]
  toolbar?: ReactNode
  onSelectionChange?: (startOffset: number, endOffset: number) => void
  onCursorLineChange?: (line: number) => void
  onPlayMeasure?: () => void
}

const MARKER_OWNER = 'jianpu'

function buildPartInlayHints(
  model: editor.ITextModel,
  scoreLineHints: ScoreLineHint[],
  monacoApi: Monaco,
): languages.InlayHint[] {
  if (scoreLineHints.length === 0) return []
  const source = model.getValue()

  return scoreLineHints.map((hint) => ({
    position: new monacoApi.Position(
      model.getPositionAt(byteOffsetToStringIndex(source, hint.line_start))
        .lineNumber,
      1,
    ),
    label: `[${hint.abbreviation}]`,
    kind: monacoApi.languages.InlayHintKind.Type,
    paddingRight: true,
  }))
}

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

export const Editor = forwardRef<EditorHandle, EditorProps>(function Editor(
  {
    value,
    onChange,
    readOnly = false,
    diagnostics = [],
    measureSpans = [],
    scoreLineHints = [],
    toolbar,
    onSelectionChange,
    onCursorLineChange,
    onPlayMeasure,
  },
  ref,
) {
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null)
  const monacoRef = useRef<Monaco | null>(null)
  const viewZoneIdsRef = useRef<string[]>([])
  const inlayHintsDisposableRef = useRef<IDisposable | null>(null)
  const scoreLineHintsForInlayRef = useRef(scoreLineHints)
  const onSelectionChangeRef = useRef(onSelectionChange)
  const onCursorLineChangeRef = useRef(onCursorLineChange)
  const onPlayMeasureRef = useRef(onPlayMeasure)
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
      for (const id of viewZoneIdsRef.current) {
        accessor.removeZone(id)
      }
      viewZoneIdsRef.current = []

      const source = model.getValue()

      measureSpans.forEach((span, index) => {
        const stringIndex = byteOffsetToStringIndex(source, span.viewZoneStart)
        const lineNumber = model.getPositionAt(stringIndex).lineNumber

        const domNode = document.createElement('div')
        domNode.style.cssText = [
          'width: 100%',
          'height: 21px',
          'background: #dbeafe',
          'color: #1e40af',
          'font-family: var(--mono)',
          'font-size: 14px',
          'font-weight: bold',
          'display: flex',
          'align-items: center',
          'padding-left: 8px',
          'box-sizing: border-box',
        ].join(';')
        domNode.textContent = `[Measure ${index + 1}]`

        const id = accessor.addZone({
          afterLineNumber: lineNumber - 1,
          heightInLines: 1,
          domNode,
        })
        viewZoneIdsRef.current.push(id)
      })
    })
  }, [measureSpans])

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
    focus() {
      editorRef.current?.focus()
    },
    getEditor() {
      return editorRef.current
    },
  }))

  const applyInlayHints = useCallback(() => {
    const monacoApi = monacoRef.current
    if (!monacoApi) return
    inlayHintsDisposableRef.current?.dispose()
    inlayHintsDisposableRef.current =
      monacoApi.languages.registerInlayHintsProvider('plaintext', {
        provideInlayHints(model, _range, _token) {
          return {
            hints: buildPartInlayHints(
              model,
              scoreLineHintsForInlayRef.current,
              monacoApi,
            ),
            dispose: () => {},
          }
        },
      })
  }, [])

  const handleMount: OnMount = (ed, monacoApi) => {
    editorRef.current = ed
    monacoRef.current = monacoApi
    applyDiagnostics()
    applyMeasureViewZones()
    applyInlayHints()

    ed.addCommand(monacoApi.KeyMod.CtrlCmd | monacoApi.KeyCode.Enter, () =>
      onPlayMeasureRef.current?.(),
    )

    const notifyCursor = () => {
      const model = ed.getModel()
      if (!model) return
      const selection = ed.getSelection()
      if (!selection) return
      const source = model.getValue()
      if (onSelectionChangeRef.current) {
        const startCharIndex = model.getOffsetAt(selection.getStartPosition())
        const endCharIndex = model.getOffsetAt(selection.getEndPosition())
        const startOffset = stringIndexToByteOffset(source, startCharIndex)
        const endOffset = stringIndexToByteOffset(source, endCharIndex)
        onSelectionChangeRef.current(startOffset, endOffset)
      }
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
    scoreLineHintsForInlayRef.current = scoreLineHints
    applyInlayHints()
  }, [scoreLineHints, applyInlayHints])

  useEffect(() => {
    return () => {
      inlayHintsDisposableRef.current?.dispose()
    }
  }, [])

  return (
    <div className="editor">
      {toolbar ? <div className="editor-toolbar">{toolbar}</div> : null}
      <div className="editor-surface">
        <MonacoEditor
          height="100%"
          language="plaintext"
          theme="vs"
          value={value}
          onChange={(next) => onChange(next ?? '')}
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
            renderLineHighlight: 'none',
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
