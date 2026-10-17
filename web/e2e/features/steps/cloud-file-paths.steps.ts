import { openFileList } from '../../fileSwitcherHelpers'
import { When } from './fixtures'

/** Selects a file of the cloud-backed list by its path (extension left off),
 * expanding each folder on the way, then leaves the file list open. */
When(
  'I open the cloud file {string} from the file list',
  async ({ page }, path: string) => {
    await openFileList(page)
    const segments = path.split('/')
    const folders = segments.slice(0, -1)
    for (const folder of folders) {
      const row = page.getByRole('button', { name: folder, exact: true })
      if ((await row.getAttribute('aria-expanded')) === 'false') {
        await row.click()
      }
    }
    const fileName = segments.at(-1) ?? path
    await page
      .locator('.file-tab-name', { hasText: new RegExp(`^${fileName}$`) })
      .click()
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await openFileList(page)
  },
)
