import { expect, type Page, test } from '@playwright/test'
import { focusEditor } from './fileSwitcherHelpers'

declare global {
  interface Window {
    monaco?: typeof import('monaco-editor')
  }
}

/**
 * Switching to Unzipped view auto-applies the Unzipped formatter, which
 * flattens each declared part's notes into one `[Abbrev]`-headed block with
 * one measure per line. A cursor position maps to a measure index via
 * `part_measure_ranges` byte ranges, not via source lines, so newlines
 * within that block — whether inserted by the auto-formatter or typed by
 * the user — are purely cosmetic and must not change which measure a token
 * belongs to.
 *
 * Measure 0 : [M] 1 2 3 4     / [C] 1 - - -
 * Measure 1 : [M] 5 6 7 1'    / [C] 4 - - -
 * Measure 2 : [M] 2 4 6 1'    / [C] 5 - - -
 * Measure 3 : [M] 0 0 0 0     / [C] 1 - - -
 *
 * Unzipped view text for this source is therefore:
 *   [M]
 *   1 2 3 4
 *   5 6 7 1'
 *   2 4 6 1'
 *   0 0 0 0
 *
 *   [C]
 *   1 - - -
 *   4 - - -
 *   5 - - -
 *   1 - - -
 */
const SOURCE = [
  '# metadata',
  'title = "unzipped view highlight test"',
  'max_measures_per_system = 48',
  '',
  '# parts',
  'Melody [M] = notes',
  'Chords [C] = chords',
  '',
  '# score',
  '[M] 1 2 3 4',
  '[C] 1 - - -',
  '',
  "[M] 5 6 7 1'",
  '[C] 4 - - -',
  '',
  "[M] 2 4 6 1'",
  '[C] 5 - - -',
  '',
  '[M] 0 0 0 0',
  '[C] 1 - - -',
].join('\n')

async function loadSource(page: Page) {
  await page.addInitScript((source) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'unzipped-view-highlight-test.jianpu',
        userFiles: { 'unzipped-view-highlight-test.jianpu': source },
        bin: {},
        fileIds: {
          'unzipped-view-highlight-test.jianpu':
            'unzipped-view-highlight-id-001',
        },
      }),
    )
  }, SOURCE)
}

async function toggleUnzippedView(page: Page) {
  await page.locator('.pane-divider-view-toggle').click()
}

/** Clicks the Monaco editor at a given 1-indexed line/column by translating
 * the model position to screen coordinates via Monaco's own API, so the
 * click lands inside the actual rendered token rather than relying on DOM
 * text-node boundaries the tokenizer may or may not create. */
async function clickAtPosition(page: Page, lineNumber: number, column: number) {
  const point = await page.evaluate(
    ({ lineNumber, column }) => {
      const editor = window.monaco?.editor.getEditors()[0]
      if (!editor) return null
      editor.revealPositionInCenter({ lineNumber, column })
      const coords = editor.getScrolledVisiblePosition({ lineNumber, column })
      const domNode = editor.getDomNode()
      if (!coords || !domNode) return null
      const rect = domNode.getBoundingClientRect()
      return {
        x: rect.left + coords.left,
        y: rect.top + coords.top + coords.height / 2,
      }
    },
    { lineNumber, column },
  )
  if (!point) throw new Error('Could not resolve editor screen position')
  await page.mouse.click(point.x, point.y)
}

async function waitForUnzippedText(page: Page) {
  await page.waitForFunction(() => {
    const model = window.monaco?.editor.getEditors()[0]?.getModel()
    return model?.getValue().startsWith('[M]') ?? false
  })
}

test('clicking a token in Unzipped view highlights the corresponding measure', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector('[data-tag="measure"][data-measure-index="3"]', {
    timeout: 15_000,
  })

  await toggleUnzippedView(page)
  await waitForUnzippedText(page)
  await focusEditor(page)

  // Line 4, column 6 lands inside "6" — the third token of measure index 2
  // ("2 4 6 1'", displayed as Measure 3), now on its own line after the
  // auto-formatter breaks each measure onto one.
  await clickAtPosition(page, 4, 6)

  // Allow the 300 ms debounce plus the highlight render worker round-trip.
  await page.waitForTimeout(1_000)

  const playBtn = page.locator('button.play-measure-btn')
  await expect(playBtn).toHaveText(/Measure 3$/, { timeout: 5_000 })

  const highlightRect = page.locator(
    '.preview-page [data-testid="measure-highlight"]',
  )
  await expect(highlightRect.first()).toBeVisible({ timeout: 5_000 })
})

test('typing a partial note in Unzipped view is not clobbered by rest-padding from live re-extraction', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector('[data-tag="measure"][data-measure-index="3"]', {
    timeout: 15_000,
  })

  await toggleUnzippedView(page)
  await waitForUnzippedText(page)
  await focusEditor(page)

  // Land on [M]'s last measure line (measure index 3, "0 0 0 0"), then move
  // to its very end and append a single note. That note starts a new,
  // not-yet-full 5th measure — exactly the case where `merge_unzipped_text`
  // rest-pads the measure (e.g. to "5 0 0 0") once it's merged into
  // `source`. Before the fix, the very next debounced re-extraction would
  // snap the Unzipped editor's own displayed text to that padded form while
  // the user was still typing.
  await clickAtPosition(page, 5, 1)
  await page.keyboard.press('End')
  await page.keyboard.type(' 5')

  // Wait well past the 300ms extraction debounce so any clobbering
  // re-extraction would have already landed.
  await page.waitForTimeout(1_500)

  const mLastLine = await page.evaluate(() => {
    const model = window.monaco?.editor.getEditors()[0]?.getModel()
    return model?.getValue().split(/\r?\n/)[4] ?? ''
  })

  expect(mLastLine.trimEnd()).toBe('0 0 0 0 5')
})

