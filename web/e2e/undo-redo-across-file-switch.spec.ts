import { expect, test } from '@playwright/test'
import {
  focusEditor,
  openFileList,
  typeAtEditorEnd,
} from './fileSwitcherHelpers'

const SOURCE_A = [
  '# metadata',
  'title = "File A"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

const EDITED_A = SOURCE_A.replace('1 2 3 4', '1 2 3 4 5')

const SOURCE_B = [
  '# metadata',
  'title = "File B"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '5 6 7 1',
].join('\n')

async function getStoredFile(
  page: import('@playwright/test').Page,
  name: string,
) {
  return page.evaluate((fileName) => {
    const raw = localStorage.getItem('jianpu:files:v1')
    if (!raw) return ''
    const store = JSON.parse(raw) as {
      active: string
      userFiles: Record<string, string>
    }
    return store.userFiles[fileName] ?? ''
  }, name)
}

// `.monaco-editor .view-lines` can transiently render two overlapping
// snapshots of the content while the virtualized scroller animates, so
// asserting on its text right after an undo/redo keystroke is flaky. Read the
// live model value of the active editor instance instead, via the `monaco`
// global that `@monaco-editor/react`'s loader exposes on `window` (each open
// file has its own model, keyed by `path`, so this must read the model
// attached to the visible editor rather than an arbitrary one).
async function getEditorValue(page: import('@playwright/test').Page) {
  return page.evaluate(() => {
    const monacoApi = (
      window as unknown as { monaco: typeof import('monaco-editor') }
    ).monaco
    const model = monacoApi.editor.getEditors()[0]?.getModel()
    return model?.getValue() ?? ''
  })
}

// Monaco groups keystrokes into undo stops at word boundaries, so whether
// typing " 5" lands as one or two undo stops is a timing detail, not
// something worth pinning down here. Press the given key until the model
// reaches `expected` (or give up after a few tries, letting the final
// assertion report the mismatch).
async function pressUntilValue(
  page: import('@playwright/test').Page,
  key: string,
  expected: string,
) {
  for (let i = 0; i < 5; i++) {
    if ((await getEditorValue(page)) === expected) return
    await page.keyboard.press(key)
  }
}

test("an unsaved edit in file A survives switching to file B and back, and undo/redo only ever touch A's own edit", async ({
  page,
}) => {
  await page.addInitScript(
    ({ sourceA, sourceB }) => {
      localStorage.setItem(
        'jianpu:files:v1',
        JSON.stringify({
          active: 'a.jianpu',
          userFiles: { 'a.jianpu': sourceA, 'b.jianpu': sourceB },
          bin: {},
          fileIds: {
            'a.jianpu': crypto.randomUUID(),
            'b.jianpu': crypto.randomUUID(),
          },
        }),
      )
    },
    { sourceA: SOURCE_A, sourceB: SOURCE_B },
  )

  await page.goto('/')

  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  // Edit file A.
  await typeAtEditorEnd(page, ' 5')

  await expect
    .poll(getStoredFile.bind(null, page, 'a.jianpu'))
    .toContain('1 2 3 4 5')

  // Switch to file B without explicitly saving.
  await openFileList(page)
  await page.locator('.file-tab-name', { hasText: 'b.jianpu' }).click()
  await openFileList(page)
  await expect(
    page.locator('.file-tab-name', { hasText: 'b.jianpu' }),
  ).toHaveAttribute('aria-current', 'true')
  await expect(page.locator('.monaco-editor .view-lines')).toContainText(
    '5 6 7 1',
  )

  // Switch back to file A: the edit must still be there.
  await page.locator('.file-tab-name', { hasText: 'a.jianpu' }).click()
  await openFileList(page)
  await expect(
    page.locator('.file-tab-name', { hasText: 'a.jianpu' }),
  ).toHaveAttribute('aria-current', 'true')
  await expect(page.locator('.monaco-editor .view-lines')).toContainText(
    '1 2 3 4 5',
  )

  // Undo, after the round trip through B, must undo A's own edit -
  // not some artifact of the tab switch (e.g. reverting to B's content).
  // Monaco's built-in undo/redo keybindings resolve to the `Control`
  // chord regardless of host OS (unlike the app's own Cmd/Ctrl+S handler),
  // so these are not platform-conditional like other specs' `Meta+...`.
  await focusEditor(page)
  const undoKey = 'Control+z'
  const redoKey = 'Control+y'

  await pressUntilValue(page, undoKey, SOURCE_A)
  await expect.poll(getEditorValue.bind(null, page)).toBe(SOURCE_A)
  await expect.poll(getStoredFile.bind(null, page, 'a.jianpu')).toBe(SOURCE_A)
  expect(await getStoredFile(page, 'b.jianpu')).toBe(SOURCE_B)

  // Redo must restore A's edit, again without touching B.
  await pressUntilValue(page, redoKey, EDITED_A)
  await expect.poll(getEditorValue.bind(null, page)).toBe(EDITED_A)
  await expect.poll(getStoredFile.bind(null, page, 'a.jianpu')).toBe(EDITED_A)
  expect(await getStoredFile(page, 'b.jianpu')).toBe(SOURCE_B)
})
