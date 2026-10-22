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
    await page.getByTestId(`notation-toggle-${notation}`).click()
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
