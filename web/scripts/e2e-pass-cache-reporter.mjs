// Playwright reporter used by resolve-e2e-flakes.mjs: appends the id of every
// test that passes to the pass-cache file named by E2E_PASS_CACHE_FILE, as
// soon as that test finishes. Appending per test (instead of writing once in
// onEnd) means progress survives a run that gets killed partway through —
// e.g. the pre-commit hook being interrupted — so the next commit attempt of
// the same code doesn't have to re-run what already passed.
import { appendFileSync } from 'node:fs'

export default class E2ePassCacheReporter {
  printsToStdio() {
    return false
  }

  onTestEnd(test, result) {
    const cacheFile = process.env.E2E_PASS_CACHE_FILE
    if (
      cacheFile &&
      test.expectedStatus === 'passed' &&
      result.status === 'passed'
    ) {
      appendFileSync(cacheFile, `${test.id}\n`)
    }
  }
}