test('a part wrapped across two visual lines still maps clicks to the correct measure', async ({
  page,
}) => {
  await loadSource(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector('[data-tag="measure"][data-measure-index="3"]', {
    timeout: 15_000,
  })

  await toggleUnzippedView(page)
  await waitForUnzippedText(page)
  await focusEditor(page)

  // Line 4 is measure index 2's own line ("2 4 6 1'"). Column 4 sits right
  // before the space separating "4" from "6". Replacing that one space with
  // a newline keeps every later byte offset unchanged, so the still-stale
  // (pre-edit) part_measure_ranges the click below resolves against remain
  // valid — this isolates "does a mid-stream newline break the mapping"
  // from unrelated re-extraction/debounce timing.
  await clickAtPosition(page, 4, 4)
  await page.keyboard.press('Delete')
  await page.keyboard.press('Enter')

  // "6 1'" is now its own visual line (line 5); "6" — still measure index
  // 2, displayed as Measure 3 — sits at column 1 on it.
  await clickAtPosition(page, 5, 1)

  await page.waitForTimeout(1_000)

  const playBtn = page.locator('button.play-measure-btn')
  await expect(playBtn).toHaveText(/Measure 3$/, { timeout: 5_000 })

  const highlightRect = page.locator(
    '.preview-page [data-testid="measure-highlight"]',
  )
  await expect(highlightRect.first()).toBeVisible({ timeout: 5_000 })
})

// Monaco's Unzipped-view model briefly holds empty content while
// `extract_unzipped_text` resolves asynchronously; a model created with
// empty content falls back to the *platform's* default line ending
// (`\r\n` on Windows), and — unless the model is explicitly pinned to
// LF — that CRLF preference sticks for the model's lifetime, since later
// content updates go through `executeEdits` (which normalizes inserted
// text to the model's existing EOL) rather than `setValue` (which would
// re-detect it). The wasm core's `part_measure_ranges` byte offsets are
// always computed against LF-only text, so a CRLF model drifts the
// cursor's computed byte offset one byte per line preceding the cursor,
// silently shifting clicks into the wrong measure the more lines (and
// the more multi-byte UTF-8 content) precede them. Spoof a Windows user
// agent so this reproduces deterministically regardless of the host
// platform running the test.
test.describe('CRLF/LF drift regression', () => {
  test.use({
    userAgent:
      'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
  })

  // Three declared parts, each auto-formatted to one measure per line, means
  // the third part's content starts on line 14 (13 preceding newlines:
  // "[M]\n<4 measure lines>\n\n[Ch]\n<4 measure lines>\n\n[C]\n"), so an
  // unfixed CRLF model drifts the click's computed byte offset forward by 13
  // bytes — enough to cross from measure 0 into measure 1 of the Caption
  // part's non-ASCII ("你好" / "世界" / "測試" / "偏移") content.
  const CRLF_SOURCE = [
    '# metadata',
    'title = "crlf drift test"',
    'max_measures_per_system = 48',
    '',
    '# parts',
    'Melody [M] = notes',
    'Chords [Ch] = chords',
    'Caption [C] = lyrics',
    '',
    '# score',
    '[M] 1 2 3 4',
    '[Ch] 1 - - -',
    '[C] 你好',
    '',
    "[M] 5 6 7 1'",
    '[Ch] 4 - - -',
    '[C] 世界',
    '',
    "[M] 2 4 6 1'",
    '[Ch] 5 - - -',
    '[C] 測試',
    '',
    '[M] 0 0 0 0',
    '[Ch] 1 - - -',
    '[C] 偏移',
  ].join('\n')

  async function loadCrlfSource(page: Page) {
    await page.addInitScript((source) => {
      localStorage.setItem(
        'jianpu:files:v1',
        JSON.stringify({
          active: 'crlf-drift-test.jianpu',
          userFiles: { 'crlf-drift-test.jianpu': source },
          bin: {},
          fileIds: { 'crlf-drift-test.jianpu': 'crlf-drift-test-id-001' },
        }),
      )
    }, CRLF_SOURCE)
  }

  test('clicking the start of a later part still maps to the correct measure when preceded by several lines of non-ASCII content', async ({
    page,
  }) => {
    await loadCrlfSource(page)
    await page.goto('/')

    await page.waitForSelector('[data-testid="play-measure-button"]', {
      timeout: 15_000,
    })
    await page.waitForSelector('[data-tag="measure"][data-measure-index="3"]', {
      timeout: 15_000,
    })

    await toggleUnzippedView(page)
    await waitForUnzippedText(page)
    await focusEditor(page)

    // Line 14, column 1 is the very first character of the Caption part's
    // first measure line ("你好") — the start of measure index 0. With an
    // unfixed CRLF model, the drift lands exactly on the start of measure
    // index 1 instead.
    await clickAtPosition(page, 14, 1)

    await page.waitForTimeout(1_000)

    const playBtn = page.locator('button.play-measure-btn')
    await expect(playBtn).toHaveText(/Measure 1$/, { timeout: 5_000 })
  })
})
