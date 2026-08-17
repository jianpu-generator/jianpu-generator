import type { Monaco } from '@monaco-editor/react'
import type { editor } from 'monaco-editor'
import type { ByteSpan, Diagnostic, DiagnosticMessage } from '../types'
import { byteOffsetToStringIndex } from '../utils/byteSpan'

export function diagnosticRange(
  model: editor.ITextModel,
  source: string,
  span: ByteSpan,
  monacoApi: Monaco,
) {
  const startIndex = byteOffsetToStringIndex(source, span.start)
  const endIndex = Math.max(
    startIndex + 1,
    byteOffsetToStringIndex(source, span.end),
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

/** Builds Monaco marker data for `diagnostics`. */
export function buildDiagnosticMarkers(
  model: editor.ITextModel,
  monacoApi: Monaco,
  diagnostics: Diagnostic[],
): editor.IMarkerData[] {
  const source = model.getValue()

  return diagnostics.map((d) => {
    const range = diagnosticRange(model, source, d.span, monacoApi)
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
}

export const ERROR_ZONE_LINE_HEIGHT_PX = 21

export function createDiagnosticViewZoneDomNode(
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
  }

  return domNode
}

export function errorViewZoneHeightInPx(
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
