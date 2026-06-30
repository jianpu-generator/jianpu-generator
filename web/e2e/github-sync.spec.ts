import { expect, test } from '@playwright/test'

const FILE_STORE_KEY = 'jianpu:files:v1'

const LOCAL_SOURCE = [
  '# metadata',
  'title = "Local Workspace Score"',
  '',
  '# parts',
  'Melody = notes',
  '',
  '# score',
  '(time=4/4 key=C4 bpm=120)',
  '1 2 3 4',
].join('\n')

const GITHUB_SOURCE = [
  '# metadata',
  'title = "GitHub Workspace Score"',
  '',
  '# parts',
  'Melody = notes',
  '',
  '# score',
  '(time=4/4 key=C4 bpm=120)',
  '5 6 7 8',
].join('\n')

function githubStorePayload() {
  return {
    manifest: {
      active: 'github-only.jianpu',
      fileIds: { 'github-only.jianpu': 'github-file-id' },
      bin: [],
    },
    scoreFiles: {
      'github-only.jianpu': GITHUB_SOURCE,
    },
    binFiles: {},
  }
}

async function mockDisconnectedSession(page: import('@playwright/test').Page) {
  await page.route('**/api/github/session', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ connected: false }),
    })
  })
}

async function mockConnectedGitHub(
  page: import('@playwright/test').Page,
  options: { trackPuts?: boolean } = {},
) {
  const puts: Array<{ path: string; content: string }> = []

  await page.route('**/api/github/session', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        connected: true,
        username: 'test-user',
        repo: 'jianpu-scores',
      }),
    })
  })

  await page.route('**/api/github/store', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(githubStorePayload()),
    })
  })

  await page.route('**/api/github/manifest', async (route) => {
    if (route.request().method() === 'PATCH') {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ sha: 'manifest-sha' }),
      })
      return
    }
    await route.continue()
  })

  await page.route('**/api/github/files/**', async (route) => {
    if (route.request().method() === 'PUT') {
      const url = new URL(route.request().url())
      const path = decodeURIComponent(url.pathname.replace(/^.*\/files\//, ''))
      const body = route.request().postDataJSON() as { content?: string }
      if (options.trackPuts && typeof body.content === 'string') {
        puts.push({ path, content: body.content })
      }
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ sha: 'file-sha' }),
      })
      return
    }
    await route.continue()
  })

  return {
    puts,
  }
}

test('shows connect prompt when GitHub workspace is selected without a session', async ({
  page,
}) => {
  await mockDisconnectedSession(page)
  await page.goto('/')

  await page.getByRole('tab', { name: 'GitHub' }).click()

  await expect(page.getByRole('region', { name: 'GitHub sync' })).toBeVisible()
  await expect(
    page.getByRole('link', { name: 'Connect with GitHub' }),
  ).toHaveAttribute('href', '/api/github/login')
  await expect(page.locator('.file-tab')).toHaveCount(0)
})

test('loads GitHub scores when connected', async ({ page }) => {
  await mockConnectedGitHub(page)
  await page.goto('/')

  await page.getByRole('tab', { name: 'GitHub' }).click()

  await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
    'github-only.jianpu',
    { timeout: 10_000 },
  )
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
  const previewContent = await page.locator('.preview-page').first().innerHTML()
  expect(previewContent).toContain('GitHub Workspace Score')
})

test('debounces autosave and PUTs changed score files', async ({ page }) => {
  const { puts } = await mockConnectedGitHub(page, { trackPuts: true })
  await page.goto('/')

  await page.getByRole('tab', { name: 'GitHub' }).click()
  await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
    'github-only.jianpu',
    { timeout: 10_000 },
  )

  await page.locator('.monaco-editor').click()
  await page.keyboard.press('End')
  await page.keyboard.type('\n# autosave marker')

  await expect(page.getByTestId('sync-status')).toHaveAttribute(
    'data-sync-status',
    'saved',
    { timeout: 10_000 },
  )

  expect(puts.length).toBeGreaterThan(0)
  expect(puts.some((put) => put.path.includes('github-only.jianpu'))).toBe(true)
  expect(puts.some((put) => put.content.includes('# autosave marker'))).toBe(
    true,
  )
})

test('keeps local workspace tabs isolated from GitHub workspace', async ({
  page,
}) => {
  await mockConnectedGitHub(page)
  await page.addInitScript(
    ({ key, source }: { key: string; source: string }) => {
      localStorage.setItem(
        key,
        JSON.stringify({
          active: 'local-only.jianpu',
          userFiles: { 'local-only.jianpu': source },
          bin: {},
          fileIds: { 'local-only.jianpu': 'local-file-id' },
        }),
      )
    },
    { key: FILE_STORE_KEY, source: LOCAL_SOURCE },
  )

  await page.goto('/')

  await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
    'local-only.jianpu',
  )

  await page.getByRole('tab', { name: 'GitHub' }).click()
  await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
    'github-only.jianpu',
    { timeout: 10_000 },
  )

  await page.getByRole('tab', { name: 'Local' }).click()
  await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
    'local-only.jianpu',
  )

  await page.waitForSelector('.preview-page', { timeout: 15_000 })
  const previewContent = await page.locator('.preview-page').first().innerHTML()
  expect(previewContent).toContain('Local Workspace Score')
})
