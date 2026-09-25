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
// as it finishes (see e2e-pass-cache-reporter.mjs), and a later invocation on
// the same fingerprint only runs the tests that haven't passed yet. This makes
// re-attempting the same commit (after a killed hook, a failing non-e2e check,
// or a genuine e2e failure that turned out to be a flake) cheap. Any change to
// the code yields a new fingerprint and therefore a full run — a pass is only
// ever reused for byte-identical code. Pass `--no-pass-cache` to discard the
// recorded passes for the current code and run everything fresh (the fresh
// passes are still recorded for the next invocation).
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

const LAST_RUN_FILE = join('test-results', '.last-run.json')
const PASS_CACHE_DIR = '.e2e-pass-cache'
const PASS_CACHE_REPORTER = './scripts/e2e-pass-cache-reporter.mjs'
const NO_PASS_CACHE_FLAG = '--no-pass-cache'
const STABLE_STREAK_TO_CONFIRM = 3
const MAX_PASSES = 15

function run(args, passCacheFile) {
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
}

function git(args, { cwd, input } = {}) {
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
function codeFingerprint() {
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
function openPassCache(fingerprint, { fresh }) {
  mkdirSync(PASS_CACHE_DIR, { recursive: true })
  for (const entry of readdirSync(PASS_CACHE_DIR)) {
    if (fresh || entry !== fingerprint) rmSync(join(PASS_CACHE_DIR, entry))
  }
  const file = resolve(PASS_CACHE_DIR, fingerprint)
  let passed = new Set()
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

function listTestIds(args) {
  const result = spawnSync(
    'pnpm',
    ['exec', 'playwright', 'test', '--list', '--reporter=json', ...args],
    { encoding: 'utf-8', maxBuffer: 1 << 30 },
  )
  if (result.error) throw result.error
  if (result.status !== 0) {
    throw new Error(`playwright test --list failed:\n${result.stderr}`)
  }
  const collectIds = (suite) => [
    ...(suite.specs ?? []).map((spec) => spec.id),
    ...(suite.suites ?? []).flatMap(collectIds),
  ]
  return JSON.parse(result.stdout).suites.flatMap(collectIds)
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
  // `pnpm test:e2e:resolve -- --grep ...` forwards that separating `--`
  // itself as a literal arg (unlike `npm run`, which swallows it) -- left
  // in, it becomes `playwright test -- --grep ...`, and Playwright treats
  // everything after a `--` as positional file-path filters rather than
  // flags, silently discarding `--grep` and running the whole suite
  // instead of just the scoped test. Stripping a single leading `--` here
  // keeps both `pnpm run` (which already swallows it) and `pnpm` (which
  // doesn't) working the same way.
  const cliArgs = process.argv.slice(2)
  if (cliArgs[0] === '--') cliArgs.shift()
  // Our own flag is consumed here rather than forwarded, since Playwright
  // would reject it as unknown.
  const extraArgs = cliArgs.filter((arg) => arg !== NO_PASS_CACHE_FLAG)
  const fresh = extraArgs.length !== cliArgs.length

  const passCache = openPassCache(codeFingerprint(), { fresh })
  if (passCache.passed.size === 0) {
    run(extraArgs, passCache.file)
  } else {
    const notYetPassed = listTestIds(extraArgs).filter(
      (id) => !passCache.passed.has(id),
    )
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
    mkdirSync('test-results', { recursive: true })
    writeFileSync(
      LAST_RUN_FILE,
      JSON.stringify({ status: 'failed', failedTests: notYetPassed }),
    )
    run(['--last-failed'], passCache.file)
  }
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
    run(['--last-failed'], passCache.file)
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
