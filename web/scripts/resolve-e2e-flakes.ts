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
//
// Passes are also remembered *across* invocations, keyed on a fingerprint of
// the exact code under test (staged tree + unstaged diff + untracked files):
// every test that passes is recorded in .e2e-pass-cache/<fingerprint> as soon
// as it finishes (see e2e-pass-cache-reporter.ts), and a later invocation on
// the same fingerprint only runs the tests that haven't passed yet. This makes
// re-attempting the same commit (after a killed hook, a failing non-e2e check,
// or a genuine e2e failure that turned out to be a flake) cheap. Any change to
// the code yields a new fingerprint and therefore a full run — a pass is only
// ever reused for byte-identical code. Pass `--no-pass-cache` to discard the
// recorded passes for the current code and run everything fresh (the fresh
// passes are still recorded for the next invocation).
//
// Usage: resolve-e2e-flakes [--no-pass-cache] [-- <playwright test args...>]
import { spawnSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import {
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs'
import { join, resolve } from 'node:path'
import { Command } from 'commander'

const LAST_RUN_FILE = join('test-results', '.last-run.json')
const PASS_CACHE_DIR = '.e2e-pass-cache'
const PASS_CACHE_REPORTER = './scripts/e2e-pass-cache-reporter.ts'
const STABLE_STREAK_TO_CONFIRM = 3
const MAX_PASSES = 15
// Written into LAST_RUN_FILE before every pass, so a file still carrying it
// afterwards is one Playwright never rewrote — i.e. the pass never got as far
// as running tests — rather than a genuine (possibly stale) result.
const SEEDED_STATUS = 'seeded-by-resolve-e2e-flakes'

// The shape Playwright writes to LAST_RUN_FILE (and reads back for
// `--last-failed`).
interface LastRun {
  status: string
  failedTests?: string[]
}

interface PassCache {
  file: string
  passed: Set<string>
}

interface JsonReportSuite {
  specs?: { id: string }[]
  suites?: JsonReportSuite[]
}

interface CliOptions {
  passCache: boolean
}

function fail(message: string): never {
  console.error(`e2e: ${message}`)
  process.exit(1)
}

function run(args: string[], passCacheFile: string): number | null {
  const result = spawnSync(
    'pnpm',
    [
      'exec',
      'playwright',
      'test',
      `--reporter=list,${PASS_CACHE_REPORTER}`,
      ...args,
    ],
    {
      stdio: 'inherit',
      env: { ...process.env, E2E_PASS_CACHE_FILE: passCacheFile },
    },
  )
  if (result.error) throw result.error
  return result.status
}

function git(
  args: string[],
  { cwd, input }: { cwd?: string; input?: string } = {},
): Buffer {
  const result = spawnSync('git', args, { cwd, input, maxBuffer: 1 << 30 })
  if (result.error) throw result.error
  if (result.status !== 0) {
    throw new Error(`git ${args.join(' ')} failed:\n${result.stderr}`)
  }
  return result.stdout
}

// Identifies the exact code the suite is about to run against. The staged
// tree alone isn't enough: the hook runs against the working tree, so
// unstaged edits and untracked files can change what's under test too.
function codeFingerprint(): string {
  const repoRoot = git(['rev-parse', '--show-toplevel']).toString().trim()
  const untrackedPaths = git(
    ['ls-files', '--others', '--exclude-standard', '-z'],
    { cwd: repoRoot },
  )
    .toString()
    .split('\0')
    .filter((path) => path !== '')
  const untrackedContentHashes =
    untrackedPaths.length === 0
      ? ''
      : git(['hash-object', '--stdin-paths'], {
          cwd: repoRoot,
          input: untrackedPaths.join('\n'),
        })
  return createHash('sha256')
    .update(git(['write-tree'], { cwd: repoRoot }))
    .update(git(['diff', '--binary'], { cwd: repoRoot }))
    .update(untrackedPaths.join('\0'))
    .update(untrackedContentHashes)
    .digest('hex')
}

// Returns the cache file for `fingerprint` and the tests already recorded as
// passed in it. Cache files for any other fingerprint are stale (that code is
// no longer what's being committed), so they're deleted rather than left to
// accumulate; with `fresh`, the one for `fingerprint` is deleted too.
function openPassCache(
  fingerprint: string,
  { fresh }: { fresh: boolean },
): PassCache {
  mkdirSync(PASS_CACHE_DIR, { recursive: true })
  for (const entry of readdirSync(PASS_CACHE_DIR)) {
    if (fresh || entry !== fingerprint) rmSync(join(PASS_CACHE_DIR, entry))
  }
  const file = resolve(PASS_CACHE_DIR, fingerprint)
  let passed = new Set<string>()
  try {
    passed = new Set(
      readFileSync(file, 'utf-8')
        .split('\n')
        .filter((id) => id !== ''),
    )
  } catch {
    // No cache yet for this fingerprint.
  }
  return { file, passed }
}

function listTestIds(args: string[]): string[] {
  const result = spawnSync(
    'pnpm',
    ['exec', 'playwright', 'test', '--list', '--reporter=json', ...args],
    { encoding: 'utf-8', maxBuffer: 1 << 30 },
  )
  if (result.error) throw result.error
  if (result.status !== 0) {
    throw new Error(`playwright test --list failed:\n${result.stderr}`)
  }
  const collectIds = (suite: JsonReportSuite): string[] => [
    ...(suite.specs ?? []).map((spec) => spec.id),
    ...(suite.suites ?? []).flatMap(collectIds),
  ]
  const report: { suites: JsonReportSuite[] } = JSON.parse(result.stdout)
  return report.suites.flatMap(collectIds)
}

function seedLastRun(failedTests: string[]): void {
  mkdirSync('test-results', { recursive: true })
  writeFileSync(
    LAST_RUN_FILE,
    JSON.stringify({ status: SEEDED_STATUS, failedTests } satisfies LastRun),
  )
}

// Runs one pass and returns the set of tests still failing after it. Fails
// closed: anything short of Playwright actually running the tests and
// reporting on them (webServer that never starts, config error, no tests
// matched, missing/unparseable/unrewritten .last-run.json) aborts the whole
// script instead of being read as "nothing failed". Playwright reports all of
// those as a non-zero exit with an empty `failedTests`, which is
// indistinguishable from a clean run unless the exit code is checked too.
function runPass(args: string[], passCacheFile: string): Set<string> {
  const exitCode = run(args, passCacheFile)
  let lastRun: LastRun
  try {
    lastRun = JSON.parse(readFileSync(LAST_RUN_FILE, 'utf-8'))
  } catch {
    fail(
      `Playwright exited ${exitCode} without writing a readable ${LAST_RUN_FILE}; ` +
        'no test results to go on, aborting.',
    )
  }
  if (lastRun.status === SEEDED_STATUS) {
    fail(
      `Playwright exited ${exitCode} without rewriting ${LAST_RUN_FILE} — ` +
        'it never got as far as running tests; aborting.',
    )
  }
  const failedTests = lastRun.failedTests ?? []
  if (exitCode !== 0 && failedTests.length === 0) {
    fail(
      `Playwright exited ${exitCode} (run status "${lastRun.status}") without any failing test — ` +
        'it failed before running tests (e.g. a webServer that did not start, a config error, ' +
        'or no tests matched). See the Playwright output above; aborting.',
    )
  }
  return new Set(failedTests)
}

function setsEqual(a: Set<string>, b: Set<string>): boolean {
  return a.size === b.size && [...a].every((id) => b.has(id))
}

function parseCli(): { options: CliOptions; playwrightArgs: string[] } {
  const program = new Command()
    .name('resolve-e2e-flakes')
    .description(
      'Run the Playwright e2e suite, re-running only still-failing tests until the failing set settles.',
    )
    .option(
      '--no-pass-cache',
      'discard the passes recorded for the current code and run every test fresh',
    )
    .argument(
      '[playwright-args...]',
      'extra `playwright test` args for the first pass (e.g. --grep), after a `--`',
    )
    .showHelpAfterError(
      '(Playwright args such as --grep must come after a `--`, e.g. `pnpm test:e2e:resolve -- --grep foo`.)',
    )
    .parse()
  return {
    options: program.opts<CliOptions>(),
    playwrightArgs: program.processedArgs[0] as string[],
  }
}

function main(): void {
  // Playwright args (e.g. --grep) scope only the first pass; --last-failed
  // reruns are already scoped to that pass's failures, so they don't need
  // repeating.
  const { options, playwrightArgs } = parseCli()

  spawnSync('pnpm', ['exec', 'bddgen'], { stdio: 'inherit' })

  const passCache = openPassCache(codeFingerprint(), {
    fresh: !options.passCache,
  })
  let failing: Set<string>
  if (passCache.passed.size === 0) {
    seedLastRun([])
    failing = runPass(playwrightArgs, passCache.file)
  } else {
    const testIds = listTestIds(playwrightArgs)
    if (testIds.length === 0) {
      fail('no tests matched; nothing was run, aborting.')
    }
    const notYetPassed = testIds.filter((id) => !passCache.passed.has(id))
    if (notYetPassed.length === 0) {
      console.error(
        'e2e: every test already passed against this exact code in an earlier run; skipping.',
      )
      return
    }
    console.error(
      `e2e: ${passCache.passed.size} test(s) already passed against this exact code in an earlier run; ` +
        `running only the ${notYetPassed.length} that haven't...`,
    )
    // Seeds --last-failed with exactly the not-yet-passed tests, so this
    // first pass skips everything the cache already vouches for.
    seedLastRun(notYetPassed)
    failing = runPass(['--last-failed'], passCache.file)
  }

  const everFailed = new Set(failing)
  const history = [failing]

  for (let pass = 2; failing.size > 0; pass++) {
    const last3 = history.slice(-STABLE_STREAK_TO_CONFIRM)
    if (
      last3.length === STABLE_STREAK_TO_CONFIRM &&
      last3.every((s) => setsEqual(s, failing))
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
    seedLastRun([...failing])
    failing = runPass(['--last-failed'], passCache.file)
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
