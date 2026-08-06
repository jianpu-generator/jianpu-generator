import type { Monaco } from '@monaco-editor/react'
import type { PartMeasureRangesOut } from 'jianpu-wasm'
import type { editor } from 'monaco-editor'
import type {
  ByteSpan,
  Diagnostic,
  DiagnosticMessage,
  DiagnosticViewZone,
  MeasureSpan,
} from '../types'
import { byteOffsetToStringIndex } from '../utils/byteSpan'
import { mapZippedSpanToUnzippedRange } from '../utils/diagnosticSpanMapping'

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

/** Builds Monaco marker data for `diagnostics`, relocating each span onto
 * the Unzipped view's text when `unzippedView` is true (see
 * `mapZippedSpanToUnzippedRange`); diagnostics with no Unzipped-text
 * position are dropped. */
export function buildDiagnosticMarkers(
  model: editor.ITextModel,
  monacoApi: Monaco,
  diagnostics: Diagnostic[],
  unzippedView: boolean,
  measureSpans: MeasureSpan[],
  partMeasureRanges: PartMeasureRangesOut[],
): editor.IMarkerData[] {
  const source = model.getValue()
  const visible = unzippedView
    ? diagnostics.flatMap((d) => {
        const span = mapZippedSpanToUnzippedRange(
          d.span,
          measureSpans,
          partMeasureRanges,
        )
        return span ? [{ ...d, span }] : []
      })
    : diagnostics

  return visible.map((d) => {
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

interface UnzippedViewZoneGroup {
  line: number
  severityOrder: number
  severity: 'error' | 'warning'
  messages: DiagnosticMessage[]
}

/** Mirrors `group_diagnostics_into_view_zones` (crates/jianpu-wasm/src/types.rs),
 * which the server runs against the Zipped source's line numbers — those
 * line numbers are meaningless against the Unzipped view's generated text,
 * so this rebuilds the same grouping from `diagnostics` after relocating
 * each span via `mapZippedSpanToUnzippedRange`. Diagnostics outside any
 * measure (e.g. `# metadata`/`# parts` errors) have no Unzipped-text
 * position and are dropped. */
export function buildUnzippedDiagnosticViewZones(
  model: editor.ITextModel,
  diagnostics: Diagnostic[],
  measureSpans: MeasureSpan[],
  partMeasureRanges: PartMeasureRangesOut[],
): DiagnosticViewZone[] {
  const source = model.getValue()
  const groups = new Map<string, UnzippedViewZoneGroup>()

  for (const diagnostic of diagnostics) {
    const range = mapZippedSpanToUnzippedRange(
      diagnostic.span,
      measureSpans,
      partMeasureRanges,
    )
    if (!range) continue

    const index = byteOffsetToStringIndex(source, range.end)
    const line = model.getPositionAt(index).lineNumber
    const severityOrder = diagnostic.severity === 'warning' ? 1 : 0
    const key = `${line}:${severityOrder}`
    const message = { message: diagnostic.message, report: diagnostic.report }

    const existing = groups.get(key)
    if (existing) {
      existing.messages.push(message)
    } else {
      groups.set(key, {
        line,
        severityOrder,
        severity: diagnostic.severity,
        messages: [message],
      })
    }
  }

  return [...groups.values()]
    .sort((a, b) => a.line - b.line || a.severityOrder - b.severityOrder)
    .map((group) => ({
      severity: group.severity,
      after_line_number: group.line,
      messages: group.messages,
    }))
}

/** Picks the server-computed `diagnosticViewZones` (Zipped view) or rebuilds
 * them against the Unzipped view's text (see `buildUnzippedDiagnosticViewZones`). */
export function resolveDiagnosticViewZones(
  model: editor.ITextModel,
  unzippedView: boolean,
  diagnostics: Diagnostic[],
  diagnosticViewZones: DiagnosticViewZone[],
  measureSpans: MeasureSpan[],
  partMeasureRanges: PartMeasureRangesOut[],
): DiagnosticViewZone[] {
  return unzippedView
    ? buildUnzippedDiagnosticViewZones(
        model,
        diagnostics,
        measureSpans,
        partMeasureRanges,
      )
    : diagnosticViewZones
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
