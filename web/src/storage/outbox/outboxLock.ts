const LOCK_NAME = 'jianpu-outbox-looper'
const STORE_LOCK_PREFIX = 'jianpu-outbox-store:'

/** Runs `task` while holding the lock, resolving to what `task` returns. */
export type StoreLock = <T>(task: () => Promise<T>) => Promise<T>

/** A lock within one tab: tasks run one at a time, in the order given. */
export function serialLock(): StoreLock {
  let tail: Promise<unknown> = Promise.resolve()
  return (task) => {
    const run = tail.then(task, task)
    tail = run.catch(() => undefined)
    return run
  }
}

/**
 * The lock every tab of `account` holds while it reads, changes and writes
 * the stored outbox queue, so no tab writes over a change it has not read.
 * Without `navigator.locks` it only orders this tab's own tasks.
 */
export function storeLock(account: string): StoreLock {
  if (typeof navigator === 'undefined' || !navigator.locks) return serialLock()
  const name = `${STORE_LOCK_PREFIX}${account}`
  return (task) => navigator.locks.request(name, task)
}

/**
 * Runs `task` only in the one tab that holds the outbox looper lock. Other tabs wait
 * for the lock; the browser hands it over when the holding tab closes. Returns a stop
 * function that aborts the task's signal and releases (or cancels waiting for) the lock.
 * Without `navigator.locks`, `task` simply runs (single-tab fallback).
 */
export function runAsLeader(
  task: (signal: AbortSignal) => Promise<void>,
): () => void {
  const controller = new AbortController()
  const { signal } = controller
  const stop = () => controller.abort()

  if (typeof navigator === 'undefined' || !navigator.locks) {
    void task(signal).catch(() => undefined)
    return stop
  }

  const aborted = new Promise<void>((resolve) => {
    signal.addEventListener('abort', () => resolve(), { once: true })
  })
  navigator.locks
    .request(LOCK_NAME, { signal }, async () => {
      await Promise.race([task(signal), aborted])
    })
    .catch(() => undefined)
  return stop
}
