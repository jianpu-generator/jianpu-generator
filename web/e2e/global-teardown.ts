import { readdir, rm } from 'node:fs/promises'
import { basename, dirname, join } from 'node:path'
import { CHROMIUM_CACHE_DIR_PREFIX } from './chromiumCachePrefix'

// Each worker's Chromium instance gets its own `--disk-cache-dir` (see the
// comment in playwright.config.ts), but nothing ever cleaned those dirs up,
// so they accumulated in /tmp across every run. Playwright always runs
// globalTeardown after the full run completes, pass or fail, so this is the
// one place guaranteed to catch leftovers regardless of how the run ended.
export default async function globalTeardown() {
  const cacheDir = dirname(CHROMIUM_CACHE_DIR_PREFIX)
  const cacheDirNamePrefix = basename(CHROMIUM_CACHE_DIR_PREFIX)
  const entries = await readdir(cacheDir)
  const cacheDirs = entries.filter((entry) =>
    entry.startsWith(cacheDirNamePrefix),
  )

  await Promise.all(
    cacheDirs.map((entry) =>
      rm(join(cacheDir, entry), { recursive: true, force: true }),
    ),
  )
}
