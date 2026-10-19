import { DiffEditor, type Monaco } from '@monaco-editor/react'
import type { editor } from 'monaco-editor'
import { useEffect, useMemo, useRef, useState } from 'react'
import {
  jianpuWasm,
  type PartStream,
  type StreamablePart,
} from '../../jianpuWasm'
import {
  JIANPU_LANGUAGE_ID,
  registerJianpuLanguage,
} from '../../monacoJianpuLanguage'
import { buildDiagnosticMarkers } from '../editorDiagnosticViewZones'
import { EDITOR_THEME } from '../editorTheme'
import { measureMarkerDecorations } from './partStreamMarkers'

const MARKER_OWNER = 'part-stream'

interface PartStreamDiffEditorProps {
  /** The main editor's current source; the stream is applied onto it. */
  source: string
  part: StreamablePart
  /** The stream as extracted when the part was chosen (the diff's left side). */
  stream: PartStream
  onApply: (newSource: string) => void
  onCancel: () => void
}

interface MountedEditor {
  modified: editor.IStandaloneCodeEditor
  monacoApi: Monaco
  measureMarkers: editor.IEditorDecorationsCollection
}

function changedMeasuresLabel(count: number): string {
  return `${count} ${count === 1 ? 'measure' : 'measures'} changed`
}

/**
 * The extracted stream (read-only) beside an editable copy. Every edit is
 * handed to Rust's `applyPartStream`, whose diagnostics, measure starts and
 * changed-measure count are rendered as is; its `source` is what Apply
 * writes back. Nothing is written until Apply.
 */
export function PartStreamDiffEditor({
  source,
  part,
  stream,
  onApply,
  onCancel,
}: PartStreamDiffEditorProps) {
  const [editedText, setEditedText] = useState(stream.text)
  const [mounted, setMounted] = useState<MountedEditor | null>(null)
  const mountedRef = useRef<MountedEditor | null>(null)
  const result = useMemo(
    () => jianpuWasm().applyPartStream(source, part.abbreviation, editedText),
    [source, part.abbreviation, editedText],
  )

  useEffect(() => {
    if (!mounted) return
    const model = mounted.modified.getModel()
    if (!model) return
    mounted.monacoApi.editor.setModelMarkers(
      model,
      MARKER_OWNER,
      buildDiagnosticMarkers(model, mounted.monacoApi, result.diagnostics),
    )
    mounted.measureMarkers.set(
      measureMarkerDecorations(
        mounted.monacoApi,
        editedText,
        result.measureStarts,
      ),
    )
  }, [mounted, result, editedText])

  useEffect(
    () => () => {
      mountedRef.current?.monacoApi.editor.removeAllMarkers(MARKER_OWNER)
    },
    [],
  )

  const appliedSource = result.source
  const canApply = appliedSource !== undefined && result.changedMeasureCount > 0
  const errorCount = result.diagnostics.filter(
    (diagnostic) => diagnostic.severity === 'error',
  ).length

  return (
    <div className="part-stream-editor" data-testid="part-stream-editor">
      <div className="part-stream-editor__diff">
        <DiffEditor
          height="100%"
          language={JIANPU_LANGUAGE_ID}
          theme={EDITOR_THEME}
          original={stream.text}
          modified={stream.text}
          beforeMount={registerJianpuLanguage}
          options={{
            readOnly: false,
            originalEditable: false,
            wordWrap: 'on',
            diffWordWrap: 'on',
            minimap: { enabled: false },
            fontFamily: 'var(--mono)',
            fontSize: 14,
            lineHeight: 24,
            renderValidationDecorations: 'on',
            scrollBeyondLastLine: false,
          }}
          onMount={(diffEditor, monacoApi) => {
            const original = diffEditor.getOriginalEditor()
            original.createDecorationsCollection(
              measureMarkerDecorations(
                monacoApi,
                stream.text,
                stream.measureStarts,
              ),
            )
            const modified = diffEditor.getModifiedEditor()
            modified.onDidChangeModelContent(() => {
              const model = modified.getModel()
              if (!model) return
              setEditedText(
                model.getValue(monacoApi.editor.EndOfLinePreference.LF),
              )
            })
            const editorState: MountedEditor = {
              modified,
              monacoApi,
              measureMarkers: modified.createDecorationsCollection(),
            }
            mountedRef.current = editorState
            setMounted(editorState)
            modified.focus()
          }}
        />
      </div>
      <div className="part-stream-editor__footer">
        <span data-testid="part-stream-footer" role="status">
          {changedMeasuresLabel(result.changedMeasureCount)}
          {errorCount > 0 &&
            ` · ${errorCount} ${errorCount === 1 ? 'error' : 'errors'}`}
        </span>
        <div className="part-stream-editor__actions">
          <button type="button" onClick={onCancel}>
            Cancel
          </button>
          <button
            type="button"
            className="part-stream-editor__apply"
            disabled={!canApply}
            onClick={() => {
              if (appliedSource !== undefined) onApply(appliedSource)
            }}
          >
            Apply
          </button>
        </div>
      </div>
    </div>
  )
}
