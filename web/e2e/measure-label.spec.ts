import { expect, test } from '@playwright/test'

/**
 * The default demo source (reference.jianpu) has the following
 * Monaco line numbers (1-based):
 *
 *   1  # metadata
 *   2  title = "Jianpu Postcard"
 *   ...
 *  14  # score
 *  15  [M] 0 0 0 0    ← first note line → measure 1
 */
test('shows measure number when cursor is placed on a note line', async ({
  page,
}) => {
  await page.goto('/')

  // The PlayMeasureButton toolbar is only rendered once the WASM module reports
  // audioAvailable=true.  Wait up to 15 s for it to appear.
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })

  // Focus the Monaco editor.
  await page.click('.monaco-editor .view-lines')

  // Use Monaco's "Go to Line" command (Ctrl+G) to jump to line 15,
  // which is the first note line in the default reference.jianpu source.
  await page.keyboard.press('Control+g')
  await page.keyboard.type('15')
  await page.keyboard.press('Enter')

  // Allow the 300 ms debounce in notifySelection plus worker round-trip.
  await page.waitForTimeout(700)

  // The label should show "measure 1", not "measure null".
  await expect(page.getByTestId('measure-status')).toContainText('measure 1', {
    timeout: 3_000,
  })
})

/**
 * Regression: when the cursor is positioned directly AFTER the last character
 * of a note line, the byte offset equals source_span.end and the measure must
 * still be detected.
 *
 * Line 15 of the default demo is "[M] 0 0 0 0" — the entire span of
 * measure 1 (the following line 16 is blank). Pressing End places the
 * cursor after the trailing "0", at the end of the measure's span.
 */
test('detects measure when cursor is at end of last character of a note line', async ({
  page,
}) => {
  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })

  // Focus the Monaco editor.
  await page.click('.monaco-editor .view-lines')

  // Navigate to line 15 ("[M] 0 0 0 0") and press End to put the cursor
  // after the trailing "0" — the last character of the measure span.
  await page.keyboard.press('Control+g')
  await page.keyboard.type('15')
  await page.keyboard.press('Enter')
  await page.keyboard.press('End')

  // Allow the 300 ms debounce in notifySelection plus worker round-trip.
  await page.waitForTimeout(700)

  // Should still detect measure 1, not "measure null".
  await expect(page.getByTestId('measure-status')).toContainText('measure 1', {
    timeout: 3_000,
  })
})

/**
 * Regression: when the cursor is at the end of a Chinese lyric line that is the
 * last line of the measure, the byte offset equals span.end (exclusive) and
 * measureRangeInSpan's strict `span.end > selStart` check returns null → no label.
 *
 * The lyric line "白陽旗旛在大道盛宏" is line 16 (1-based) in the source below.
 * Pressing End places the cursor after "宏" (3-byte UTF-8 char), byte offset = span.end.
 */
test('detects measure when cursor is at end of last character of a Chinese lyric line', async ({
  page,
}) => {
  const source = [
    '# metadata',
    'title = "abc"',
    'author = "author"',
    '',
    '# parts',
    'Chord = chords',
    'Alto 1 & Tenor (A1,T) = notes+lyrics',
    '',
    '',
    '# score',
    '',
    '',
    '(bpm=80 key=C4 time=4/4 label="Verse 1")',
    '1 - - -',
    '5_ 5_ 5_ 5= 5= 5_ 3_ 2_ (3_',
    '白陽旗旛在大道盛宏',
    '',
    '',
    '6m/3',
    '3_) (1_1-) 0_ 1= 1=',
    '昌花花',
  ].join('\n')

  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'chinese-test.jianpu',
        userFiles: { 'chinese-test.jianpu': src },
        bin: {},
        fileIds: { 'chinese-test.jianpu': crypto.randomUUID() },
      }),
    )
  }, source)

  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })

  await page.click('.monaco-editor .view-lines')

  // Go to line 16 ("白陽旗旛在大道盛宏") and press End to place cursor after "宏".
  await page.keyboard.press('Control+g')
  await page.keyboard.type('16')
  await page.keyboard.press('Enter')
  await page.keyboard.press('End')

  // Allow 300 ms debounce + worker round-trip.
  await page.waitForTimeout(700)

  // Must show "measure 1" (first measure, index 0).
  await expect(page.getByTestId('measure-status')).toContainText('measure 1', {
    timeout: 3_000,
  })
})
