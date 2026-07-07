import { expect, test } from '@playwright/test'

const SOURCE = [
  '# metadata',
  'title = "Autosave Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

async function getStoredSource(page: import('@playwright/test').Page) {
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

test('editing a file persists to the local storage backend without waiting out the autosave debounce', async ({
  page,
}) => {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'auto.jianpu',
        userFiles: { 'auto.jianpu': src },
        bin: {},
        fileIds: { 'auto.jianpu': crypto.randomUUID() },
      }),
    )
  }, SOURCE)

  // Install the fake clock (and never advance it). `localBackend`'s
  // `saveContent` is a no-op — `useLocalStorage` already writes synchronously
  // on every keystroke — so proving the edit lands in `localStorage` without
  // ever fast-forwarding the clock demonstrates local persistence has no
  // dependency on the autosave debounce timer at all.
  await page.clock.install()

  await page.goto('/')

  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  await page.click('.monaco-editor .view-lines')
  await page.keyboard.press('Control+End')
  await page.keyboard.type(' 5')

  await expect
    .poll(getStoredSource.bind(null, page))
    .toContain('1 2 3 4 5')

  // No debounced save is even scheduled for the local backend, so the
  // save-status badge (only meaningful for GitHub) never appears.
  await expect(page.getByTestId('save-status-badge')).toHaveCount(0)
})
