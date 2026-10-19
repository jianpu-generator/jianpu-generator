import { expect, type Page } from '@playwright/test'
import { Given, Then, When } from './fixtures'

declare global {
  interface Window {
    monaco?: typeof import('monaco-editor')
  }
}

const SOURCE = `# parts
Melody [M] = notes
Alto [A] = follow[M]
Words [W] = lyrics[M]

# score
[M] 1 2 3 4
[W] a b c d

[M] 5 6 7 1
[A] 3 4 5 6
[W] e f g h
`

const SHIFTED_MELODY_SOURCE = `# parts
Melody [M] = notes
Alto [A] = follow[M]
Words [W] = lyrics[M]

# score
[M] 0 1 2 3
[W] a b c d

[M] 4 5 6 7
[A] 3 4 5 6
[W] e f g h

[M] 1 0 0 0
`

const SHORTENED_MELODY_SOURCE = `# parts
Melody [M] = notes
Alto [A] = follow[M]
Words [W] = lyrics[M]

# score
[M] 2 3 4 5
[W] a b c d

[M] 6 7 1 0
[A] 3 4 5 6
[W] e f g h
`

const SHIFTED_LYRICS_SOURCE = `# parts
Melody [M] = notes
Alto [A] = follow[M]
Words [W] = lyrics[M]

# score
[M] 1 2 3 4
[W] z a b c

[M] 5 6 7 1
[A] 3 4 5 6
[W] d e f g
`

const CHORD_BOUND_LYRICS_SOURCE = `# parts
Harmony [H] = chords
Chant [C] = lyrics[H]

# score
[H] 1 4 5 1
[C] (a) (b) (c) (d)
`

const EXPECTED_SOURCES: Record<string, string> = {
  'original fixture': SOURCE,
  'shifted melody': SHIFTED_MELODY_SOURCE,
  'shortened melody': SHORTENED_MELODY_SOURCE,
  'shifted lyrics': SHIFTED_LYRICS_SOURCE,
}

async function loadSource(page: Page, source: string) {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'stream.jianpu',
        userFiles: { 'stream.jianpu': src },
        bin: {},
        fileIds: { 'stream.jianpu': crypto.randomUUID() },
      }),
    )
  }, source)
  await page.goto('/')
  await page.waitForSelector('.editor-surface .monaco-editor .view-lines', {
    timeout: 30_000,
  })
}

/** Reads one of the three Monaco editors on the page: the main source
 * editor, or the part stream diff editor's original/modified side. */
async function editorValue(
  page: Page,
  which: 'main' | 'original' | 'modified',
): Promise<string | null> {
  return page.evaluate((side) => {
    const container =
      side === 'main'
        ? document.querySelector('.editor-surface')
        : document.querySelector(
            `[data-testid="part-stream-editor"] .editor.${side}`,
          )
    const editor = window.monaco?.editor
      .getEditors()
      .find((candidate) => container?.contains(candidate.getContainerDomNode()))
    return editor?.getModel()?.getValue() ?? null
  }, which)
}

Given('the part stream editor fixture is loaded', async ({ page }) => {
  await loadSource(page, SOURCE)
})

Given('the chord-bound lyrics fixture is loaded instead', async ({ page }) => {
  await loadSource(page, CHORD_BOUND_LYRICS_SOURCE)
})

When('I open the part stream picker', async ({ page }) => {
  await page.click('.editor-toolbar-button[aria-label="Edit part as stream"]')
  await expect(page.getByTestId('part-stream-modal')).toBeVisible()
})

When(
  'I open the part stream editor for part {string}',
  async ({ page }, abbreviation: string) => {
    await page.click('.editor-toolbar-button[aria-label="Edit part as stream"]')
    await page
      .locator(
        `[data-testid="part-stream-part-row"][data-abbreviation="${abbreviation}"]`,
      )
      .click()
    await expect
      .poll(() => editorValue(page, 'modified'), { timeout: 15_000 })
      .not.toBeNull()
  },
)

When(
  'I replace the edited part stream with {string}',
  async ({ page }, text: string) => {
    await page.evaluate((replacement) => {
      const container = document.querySelector(
        '[data-testid="part-stream-editor"] .editor.modified',
      )
      const editor = window.monaco?.editor
        .getEditors()
        .find((candidate) =>
          container?.contains(candidate.getContainerDomNode()),
        )
      const model = editor?.getModel()
      if (!editor || !model) throw new Error('no modified stream editor')
      editor.executeEdits('e2e', [
        { range: model.getFullModelRange(), text: replacement },
      ])
    }, text)
    await expect.poll(() => editorValue(page, 'modified')).toBe(text)
  },
)

