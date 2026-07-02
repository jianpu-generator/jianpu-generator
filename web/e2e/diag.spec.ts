import { test } from '@playwright/test'
import { encodeShareHashSuffix } from '../src/shareUrl'

const SHARED_FILENAME = 'shared-test.jianpu'
const SHARED_SOURCE = [
  '# metadata',
  'title = "Shared Score"',
  '',
  '# parts',
  'Melody = notes',
  '',
  '# score',
  '(time=4/4 key=C4 bpm=120)',
  '1 2 3 4',
].join('\n')

test.beforeEach(async ({ context }) => {
  await context.addInitScript(() => {
    console.log(
      '[initScript] RUNNING - localStorage keys:',
      Object.keys(localStorage),
    )
    localStorage.clear()
    console.log('[initScript] AFTER CLEAR - keys:', Object.keys(localStorage))
  })
})

test('what does localStorage contain when share URL loads?', async ({
  page,
}) => {
  page.on('console', (msg) => {
    if (
      msg.text().includes('[initScript]') ||
      msg.text().includes('files:v1')
    ) {
      console.log('BROWSER LOG:', msg.text())
    }
  })
  const shareUrl = `http://localhost:5173/#share=${encodeShareHashSuffix(SHARED_FILENAME, SHARED_SOURCE)}`
  await page.goto(shareUrl)
  const state = await page.evaluate(() =>
    localStorage.getItem('jianpu:files:v1'),
  )
  console.log('After page load:', state ? JSON.parse(state).active : 'EMPTY')
  console.log(
    'All keys after load:',
    await page.evaluate(() => Object.keys(localStorage)),
  )
})
