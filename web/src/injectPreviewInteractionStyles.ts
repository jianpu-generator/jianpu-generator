import {
  DATA_RANGE_ACTIVE_FLAG,
  groupTagSelector,
  rectVariantSelector,
} from './dataAttributes'

// The hover/selection fills painted onto the preview SVG's click-target
// rects. These selectors name `data-tag`/`data-variant` values that are
// jco-generated WIT types, so they're built here from the typed selector
// helpers instead of being hand-typed in a plain CSS file, where a renamed
// WIT case would compile clean and silently match nothing.

const HOVER_FILL = 'rgba(100, 160, 255, 0.2)'
const SELECTED_FILL = 'rgba(37, 99, 235, 0.25)'
const LABEL_ACTIVE_FILL = 'rgba(37, 99, 235, 0.12)'

interface FillRule {
  selectors: string[]
  fill: string
}

const PREVIEW_INTERACTION_RULES: FillRule[] = [
  {
    selectors: [
      `.preview-page svg g${groupTagSelector('section-label')}:hover > ${rectVariantSelector('section-label-background')}`,
    ],
    fill: HOVER_FILL,
  },
  {
    selectors: [
      `.preview-page svg g${groupTagSelector('note')}:hover > ${rectVariantSelector('note-click-target')}`,
    ],
    fill: HOVER_FILL,
  },
  {
    selectors: [
      `.preview-page svg g${groupTagSelector('lyric')}:hover > ${rectVariantSelector('lyric-click-target')}`,
    ],
    fill: HOVER_FILL,
  },
  {
    selectors: [
      `.preview-page svg g${groupTagSelector('bar-number')}:hover > ${rectVariantSelector('bar-number-click-target')}`,
    ],
    fill: HOVER_FILL,
  },
  {
    selectors: [
      `.preview-page svg g${groupTagSelector('bar-line')}:hover > ${rectVariantSelector('bar-line-click-target')}`,
    ],
    fill: 'rgba(100, 160, 255, 0.45)',
  },
  {
    selectors: [
      `${groupTagSelector('note')}[data-note-range-selected] ${rectVariantSelector('note-click-target')}`,
    ],
    fill: SELECTED_FILL,
  },
  {
    selectors: [
      `${groupTagSelector('lyric')}[data-lyric-range-selected] ${rectVariantSelector('lyric-click-target')}`,
    ],
    fill: SELECTED_FILL,
  },
  // A click-and-click gesture's anchor, while it's still waiting on a second
  // click (`usePreviewClickSelection`'s `pendingSecondClick` mirrored onto
  // the preview container as `data-pending-selection`), paints in this amber
  // "pending" color instead of the committed-selection blue above, so
  // mid-gesture never reads as an already-final selection. These win on
  // specificity (one extra ancestor attribute selector) whenever the
  // container carries the pending flag.
  {
    selectors: [
      `[data-pending-selection] ${groupTagSelector('note')}[data-note-range-selected] ${rectVariantSelector('note-click-target')}`,
    ],
    fill: 'rgba(217, 119, 6, 0.25)',
  },
  {
    selectors: [
      `[data-pending-selection] ${groupTagSelector('lyric')}[data-lyric-range-selected] ${rectVariantSelector('lyric-click-target')}`,
    ],
    fill: 'rgba(217, 119, 6, 0.25)',
  },
  {
    selectors: [
      `${rectVariantSelector('part-label-click-target')}:hover`,
      `${rectVariantSelector('part-label-click-target')}[${DATA_RANGE_ACTIVE_FLAG.partLabel}]`,
    ],
    fill: LABEL_ACTIVE_FILL,
  },
  {
    selectors: [
      `${rectVariantSelector('lyric-label-click-target')}:hover`,
      `${rectVariantSelector('lyric-label-click-target')}[${DATA_RANGE_ACTIVE_FLAG.lyricLabel}]`,
    ],
    fill: LABEL_ACTIVE_FILL,
  },
]

/** Builds the stylesheet text — split out from
 * `injectPreviewInteractionStyles` so it's testable without a DOM. */
export function buildPreviewInteractionCss(): string {
  return PREVIEW_INTERACTION_RULES.map(
    ({ selectors, fill }) => `${selectors.join(',\n')} {\n  fill: ${fill};\n}`,
  ).join('\n\n')
}

/** Call once, before the app renders. */
export function injectPreviewInteractionStyles(): void {
  const style = document.createElement('style')
  style.textContent = buildPreviewInteractionCss()
  document.head.appendChild(style)
}
