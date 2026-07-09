import type { Page } from '@playwright/test'

declare global {
  interface Window {
    monaco?: typeof import('monaco-editor')
  }
}

function activeEditorValue(): string | null {
  const model = window.monaco?.editor.getEditors()[0]?.getModel()
  return model?.getValue() ?? null
}

function editorHasFocus() {
  const active = document.activeElement
  return (
    !!active &&
    (active.classList.contains('native-edit-context') ||
      active.tagName === 'TEXTAREA')
  )
}

/** Clicks the Monaco editor's view-lines and waits for it to actually gain
 * focus before returning. A plain `page.click(...)` can resolve before
 * Monaco has finished wiring up its focus target — a race that's normally
 * lost within milliseconds, but occasionally not at all — which then races
 * immediately-following keyboard shortcuts. Retries the click, and falls
 * back to focusing the editor's input node directly, until focus lands. */
export async function focusEditor(page: Page) {
  for (let attempt = 0; attempt < 10; attempt++) {
    await page.click('.monaco-editor .view-lines')
    if (await page.evaluate(editorHasFocus)) return
    await page.waitForTimeout(150)
    if (await page.evaluate(editorHasFocus)) return
    await page.evaluate(() => {
      const node = document.querySelector<HTMLElement>(
        '.monaco-editor .native-edit-context, .monaco-editor textarea',
      )
      node?.focus()
    })
    if (await page.evaluate(editorHasFocus)) return
  }
  throw new Error('focusEditor: editor never gained focus')
}

/** Focuses the editor, moves the cursor to the end of the document, and
 * types `text` there — the pattern nearly every editing test uses. Beyond
 * `focusEditor`'s DOM-focus race, there is a second, narrower race: even
 * once the editor's input node has DOM focus, Monaco can take a further
 * moment to wire up its keybinding/input handling, so a `Control+End` or
 * `type()` fired immediately after focus can be dropped or partially land.
 * This waits for the cursor to actually reach the end (via the real Monaco
 * instance exposed on `window.monaco`, set up in `monacoSetup.ts`) before
 * typing, then verifies the typed text landed by reading the model value
 * back. If it didn't, it restores the model to its pre-attempt value and
 * retries the whole focus → move → type sequence. */
export async function typeAtEditorEnd(page: Page, text: string) {
  for (let attempt = 0; attempt < 5; attempt++) {
    await focusEditor(page)

    const before = await page.evaluate(activeEditorValue)
    if (before === null) {
      throw new Error('typeAtEditorEnd: no active Monaco editor model')
    }

    await page.keyboard.press('Control+End')
    await page.waitForFunction(() => {
      const ed = window.monaco?.editor.getEditors()[0]
      const model = ed?.getModel()
      const pos = ed?.getPosition()
      if (!model || !pos) return false
      const end = model.getFullModelRange().getEndPosition()
      return pos.lineNumber === end.lineNumber && pos.column === end.column
    })

    await page.keyboard.type(text)

    const expected = before + text
    try {
      await page.waitForFunction(
        (expected) =>
          window.monaco?.editor.getEditors()[0]?.getModel()?.getValue() ===
          expected,
        expected,
        { timeout: 2_000 },
      )
      return
    } catch {
      // The typed text was dropped or only partially landed. Reset the
      // model back to its known-good pre-attempt value (still routed
      // through the app's normal onChange, since it's a real model edit)
      // and retry the whole sequence from a clean state.
      await page.evaluate((before) => {
        window.monaco?.editor.getEditors()[0]?.getModel()?.setValue(before)
      }, before)
    }
  }
  throw new Error(`typeAtEditorEnd: "${text}" never landed after retries`)
}

/** The "<filename> ▾" trigger button in the header — its text is the
 * currently active file's name. */
export function fileSwitcherTrigger(page: Page) {
  return page
    .locator('.file-tab-bar .export-menu')
    .first()
    .locator('.preview-export-btn')
}

/** Opens the file-switcher dropdown (the "<filename> ▾" trigger in the
 * header) so its file list / Bin become visible and interactable. Idempotent
 * — the trigger toggles open/closed, so this is a no-op if already open. */
export async function openFileList(page: Page) {
  const trigger = fileSwitcherTrigger(page)
  if ((await trigger.getAttribute('aria-expanded')) === 'true') return
  await trigger.click()
}

/** Opens the "⋯" file-actions dropdown (New / Duplicate / Share / Storage…)
 * in the header. Idempotent — the trigger toggles open/closed, so this is a
 * no-op if already open. */
export async function openFileActions(page: Page) {
  const trigger = page.getByRole('button', { name: 'File actions' })
  if ((await trigger.getAttribute('aria-expanded')) === 'true') return
  await trigger.click()
}

/** Opens the "Bin (N)" dropdown in the header. Idempotent — the trigger
 * toggles open/closed, so this is a no-op if already open. */
export async function openBin(page: Page) {
  const trigger = page.locator('.file-tab-bar-bin-trigger')
  if ((await trigger.getAttribute('aria-expanded')) === 'true') return
  await trigger.click()
}
