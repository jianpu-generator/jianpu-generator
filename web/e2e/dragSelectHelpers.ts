import type { Page } from '@playwright/test'

/**
 * Drives the preview's click-and-click range-selection gesture (see
 * `previewClickHandler.ts`/`usePreviewClickSelection.ts`): a first click at
 * `(startX, startY)` anchors the selection, a `mousemove` to `(endX, endY)`
 * live-updates the hover preview (mirroring what a real mouse user sees
 * between the two clicks), and a second click at `(endX, endY)` resolves and
 * commits the range. Replaces the old held-button drag
 * (`mouse.down()` → `mouse.move({ steps })` → `mouse.up()`) every preview
 * drag-select e2e step used before the gesture became click-and-click.
 */
export async function clickAndClickSelect(
  page: Page,
  startX: number,
  startY: number,
  endX: number,
  endY: number,
  moveSteps = 10,
): Promise<void> {
  await page.mouse.move(startX, startY)
  await page.mouse.down()
  await page.mouse.up() // click #1 — anchors
  await page.mouse.move(endX, endY, { steps: moveSteps }) // hover preview
  await page.mouse.down()
  await page.mouse.up() // click #2 — commits
}
