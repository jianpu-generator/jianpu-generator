import type { StaffMeasure } from '../jianpuWasm'
import type { MeasureRange } from './previewSelection'

const SVG_NS = 'http://www.w3.org/2000/svg'

/** `data-testid` of the rect painted behind each selected staff measure. */
export const STAFF_MEASURE_HIGHLIGHT_TEST_ID = 'staff-measure-highlight'

/** `data-testid` of each staff measure's transparent hover/click target. */
export const STAFF_MEASURE_HIT_TARGET_TEST_ID = 'staff-measure-hit-target'

interface StaffMeasureElement {
  element: SVGGraphicsElement
  measureIndex: number
}

/** Each measure's group in Verovio's SVG, found by the element id Rust
 * listed for it (`StaffMeasure`), so the id format stays Rust's alone. */
function staffMeasureElements(
  container: HTMLElement,
  measures: StaffMeasure[],
): StaffMeasureElement[] {
  return measures.flatMap(({ elementId, measureIndex }) => {
    const element = container.querySelector<SVGGraphicsElement>(
      `#${CSS.escape(elementId)}`,
    )
    return element ? [{ element, measureIndex }] : []
  })
}

/** The staff measure whose box contains `(x, y)`. Tested against each
 * measure's whole box rather than the element under the pointer, since the
 * blank space between a measure's notes isn't painted and so never hit. */
export function staffMeasureRangeAtPoint(
  container: HTMLElement,
  measures: StaffMeasure[],
  x: number,
  y: number,
): MeasureRange | undefined {
  const hit = staffMeasureElements(container, measures).find(({ element }) => {
    const box = element.getBoundingClientRect()
    return x >= box.left && x <= box.right && y >= box.top && y <= box.bottom
  })
  return hit && { start: hit.measureIndex, end: hit.measureIndex }
}

/** The group of the staff measure `measureIndex`, to scroll into view. */
export function staffMeasureElement(
  container: HTMLElement,
  measures: StaffMeasure[],
  measureIndex: number,
): Element | undefined {
  return staffMeasureElements(container, measures).find(
    (measure) => measure.measureIndex === measureIndex,
  )?.element
}

/** A rect covering `element`'s whole box, inserted behind its contents. */
function insertMeasureRect(
  element: SVGGraphicsElement,
  className: string,
  testId: string,
): void {
  const box = element.getBBox()
  const rect = document.createElementNS(SVG_NS, 'rect')
  rect.setAttribute('x', String(box.x))
  rect.setAttribute('y', String(box.y))
  rect.setAttribute('width', String(box.width))
  rect.setAttribute('height', String(box.height))
  rect.setAttribute('class', className)
  rect.dataset.testid = testId
  element.insertBefore(rect, element.firstChild)
}

/** Gives every staff measure a transparent rect over its whole box, so the
 * blank space between its notes (which Verovio leaves unpainted) is hovered
 * too — `preview.css` tints it and shows a pointer, like the jianpu view's
 * click targets. Run once per set of pages. */
export function applyStaffMeasureHitTargets(
  container: HTMLElement,
  measures: StaffMeasure[],
): void {
  for (const { element } of staffMeasureElements(container, measures)) {
    if (
      element.querySelector(
        `:scope > [data-testid="${STAFF_MEASURE_HIT_TARGET_TEST_ID}"]`,
      )
    )
      continue
    insertMeasureRect(
      element,
      'staff-measure-hit-target',
      STAFF_MEASURE_HIT_TARGET_TEST_ID,
    )
  }
}

/** Paints a rect behind every staff measure inside `ranges`, clearing the
 * previous ones. Idempotent, so it's re-run whenever Verovio's pages are
 * swapped in. */
export function applyStaffMeasureHighlights(
  container: HTMLElement,
  measures: StaffMeasure[],
  ranges: MeasureRange[],
): void {
  for (const rect of Array.from(
    container.querySelectorAll(
      `[data-testid="${STAFF_MEASURE_HIGHLIGHT_TEST_ID}"]`,
    ),
  )) {
    rect.remove()
  }
  for (const { element, measureIndex } of staffMeasureElements(
    container,
    measures,
  )) {
    const selected = ranges.some(
      ({ start, end }) => start <= measureIndex && measureIndex <= end,
    )
    if (!selected) continue
    insertMeasureRect(
      element,
      'staff-measure-highlight',
      STAFF_MEASURE_HIGHLIGHT_TEST_ID,
    )
  }
}