Then('the original part stream is:', async ({ page }, text: string) => {
  await expect.poll(() => editorValue(page, 'original')).toBe(text)
})

Then('the edited part stream is:', async ({ page }, text: string) => {
  await expect.poll(() => editorValue(page, 'modified')).toBe(text)
})

Then('the part stream footer says {string}', async ({ page }, text: string) => {
  await expect(page.getByTestId('part-stream-footer')).toContainText(text)
})

Then('the part stream Apply button is enabled', async ({ page }) => {
  await expect(
    page.getByRole('button', { name: 'Apply', exact: true }),
  ).toBeEnabled()
})

Then('the part stream Apply button is disabled', async ({ page }) => {
  await expect(
    page.getByRole('button', { name: 'Apply', exact: true }),
  ).toBeDisabled()
})

Then('the edited part stream has an error marker', async ({ page }) => {
  await expect
    .poll(() =>
      page.evaluate(() => {
        const container = document.querySelector(
          '[data-testid="part-stream-editor"] .editor.modified',
        )
        const monaco = window.monaco
        const model = monaco?.editor
          .getEditors()
          .find((candidate) =>
            container?.contains(candidate.getContainerDomNode()),
          )
          ?.getModel()
        if (!monaco || !model) return 0
        return monaco.editor
          .getModelMarkers({ resource: model.uri })
          .filter((marker) => marker.severity === monaco.MarkerSeverity.Error)
          .length
      }),
    )
    .toBeGreaterThan(0)
})

When('I apply the part stream', async ({ page }) => {
  await page.getByRole('button', { name: 'Apply', exact: true }).click()
})

When('I press Escape in the part stream editor', async ({ page }) => {
  await page
    .locator('[data-testid="part-stream-editor"] .editor.modified .view-lines')
    .click()
  await page.keyboard.press('Escape')
})

When('I click Cancel in the part stream editor', async ({ page }) => {
  await page.getByRole('button', { name: 'Cancel', exact: true }).click()
})

When('I resize the window to a phone screen', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 800 })
})

When('I toggle the part stream diff layout', async ({ page }) => {
  await page.getByTestId('part-stream-layout-toggle').click()
})

Then(
  'the part stream diff is shown {}',
  async ({ page }, layout: 'side by side' | 'inline') => {
    const diffEditor = page.locator(
      '[data-testid="part-stream-editor"] .monaco-diff-editor',
    )
    // Monaco sets `side-by-side` on the diff editor root only in that layout.
    if (layout === 'side by side') {
      await expect(diffEditor).toHaveClass(/\bside-by-side\b/)
    } else {
      await expect(diffEditor).not.toHaveClass(/\bside-by-side\b/)
    }
  },
)

Then('the part stream editor is closed', async ({ page }) => {
  await expect(page.getByTestId('part-stream-modal')).toBeHidden()
})

Then(
  'the main editor source is the {} source',
  async ({ page }, name: string) => {
    await expect
      .poll(() => editorValue(page, 'main'))
      .toBe(EXPECTED_SOURCES[name])
  },
)

When('I press undo in the main editor', async ({ page, focusEditor }) => {
  await focusEditor()
  // Monaco's undo keybinding is the `Control` chord on every host OS.
  await page.keyboard.press('Control+z')
})

Then(
  'the part stream picker row for part {string} is disabled',
  async ({ page }, abbreviation: string) => {
    await expect(partRow(page, abbreviation)).toBeDisabled()
  },
)

Then(
  'the part stream picker row for part {string} is enabled',
  async ({ page }, abbreviation: string) => {
    await expect(partRow(page, abbreviation)).toBeEnabled()
  },
)

Then(
  'the part stream picker row for part {string} explains {string}',
  async ({ page }, abbreviation: string, reason: string) => {
    await expect(partRow(page, abbreviation)).toContainText(reason)
  },
)

function partRow(page: Page, abbreviation: string) {
  return page.locator(
    `[data-testid="part-stream-part-row"][data-abbreviation="${abbreviation}"]`,
  )
}
