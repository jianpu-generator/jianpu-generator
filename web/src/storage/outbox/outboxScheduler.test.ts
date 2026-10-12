import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createScheduler } from './outboxScheduler'

type FakeTarget = {
  listeners: Map<string, Set<() => void>>
  addEventListener: (type: string, listener: () => void) => void
  removeEventListener: (type: string, listener: () => void) => void
  dispatch: (type: string) => void
  count: (type: string) => number
}

function fakeTarget(): FakeTarget {
  const listeners = new Map<string, Set<() => void>>()
  return {
    listeners,
    addEventListener: (type, listener) => {
      listeners.set(type, (listeners.get(type) ?? new Set()).add(listener))
    },
    removeEventListener: (type, listener) => {
      listeners.get(type)?.delete(listener)
    },
    dispatch: (type) => {
      for (const listener of [...(listeners.get(type) ?? [])]) listener()
    },
    count: (type) => listeners.get(type)?.size ?? 0,
  }
}

function setup() {
  const time = 1000
  let nextHandle = 1
  const timers = new Map<number, { callback: () => void; delayMs: number }>()
  const onWake = vi.fn()
  const setTimer = vi.fn((callback: () => void, delayMs: number) => {
    const handle = nextHandle++
    timers.set(handle, { callback, delayMs })
    return handle
  })
  const clearTimer = vi.fn((handle: number) => {
    timers.delete(handle)
  })
  const scheduler = createScheduler({
    now: () => time,
    setTimer,
    clearTimer,
    onWake,
  })
  return { scheduler, timers, onWake, setTimer, clearTimer }
}

describe('createScheduler', () => {
  let windowTarget: FakeTarget
  let documentTarget: FakeTarget & { visibilityState: string }

  beforeEach(() => {
    windowTarget = fakeTarget()
    documentTarget = { ...fakeTarget(), visibilityState: 'visible' }
    vi.stubGlobal('window', windowTarget)
    vi.stubGlobal('document', documentTarget)
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('schedules a timer with the delay relative to now', () => {
    const { scheduler, timers } = setup()
    scheduler.wakeAt(1500)
    expect([...timers.values()].map((t) => t.delayMs)).toEqual([500])
  })

  it('clamps past times to a zero delay', () => {
    const { scheduler, timers } = setup()
    scheduler.wakeAt(10)
    expect([...timers.values()].map((t) => t.delayMs)).toEqual([0])
  })

  it('replaces the earlier timer', () => {
    const { scheduler, timers, clearTimer } = setup()
    scheduler.wakeAt(1500)
    scheduler.wakeAt(2000)
    expect(clearTimer).toHaveBeenCalledTimes(1)
    expect([...timers.values()].map((t) => t.delayMs)).toEqual([1000])
  })

  it('clears the timer on null', () => {
    const { scheduler, timers } = setup()
    scheduler.wakeAt(1500)
    scheduler.wakeAt(null)
    expect(timers.size).toBe(0)
  })

  it('does not clear when nothing is pending', () => {
    const { scheduler, clearTimer } = setup()
    scheduler.wakeAt(null)
    expect(clearTimer).not.toHaveBeenCalled()
  })

  it('calls onWake when the timer fires and does not clear it afterwards', () => {
    const { scheduler, timers, onWake, clearTimer } = setup()
    scheduler.wakeAt(1500)
    for (const timer of [...timers.values()]) timer.callback()
    expect(onWake).toHaveBeenCalledTimes(1)
    scheduler.wakeAt(2000)
    expect(clearTimer).not.toHaveBeenCalled()
  })

  it('wakeNow calls onWake immediately', () => {
    const { scheduler, onWake } = setup()
    scheduler.wakeNow()
    expect(onWake).toHaveBeenCalledTimes(1)
  })

  it('wakes on the online event', () => {
    const { onWake } = setup()
    windowTarget.dispatch('online')
    expect(onWake).toHaveBeenCalledTimes(1)
  })

  it('wakes on visibilitychange only when visible', () => {
    const { onWake } = setup()
    documentTarget.visibilityState = 'hidden'
    documentTarget.dispatch('visibilitychange')
    expect(onWake).not.toHaveBeenCalled()
    documentTarget.visibilityState = 'visible'
    documentTarget.dispatch('visibilitychange')
    expect(onWake).toHaveBeenCalledTimes(1)
  })

  it('dispose clears the timer and removes listeners', () => {
    const { scheduler, timers, onWake } = setup()
    scheduler.wakeAt(1500)
    scheduler.dispose()
    expect(timers.size).toBe(0)
    expect(windowTarget.count('online')).toBe(0)
    expect(documentTarget.count('visibilitychange')).toBe(0)
    windowTarget.dispatch('online')
    documentTarget.dispatch('visibilitychange')
    expect(onWake).not.toHaveBeenCalled()
  })
})
