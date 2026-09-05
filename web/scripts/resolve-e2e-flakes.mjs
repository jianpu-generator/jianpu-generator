#!/usr/bin/env node
// Resolves e2e flakiness instead of masking it with in-run retries
// (playwright.config.ts sets `retries: 0` for exactly this reason).
//
// Algorithm: run the full suite, then keep re-running only the tests still
// failing (`playwright test --last-failed`) until the same failing set shows
// up 3 passes in a row. --last-failed only re-executes tests that were
// already failing, so the failing *count* can never grow from one pass to
// the next — but its *membership* can still shuffle indefinitely (e.g. two
// independently-flaky tests where exactly one fails each pass, alternating)
// without ever shrinking or repeating 3 times in a row. MAX_PASSES exists
// for that case: it's a real possibility, not just a fuse against script
// bugs.
import { spawnSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { join } from 'node:path'

const LAST_RUN_FILE = join('test-results', '.last-run.json')
const STABLE_STREAK_TO_CONFIRM = 3
const MAX_PASSES = 15

function run(args) {
  const result = spawnSync('pnpm', ['exec', 'playwright', 'test', ...args], {
    stdio: 'inherit',
  })
  if (result.error) throw result.error
}

function readFailingSet() {
  let parsed
  try {
    parsed = JSON.parse(readFileSync(LAST_RUN_FILE, 'utf-8'))
  } catch {
    // No file (or unreadable) means playwright didn't get far enough to
    // write one — treat as "everything still failing" so the caller doesn't
    // mistake this for a clean run.
    return null
  }
  return new Set(parsed.failedTests ?? [])
}

function setsEqual(a, b) {
  return a.size === b.size && [...a].every((id) => b.has(id))
}

function main() {
  spawnSync('pnpm', ['exec', 'bddgen'], { stdio: 'inherit' })

  // Extra CLI args (e.g. --grep) scope the first pass; --last-failed reruns
  // are already scoped to that pass's failures, so they don't need repeating.
  const extraArgs = process.argv.slice(2)
  run(extraArgs)
  let failing = readFailingSet()
  if (failing === null) {
    console.error(
      'e2e: no test-results/.last-run.json after the first pass; aborting.',
    )
    process.exit(1)
  }

  const everFailed = new Set(failing)
  const history = [failing]

  for (let pass = 2; failing.size > 0; pass++) {
    const last3 = history.slice(-STABLE_STREAK_TO_CONFIRM)
    if (
      last3.length === STABLE_STREAK_TO_CONFIRM &&
      last3.every((s) => setsEqual(s, last3[0]))
    ) {
      const flaky = [...everFailed].filter((id) => !failing.has(id))
      console.error(
        `e2e: ${failing.size} test(s) failed ${STABLE_STREAK_TO_CONFIRM} passes in a row — genuine failures, not flakes:\n` +
          [...failing].join('\n'),
      )
      if (flaky.length > 0) {
        console.error(
          `\ne2e: ${flaky.length} test(s) were flaky but eventually passed:\n${flaky.join('\n')}`,
        )
      }
      process.exit(1)
    }

    if (pass > MAX_PASSES) {
      console.error(
        `e2e: failing set didn't settle into 3 identical passes within ${MAX_PASSES} passes — its membership keeps ` +
          `shuffling (likely multiple independently-flaky tests). Treating the current set as failing rather than ` +
          `looping forever:\n${[...failing].join('\n')}`,
      )
      process.exit(1)
    }

    console.error(
      `e2e: pass ${pass}, re-running ${failing.size} previously-failing test(s)...`,
    )
    run(['--last-failed'])
    failing = readFailingSet() ?? failing
    for (const id of failing) everFailed.add(id)
    history.push(failing)
  }

  if (everFailed.size > 0) {
    console.error(
      `e2e: ${everFailed.size} test(s) were flaky but eventually passed:\n${[...everFailed].join('\n')}`,
    )
  }
  console.error('e2e: all tests passed (after resolving flakes).')
}

main()
