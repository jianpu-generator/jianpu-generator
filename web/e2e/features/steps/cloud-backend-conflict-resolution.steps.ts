import { expect, type Page } from '@playwright/test'
import {
  CLOUD_WORKER_ORIGIN,
  seedCloudFile,
  updateCloudFileContent,
} from '../../cloudFileHelpers'
import {
  fileSwitcherTrigger,
  fileTabByExactName,
  openFileActions,
  openFileList,
  typeAtEditorEnd,
} from '../../fileSwitcherHelpers'
import { currentSignedInLogin } from './cloud-account.steps'
import { gotoCloudApp } from './cloud-account-helpers'
import {
  type ContentSaveTracker,
  installContentSaveTracking,
} from './cloud-content-save-tracking'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  '# metadata',
  'title = "Conflict Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

const REMOTE_SOURCE = [
  '# metadata',
  'title = "Conflict Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '5 6 7 1',
].join('\n')

let conflictTracker: ContentSaveTracker
let conflictFileId = ''
let conflictFileRevision = 0

/**
 * Mirrors the deleted GitHub backend's `setUpConflictingEdit`: seeds the
 * file, types an edit, force-saves it, and injects a one-shot fake `409`
 * (revision-conflict) response on that very first content-save request,
 * reporting the seeded row's actual current revision (not a hardcoded `0`
 * -- see the fulfill call below) so `forceOverwrite`'s realign-then-retry
 * lands for real against the real worker on the next attempt. Every
 * request after the first one-shot 409 passes through untouched
 * (`route.continue()`).
 */
async function setUpConflictingEdit(
  page: Page,
  filename: string,
): Promise<void> {
  const login = currentSignedInLogin()
  const seeded = await seedCloudFile(login, filename, SOURCE)
  conflictFileId = seeded.id
  conflictFileRevision = seeded.revision
  conflictTracker = installContentSaveTracking(page)

  let contentSaveCount = 0
  await page.route(`${CLOUD_WORKER_ORIGIN}/files/*/content`, async (route) => {
    contentSaveCount += 1
    if (contentSaveCount === 1) {
      // `seeded.revision` (not a hardcoded `0`): `seedCloudFile` is
      // idempotent across reruns of this same scenario (see its own doc
      // comment), reusing -- rather than recreating -- a pre-existing row
      // left by an earlier pass, which can already sit at a nonzero
      // revision. The one-shot fake 409 must report whatever the real row's
      // revision genuinely is right now, so `forceOverwrite`'s
      // realign-then-retry lands for real against the real worker on the
      // very next attempt instead of colliding with a *second*, real
      // conflict.
      await route.fulfill({
        status: 409,
        contentType: 'application/json',
        body: JSON.stringify({ currentRevision: seeded.revision }),
      })
      return
    }
    await route.continue()
  })

  await gotoCloudApp(page)

  const displayName = filename.replace(/\.jianpu$/, '')
  await openFileList(page)
  const tab = fileTabByExactName(page, displayName)
  await tab.waitFor({ timeout: 15_000 })
  await tab.click()
  await expect(fileSwitcherTrigger(page)).toContainText(displayName)
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  await typeAtEditorEnd(page, ' 5')

  // Force-save immediately rather than waiting out the debounce; the next
  // content-save request this triggers is the one the one-shot 409 above
  // targets.
  await page.keyboard.press('Meta+s')

  // Opening the modal after the failed save (rather than before) avoids its
  // overlay intercepting the editor click above.
  await openFileActions(page)
  await page.getByRole('menuitem', { name: 'Storage…' }).click()
  await page.getByTestId('storage-settings-modal').waitFor()

  await expect(page.getByTestId('conflict-banner')).toBeVisible({
    timeout: 10_000,
  })

  // The failed force-save left the "Saved" badge showing the conflict's
  // error status -- resolving it below should update the badge, not leave
  // it stuck.
  await expect(page.getByTestId('save-status-badge')).toHaveText('Save failed')
}

Given(
  'a cloud save conflict is set up on {string} for overwrite-mine',
  async ({ page }, filename: string) => {
    await setUpConflictingEdit(page, filename)
  },
)

Given(
  'a cloud save conflict is set up on {string} for discard-mine',
  async ({ page }, filename: string) => {
    await setUpConflictingEdit(page, filename)
  },
)

Given(
  'the remote file has since changed to the conflicting content',
  async () => {
    // Simulates the change that raced the user's save actually landing on
    // the worker, so "discard mine" has different remote content to pull
    // in. The seeded row's real revision is still whatever
    // `setUpConflictingEdit` observed it at (the earlier "conflict" was a
    // fake, one-shot 409 that never reached the real worker, so nothing
    // since then has really written to it), so this update is expected to
    // (and does) succeed for real.
    await updateCloudFileContent(
      currentSignedInLogin(),
      conflictFileId,
      REMOTE_SOURCE,
      conflictFileRevision,
    )
  },
)

When(
  'I click the conflict-resolution button {string}',
  async ({ page }, buttonName: string) => {
    await page.getByRole('button', { name: buttonName }).click()
  },
)

Then('the conflict banner is gone', async ({ page }) => {
  await expect(page.getByTestId('conflict-banner')).toHaveCount(0)
})

Then(
  'the last content save for the conflict contains {string}',
  async ({}, text: string) => {
    await expect
      .poll(() => conflictTracker.records.at(-1)?.content, {
        timeout: 10_000,
      })
      .toEqual(expect.stringContaining(text))
  },
)

Then(
  'the conflict status badge shows exactly {string}',
  async ({ page }, text: string) => {
    await expect(page.getByTestId('save-status-badge')).toHaveText(text)
  },
)

Then('the editor still contains {string}', async ({ page }, text: string) => {
  // The editor still shows the user's edit -- overwrite-mine must not have
  // discarded it.
  await expect(page.locator('.monaco-editor .view-lines')).toContainText(text)
})

Then(
  'the editor now shows the remote content {string}',
  async ({ page }, text: string) => {
    await expect(page.locator('.monaco-editor .view-lines')).toContainText(text)
  },
)

Then(
  'the editor no longer contains {string}',
  async ({ page }, text: string) => {
    await expect(page.locator('.monaco-editor .view-lines')).not.toContainText(
      text,
    )
  },
)
