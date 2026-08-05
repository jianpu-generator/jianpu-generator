import type { editor } from 'monaco-editor'
import type { MeasureSpan } from '../types'
import { byteOffsetToStringIndex } from '../utils/byteSpan'

export function createMeasureViewZoneDomNode(
  span: MeasureSpan,
  index: number,
): HTMLElement {
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
  return domNode
}

export function measureViewZoneLineNumber(
  model: editor.ITextModel,
  source: string,
  span: MeasureSpan,
): number {
  const stringIndex = byteOffsetToStringIndex(source, span.view_zone_start)
  return model.getPositionAt(stringIndex).lineNumber
}
