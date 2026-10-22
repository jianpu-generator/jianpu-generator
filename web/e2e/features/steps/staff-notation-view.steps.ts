import { expect, type Page } from '@playwright/test'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  '# metadata',
  'title = "Staff View Test"',
  '',
  '# parts',
  'Soprano [S] = notes',
  'Alto [A] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '[S] 1 2 3 4',
  '[A] 5, 6, 7, 1',
].join('\n')

const SOURCE_NOTE_COUNT = 8

/** Verovio's first render also downloads its ~7 MB module. */
const STAFF_RENDER_TIMEOUT = 30_000

function staffPage(page: Page) {
  return page.getByTestId('staff-page')
}

Given(
  'a two-part score is loaded for the staff notation view',
  async ({ page }) => {
    await page.addInitScript((src) => {
      localStorage.setItem(
        'jianpu:files:v1',
        JSON.stringify({
          active: 'staff.jianpu',
          userFiles: { 'staff.jianpu': src },
          bin: {},
          fileIds: { 'staff.jianpu': crypto.randomUUID() },
        }),
      )
    }, SOURCE)
    await page.goto('/')
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

When(
  'I switch the preview notation to {string}',
  async ({ page }, notation: string) => {
    await page.getByTestId('notation-select').selectOption(notation)
  },
)

Then('the preview shows jianpu pages', async ({ page }) => {
  await expect(page.locator('.preview-page').first()).toBeVisible()
  await expect(staffPage(page)).toHaveCount(0)
})

Then('the preview shows a staff notation page', async ({ page }) => {
  await expect(staffPage(page).first().locator('svg').first()).toBeVisible({
    timeout: STAFF_RENDER_TIMEOUT,
  })
})

Then(
  'the staff notation page has a note for every source note',
  async ({ page }) => {
    await expect(staffPage(page).locator('g.note')).toHaveCount(
      SOURCE_NOTE_COUNT,
      { timeout: STAFF_RENDER_TIMEOUT },
    )
  },
)

Then(
  'the staff notation page has {int} notes',
  async ({ page }, count: number) => {
    await expect(staffPage(page).locator('g.note')).toHaveCount(count, {
      timeout: STAFF_RENDER_TIMEOUT,
    })
  },
)

When(
  'I hide the {string} part from the staff view',
  async ({ page }, abbreviation: string) => {
    const visible = page.getByRole('checkbox', {
      name: abbreviation,
      exact: true,
    })
    const eye = page
      .locator('.part-toggle-pill')
      .filter({
        has: page.locator('.part-toggle-abbr', {
          hasText: new RegExp(`^${abbreviation}$`),
        }),
      })
      .locator('.part-toggle-segment--eye')
    // The part list can still be settling right after load, which drops an
    // early click, so retry until the toggle actually turns off.
    await expect(async () => {
      if (await visible.isChecked()) await eye.click()
      await expect(visible).not.toBeChecked({ timeout: 1_000 })
    }).toPass({ timeout: 15_000 })
  },
)

const exportedFiles = new Map<string, { name: string; bytes: Buffer }>()

When(
  'I export {string} from the preview, keeping it as the {word} PDF',
  async ({ page }, itemName: string, notation: string) => {
    const menuButton = page.getByRole('button', { name: 'Export', exact: true })
    await expect(menuButton).toBeEnabled({ timeout: STAFF_RENDER_TIMEOUT })
    await menuButton.click()
    const item = page.getByRole('menuitem', { name: itemName, exact: true })
    await expect(item).toBeEnabled({ timeout: STAFF_RENDER_TIMEOUT })
    await item.click()
    const confirm = page.getByTestId('download-rename-confirm')
    await expect(confirm).toBeVisible({ timeout: STAFF_RENDER_TIMEOUT })
    const [download] = await Promise.all([
      page.waitForEvent('download', { timeout: STAFF_RENDER_TIMEOUT }),
      confirm.click(),
    ])
    const fs = await import('node:fs')
    exportedFiles.set(notation, {
      name: download.suggestedFilename(),
      bytes: fs.readFileSync((await download.path()) as string),
    })
  },
)

Then('the staff PDF is named {string}', async ({}, name: string) => {
  expect(exportedFiles.get('staff')?.name).toBe(name)
})

Then(
  'the {word} PDF differs from the jianpu PDF',
  async ({}, notation: string) => {
    const exported = exportedFiles.get(notation)?.bytes
    const jianpu = exportedFiles.get('jianpu')?.bytes
    expect(exported?.subarray(0, 4).toString()).toBe('%PDF')
    expect(jianpu?.subarray(0, 4).toString()).toBe('%PDF')
    expect(exported?.equals(jianpu as Buffer)).toBe(false)
  },
)

Then(
  'the staff ZIP holds {string} and {string}',
  async ({}, first: string, second: string) => {
    // A ZIP's central directory stores each entry's name uncompressed.
    const zip = exportedFiles.get('staff')?.bytes.toString('latin1') ?? ''
    expect(zip).toContain(first)
    expect(zip).toContain(second)
  },
)
