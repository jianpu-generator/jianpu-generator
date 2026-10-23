import type { Page } from '@playwright/test'
import { expect } from '@playwright/test'

/**
 * Waits for the asset loading banner (soundfont, fonts, wasm) at the top of
 * the app to go away. The banner sits above the workspace, so when it hides
 * everything below shifts up. A drag across a jump toolbar that's still in
 * flight at that moment has its toolbar slide out from under the cursor: the
 * toolbar's `mouseleave` cancels the drag, and only the first entry ends up
 * selected. Under parallel load the soundfont can finish at any point, so
 * wait for it before dragging.
 */
export async function waitForAssetLoadingBannerGone(page: Page) {
  await expect(page.getByTestId('asset-loading-banner')).toHaveCount(0, {
    timeout: 30_000,
  })
}
