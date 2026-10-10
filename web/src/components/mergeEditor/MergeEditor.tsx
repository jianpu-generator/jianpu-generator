import { DiffEditor } from '@monaco-editor/react'
import { type ReactElement, useState } from 'react'
import {
  containsConflictMarkers,
  type MergeEditorProps,
} from './mergeEditorTypes.ts'

export interface MergeEditorViewProps extends MergeEditorProps {
  /** Live text of the editable result; decides whether Save is enabled. */
  result: string
  markersRemain: boolean
  onResultChange: (result: string) => void
}

/** Hook-free layout, kept separate so it can be tested without Monaco. */
export function MergeEditorView(props: MergeEditorViewProps): ReactElement {
  const { markersRemain, result } = props
  return (
    <div className="merge-editor" data-testid="merge-editor">
      <h2 className="merge-editor__title">{props.fileName}</h2>
      <div className="merge-editor__diff" style={{ height: '60vh' }}>
        <DiffEditor
          original={props.theirs}
          modified={props.initialResult}
          options={{ readOnly: false, originalEditable: false }}
          onMount={(diffEditor) => {
            const modified = diffEditor.getModifiedEditor()
            modified.onDidChangeModelContent(() =>
              props.onResultChange(modified.getValue()),
            )
          }}
        />
      </div>
      {markersRemain && (
        <p className="merge-editor__hint" role="status">
          Resolve every conflict marker (&lt;&lt;&lt;&lt;&lt;&lt;&lt;, =======,
          &gt;&gt;&gt;&gt;&gt;&gt;&gt;) before saving.
        </p>
      )}
      <div className="merge-editor__actions">
        <button
          type="button"
          disabled={markersRemain}
          onClick={() => props.onSave(result)}
        >
          Save
        </button>
        <button type="button" onClick={() => props.onKeepBoth()}>
          Keep both files
        </button>
        <button type="button" onClick={() => props.onDiscard()}>
          Discard
        </button>
      </div>
    </div>
  )
}

export function MergeEditor(props: MergeEditorProps): ReactElement {
  const [result, setResult] = useState(props.initialResult)
  const [edited, setEdited] = useState(false)
  const markersRemain = edited
    ? containsConflictMarkers(result)
    : props.hasConflictMarkers
  return (
    <MergeEditorView
      {...props}
      result={result}
      markersRemain={markersRemain}
      onResultChange={(next) => {
        setEdited(true)
        setResult(next)
      }}
    />
  )
}
