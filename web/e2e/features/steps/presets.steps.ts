import { expect, type Page } from '@playwright/test'
import { Given, Then, When } from './fixtures'

// "the preview contains the chord content" and its negation come from
// `part-toggles.steps.ts`; "I press undo in the main editor" from
// `part-stream-editor.steps.ts`.

const PRESETS_SOURCE = [
  '# metadata',
  'title = "Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  'Melody lyrics [Mv1] = lyrics[M]',
  'Chords [C] = chords',
  '',
  '# preset "Melody only"',
  'visible_parts = M, Mv1',
  '',
  '# preset "Violin"',
  'notation = violin',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '[M] 1 2 3 4',
  '[Mv1] twin- kle twin- kle',
  '[C] 1 - - -',
].join('\n')

type MonacoWindow = Window & {
  monaco?: {
    editor: {
      getEditors: () => Array<{
        getModel: () => {
          getValue: () => string
          findMatches: (
            search: string,
            searchOnlyEditableRange: boolean,
            isRegex: boolean,
            matchCase: boolean,
            wordSeparators: null,
            captureMatches: boolean,
          ) => Array<{ range: unknown }>
        } | null
        getPosition: () => { lineNumber: number } | null
        executeEdits: (
          source: string,
          edits: Array<{ range: unknown; text: string }>,
        ) => void
      }>
    }
  }
}

async function editorSource(page: Page) {
  return page.evaluate(
    () =>
      (window as MonacoWindow).monaco?.editor
        .getEditors()[0]
        ?.getModel()
        ?.getValue() ?? '',
  )
}

async function storedSource(page: Page) {
  return page.evaluate(() => {
    const raw = localStorage.getItem('jianpu:files:v1')
    if (!raw) return ''
    const store = JSON.parse(raw) as {
      active: string
      userFiles: Record<string, string>
    }
    return store.userFiles[store.active] ?? ''
  })
}

/** The trimmed lines of the `# preset "name"` section, header excluded. */
function presetLines(source: string, name: string): string[] {
  const lines = source.split('\n')
  const header = lines.indexOf(`# preset "${name}"`)
  if (header === -1) return []
  const rest = lines.slice(header + 1)
  const end = rest.findIndex((line) => line.startsWith('#'))
  return rest.slice(0, end === -1 ? undefined : end).map((line) => line.trim())
}

function presetMenu(page: Page) {
  return page.getByTestId('preset-menu')
}

function partPillEye(page: Page, abbreviation: string) {
  return page.locator('.part-toggle-segment--eye', {
    hasText: new RegExp(`^${abbreviation}$`),
  })
}

function partPillCheckbox(page: Page, abbreviation: string) {
  return partPillEye(page, abbreviation).locator('input[type="checkbox"]')
}

function newPresetDialog(page: Page) {
  return page.getByRole('dialog', { name: 'New preset' })
}

async function waitForPreviewReady(page: Page) {
  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await expect(page.locator('.preview-pages')).toContainText('1234', {
    timeout: 15_000,
  })
}

async function choosePresetMenuItem(page: Page, label: string) {
  await presetMenu(page).click()
  await page.getByRole('menuitem', { name: label, exact: true }).click()
}

Given('the presets fixture is loaded', async ({ page }) => {
  await page.addInitScript((source) => {
    // Seed once: a reload must keep the same file id so its chosen preset
    // survives.
    if (localStorage.getItem('jianpu:files:v1')) return
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'test.jianpu',
        userFiles: { 'test.jianpu': source },
        bin: {},
        fileIds: { 'test.jianpu': crypto.randomUUID() },
      }),
    )
  }, PRESETS_SOURCE)
  await page.goto('/')
  await waitForPreviewReady(page)
})

When('I choose the {string} preset', async ({ page }, name: string) => {
  await choosePresetMenuItem(page, name)
})

When(
  'I choose {string} in the preset menu',
  async ({ page }, label: string) => {
    await choosePresetMenuItem(page, label)
  },
)

When(
  'I turn the {string} part pill off',
  async ({ page }, abbreviation: string) => {
    // The checkbox is visually hidden; click its segment like a user would.
    await expect(partPillCheckbox(page, abbreviation)).toBeChecked()
    await partPillEye(page, abbreviation).click()
  },
)

When('I reload the presets page', async ({ page }) => {
  await page.reload()
  await waitForPreviewReady(page)
})

When(
  'I replace {string} with {string} in the editor',
  async ({ page }, from: string, to: string) => {
    await page.evaluate(
      ({ from, to }) => {
        const editor = (window as MonacoWindow).monaco?.editor.getEditors()[0]
        const model = editor?.getModel()
        if (!editor || !model) throw new Error('No editor')
        const edits = model
          .findMatches(from, false, false, true, null, false)
          .map(({ range }) => ({ range, text: to }))
        editor.executeEdits('e2e', edits)
      },
      { from, to },
    )
  },
)

When('I open the new preset dialog', async ({ page }) => {
  await choosePresetMenuItem(page, 'New preset…')
  await expect(newPresetDialog(page)).toBeVisible()
})

When('I type {string} as the preset name', async ({ page }, name: string) => {
  await newPresetDialog(page).getByLabel('Preset name').fill(name)
})

When(
  'I create a new preset called {string}',
  async ({ page }, name: string) => {
    await choosePresetMenuItem(page, 'New preset…')
    const dialog = newPresetDialog(page)
    await dialog.getByLabel('Preset name').fill(name)
    await dialog.getByRole('button', { name: 'Create' }).click()
    await expect(dialog).toBeHidden()
  },
)

Then('the preset menu shows {string}', async ({ page }, name: string) => {
  await expect(presetMenu(page)).toHaveText(name)
})

Then(
  'the {string} part pill is off',
  async ({ page }, abbreviation: string) => {
    await expect(partPillCheckbox(page, abbreviation)).not.toBeChecked()
  },
)

Then('the {string} part pill is on', async ({ page }, abbreviation: string) => {
  await expect(partPillCheckbox(page, abbreviation)).toBeChecked()
})

Then(
  'the {string} preset has the line {string}',
  async ({ page }, name: string, line: string) => {
    await expect
      .poll(async () => presetLines(await editorSource(page), name))
      .toContain(line)
  },
)

Then(
  "the stored file's {string} preset has the line {string}",
  async ({ page }, name: string, line: string) => {
    await expect
      .poll(async () => presetLines(await storedSource(page), name))
      .toContain(line)
  },
)

Then('the stored source is the presets fixture', async ({ page }) => {
  // Give a would-be autosave time to land before checking it didn't.
  await page.waitForTimeout(1_000)
  expect(await storedSource(page)).toBe(PRESETS_SOURCE)
})

Then(
  'the new preset dialog says {string}',
  async ({ page }, message: string) => {
    await expect(newPresetDialog(page).getByRole('alert')).toHaveText(message)
  },
)

Then("the new preset dialog's Create button is disabled", async ({ page }) => {
  await expect(
    newPresetDialog(page).getByRole('button', { name: 'Create' }),
  ).toBeDisabled()
})

Then(
  "the editor cursor is on the {string} preset's header line",
  async ({ page }, name: string) => {
    const headerLine =
      PRESETS_SOURCE.split('\n').indexOf(`# preset "${name}"`) + 1
    await expect
      .poll(() =>
        page.evaluate(
          () =>
            (window as MonacoWindow).monaco?.editor
              .getEditors()[0]
              ?.getPosition()?.lineNumber,
        ),
      )
      .toBe(headerLine)
  },
)
