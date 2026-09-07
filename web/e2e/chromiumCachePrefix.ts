// Shared between playwright.config.ts (which points each worker's Chromium
// at `${CHROMIUM_CACHE_DIR_PREFIX}${process.pid}` via --disk-cache-dir, see
// the comment there) and global-teardown.ts (which globs this prefix to
// clean up the leftover cache dirs after the run). Kept in one place so the
// two can't drift apart.
export const CHROMIUM_CACHE_DIR_PREFIX = '/tmp/chromium-e2e-cache-'
