import type { Monaco } from '@monaco-editor/react'
import type { editor } from 'monaco-editor'
import type { Diagnostic, DiagnosticMessage } from '../types'
import { byteOffsetToStringIndex } from '../utils/byteSpan'

export function diagnosticRange(
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

    if (msg.report) {
      const report = document.createElement('pre')
      report.className = 'editor-error-zone-report'
      report.textContent = msg.report
      domNode.appendChild(report)
    }
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
