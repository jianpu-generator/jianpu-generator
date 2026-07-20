import MonacoEditor, { type Monaco, type OnMount } from '@monaco-editor/react'
import type { editor, IDisposable, ISelection, languages } from 'monaco-editor'
import {
  forwardRef,
  type ReactNode,
  useCallback,
  useEffect,
  useImperativeHandle,
  useLayoutEffect,
  useRef,
} from 'react'
import {
  JIANPU_LANGUAGE_ID,
  registerJianpuLanguage,
} from '../monacoJianpuLanguage'
import { registerJianpuRenameProvider } from '../monacoRenameProvider'
import type {
  Diagnostic,
  DiagnosticViewZone,
  EditorHandle,
  MeasureSpan,
} from '../types'
import { byteOffsetToStringIndex } from '../utils/byteSpan'
import {
  createDiagnosticViewZoneDomNode,
  diagnosticRange,
  errorViewZoneHeightInPx,
} from './editorDiagnosticViewZones'
import { createEditorImperativeHandle } from './editorImperativeHandle'

export interface EditorProps {
  /** Unique per-file ID; gives each file its own Monaco model and undo stack. */
  path?: string
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
  onForceSave?: () => void
  onEditPartsClick?: () => void
  onEditMetadataClick?: () => void
}

const MARKER_OWNER = 'jianpu'
const EDITOR_THEME = 'jianpu'
// Matches preview measure highlight (rgba(255, 200, 0, 0.25)); Monaco only accepts hex.
const MEASURE_HIGHLIGHT_COLOR = '#ffc80040'

export const Editor = forwardRef<EditorHandle, EditorProps>(function Editor(
  {
    path,
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
    onForceSave,
    onEditPartsClick,
    onEditMetadataClick,
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
  const onForceSaveRef = useRef(onForceSave)
  const onEditPartsClickRef = useRef(onEditPartsClick)
  const onEditMetadataClickRef = useRef(onEditMetadataClick)
  const savedSelectionRef = useRef<ISelection | null>(null)
  const isInternalChangeRef = useRef(false)
  const codeLensProviderRef = useRef<IDisposable | null>(null)
  useEffect(() => {
    onSelectionChangeRef.current = onSelectionChange
    onCursorLineChangeRef.current = onCursorLineChange
    onPlayMeasureRef.current = onPlayMeasure
    onForceSaveRef.current = onForceSave
    onEditPartsClickRef.current = onEditPartsClick
    onEditMetadataClickRef.current = onEditMetadataClick
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

  useImperativeHandle(
    ref,
    () => createEditorImperativeHandle(editorRef, monacoRef),
    [],
  )

  useEffect(() => {
    return () => {
      codeLensProviderRef.current?.dispose()
    }
  }, [])

  const handleMount: OnMount = (ed, monacoApi) => {
    editorRef.current = ed
    monacoRef.current = monacoApi
    applyDiagnostics()
    applyMeasureViewZones()
    applyDiagnosticViewZones()

    ed.addCommand(monacoApi.KeyMod.CtrlCmd | monacoApi.KeyCode.Enter, () =>
      onPlayMeasureRef.current?.(),
    )

    ed.addCommand(monacoApi.KeyMod.CtrlCmd | monacoApi.KeyCode.KeyS, () =>
      onForceSaveRef.current?.(),
    )

    const editPartsCommandId = ed.addCommand(0, () => {
      onEditPartsClickRef.current?.()
    })

    const editMetadataCommandId = ed.addCommand(0, () => {
      onEditMetadataClickRef.current?.()
    })

    codeLensProviderRef.current?.dispose()
    codeLensProviderRef.current = monacoApi.languages.registerCodeLensProvider(
      JIANPU_LANGUAGE_ID,
      {
        provideCodeLenses(model: editor.ITextModel) {
          const lenses: languages.CodeLens[] = []
          for (let line = 1; line <= model.getLineCount(); line++) {
            if (model.getLineContent(line).trim() === '# parts') {
              lenses.push({
                range: new monacoApi.Range(line, 1, line, 1),
                command: {
                  id: editPartsCommandId ?? '',
                  title: 'Edit Parts',
                },
              })
            }
            if (model.getLineContent(line).trim() === '# metadata') {
              lenses.push({
                range: new monacoApi.Range(line, 1, line, 1),
                command: {
                  id: editMetadataCommandId ?? '',
                  title: 'Edit Metadata',
                },
              })
            }
          }
          return { lenses, dispose: () => {} }
        },
      },
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
          language={JIANPU_LANGUAGE_ID}
          theme={EDITOR_THEME}
          path={path}
          value={value}
          onChange={(next) => {
            isInternalChangeRef.current = true
            onChange(next ?? '')
          }}
          beforeMount={(monacoApi) => {
            registerJianpuLanguage(monacoApi)
            registerJianpuRenameProvider(monacoApi)
            monacoApi.editor.defineTheme(EDITOR_THEME, {
              base: 'vs',
              inherit: true,
              rules: [
                { token: 'comment', foreground: '008000', fontStyle: 'italic' },
                { token: 'string', foreground: 'a31515' },
                {
                  token: 'keyword.section',
                  foreground: '0000ff',
                  fontStyle: 'bold',
                },
                { token: 'tag', foreground: '267f99', fontStyle: 'bold' },
                { token: 'keyword.directive', foreground: '0000ff' },
                {
                  token: 'keyword.control',
                  foreground: 'af00db',
                  fontStyle: 'bold italic',
                },
                { token: 'type', foreground: '267f99' },
                { token: 'variable', foreground: '001080' },
                { token: 'operator', foreground: '795e26' },
              ],
              colors: {
                'editor.lineHighlightBackground': MEASURE_HIGHLIGHT_COLOR,
                'editor.lineHighlightBorder': '#00000000',
              },
            })
          }}
          onMount={handleMount}
          options={{
            readOnly,
            codeLens: true,
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
