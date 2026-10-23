import { DiffEditor, type DiffOnMount } from '@monaco-editor/react'
import type { DiffLayout } from '../hooks/useDiffLayout'
import {
  JIANPU_LANGUAGE_ID,
  registerJianpuLanguage,
} from '../monacoJianpuLanguage'
import { EDITOR_THEME } from './editorTheme'
import './FileDialogs.css'

interface JianpuDiffEditorProps {
  original: string
  modified: string
  layout: DiffLayout
  /** Whether the modified side can be edited; the original never can. */
  editable?: boolean
  onMount?: DiffOnMount
}

/** A Monaco diff of two `.jianpu` texts, highlighted like the main editor,
 * with changed words marked inside each changed line. Fills its parent. */
export function JianpuDiffEditor({
  original,
  modified,
  layout,
  editable = false,
  onMount,
}: JianpuDiffEditorProps) {
  return (
    <DiffEditor
      height="100%"
      language={JIANPU_LANGUAGE_ID}
      theme={EDITOR_THEME}
      original={original}
      modified={modified}
      beforeMount={registerJianpuLanguage}
      options={{
        readOnly: !editable,
        originalEditable: false,
        renderSideBySide: layout === 'side-by-side',
        // Otherwise Monaco forces inline below ~900px, overriding the toggle.
        useInlineViewWhenSpaceIsLimited: false,
        wordWrap: 'on',
        diffWordWrap: 'on',
        minimap: { enabled: false },
        fontFamily: 'var(--mono)',
        fontSize: 14,
        lineHeight: 24,
        renderValidationDecorations: 'on',
        scrollBeyondLastLine: false,
      }}
      onMount={onMount}
    />
  )
}

/** Switches a `JianpuDiffEditor` between inline and side by side. */
export function DiffLayoutToggle({
  layout,
  onToggle,
  testId,
}: {
  layout: DiffLayout
  onToggle: () => void
  testId: string
}) {
  return (
    <button
      type="button"
      className="file-dialog-button"
      data-testid={testId}
      onClick={onToggle}
    >
      {layout === 'inline' ? 'Side by side' : 'Inline'}
    </button>
  )
}
