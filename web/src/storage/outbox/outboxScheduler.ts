export type SchedulerOptions<Handle> = {
  now: () => number
  setTimer: (callback: () => void, delayMs: number) => Handle
  clearTimer: (handle: Handle) => void
  onWake: () => void
}

export type Scheduler = {
  wakeAt: (ms: number | null) => void
  wakeNow: () => void
  dispose: () => void
}

export function createScheduler<Handle>(
  options: SchedulerOptions<Handle>,
): Scheduler {
  const { now, setTimer, clearTimer, onWake } = options
  let pending: { handle: Handle } | null = null

  const clear = () => {
    if (pending) clearTimer(pending.handle)
    pending = null
  }

  const handleOnline = () => onWake()
  const handleVisibilityChange = () => {
    if (document.visibilityState === 'visible') onWake()
  }
  window.addEventListener('online', handleOnline)
  document.addEventListener('visibilitychange', handleVisibilityChange)

  return {
    wakeAt: (ms) => {
      clear()
      if (ms === null) return
      pending = {
        handle: setTimer(
          () => {
            pending = null
            onWake()
          },
          Math.max(0, ms - now()),
        ),
      }
    },
    wakeNow: () => onWake(),
    dispose: () => {
      clear()
      window.removeEventListener('online', handleOnline)
      document.removeEventListener('visibilitychange', handleVisibilityChange)
    },
  }
}
