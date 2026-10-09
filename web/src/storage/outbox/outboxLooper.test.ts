import { readFileSync } from 'node:fs'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import { instantiate } from '../../../../crates/jianpu-wasm/pkg-component/jianpu_wasm.js'
import {
  type SendRequest,
  type SendResult,
  setWasmRoot,
} from '../../jianpuWasm'
import { createMemoryOutboxStore } from './memoryOutboxStore'
import { createOutboxLooper } from './outboxLooper'
import type { Scheduler } from './outboxScheduler'
import type { OutboxStore } from './outboxStore'

beforeAll(async () => {
  const bytes = readFileSync(
    new URL(
      '../../../../crates/jianpu-wasm/pkg-component/jianpu_wasm.core.wasm',
      import.meta.url,
    ),
  )
  const module = await WebAssembly.compile(bytes)
  setWasmRoot(await instantiate(() => module, {}))
})

const ok = (revision: bigint): SendResult => ({
  tag: 'ok',
  val: { revision },
})

const save = (content: string) => ({
  tag: 'save-content' as const,
  val: { content },
})

function makeHarness(
  options: {
    store?: OutboxStore
    send?: (request: SendRequest) => Promise<SendResult>
  } = {},
) {
  let time = 1_000_000
  const store = options.store ?? createMemoryOutboxStore()
  const sent: SendRequest[] = []
  const wakes: Array<number | null> = []
  let onWake: () => void = () => undefined
  const send = vi.fn(async (request: SendRequest) => {
    sent.push(request)
    return options.send ? options.send(request) : ok(BigInt(sent.length))
  })
  const scheduler: Scheduler = {
    wakeAt: (ms) => {
      wakes.push(ms)
    },
    wakeNow: () => onWake(),
    dispose: () => undefined,
  }
  const looper = createOutboxLooper({
    store,
    sendMessage: send,
    createScheduler: (wake) => {
      onWake = wake
      return scheduler
    },
    runAsLeader: (task) => {
      const controller = new AbortController()
      void task(controller.signal)
      return () => controller.abort()
    },
    now: () => time,
    random: () => 0,
  })
  return {
    looper,
    store,
    sent,
    wakes,
    wake: () => onWake(),
    advance: (ms: number) => {
      time += ms
    },
  }
}

const settle = () => new Promise((resolve) => setTimeout(resolve, 0))

describe('outbox looper', () => {
  it('persists an enqueued message before sending it', async () => {
    let storedWhenSent: string[] = []
    const store = createMemoryOutboxStore()
    const harness = makeHarness({
      store,
      send: async () => {
        storedWhenSent = (await store.loadAll()).map((record) => record.key)
        return ok(1n)
      },
    })
    await harness.looper.start()
    await harness.looper.enqueue('file-a', save('hello'))
    await harness.looper.whenIdle()
    expect(storedWhenSent).toContain('lane:file-a')
  })

  it('clears the message from the stored lane once it is acknowledged', async () => {
    const harness = makeHarness()
    await harness.looper.start()
    await harness.looper.enqueue('file-a', save('hello'))
    await harness.looper.whenIdle()
    const lane = (await harness.store.loadAll()).find(
      (record) => record.key === 'lane:file-a',
    )
    expect(JSON.parse(lane?.value ?? '{}').messages).toEqual([])
    expect(harness.looper.snapshot().summary.hasUnsynced).toBe(false)
    expect(harness.sent).toHaveLength(1)
  })

  it('waits after a transient failure and retries on wake', async () => {
    let attempts = 0
    const harness = makeHarness({
      send: async () => {
        attempts += 1
        return attempts === 1
          ? { tag: 'transient', val: { reason: 'offline' } }
          : ok(1n)
      },
    })
    await harness.looper.start()
    await harness.looper.enqueue('file-a', save('hello'))
    await harness.looper.whenIdle()
    expect(harness.sent).toHaveLength(1)
    expect(harness.wakes.at(-1)).toBeGreaterThan(1_000_000)

    harness.wake()
    await settle()
    await harness.looper.whenIdle()
    expect(harness.sent).toHaveLength(1)

    harness.advance(120_000)
    harness.wake()
    await settle()
    await harness.looper.whenIdle()
    expect(harness.sent).toHaveLength(2)
    expect(harness.looper.snapshot().summary.hasUnsynced).toBe(false)
  })

  it('does not let a halted lane block another file', async () => {
    const harness = makeHarness({
      send: async (request) =>
        request.fileId === 'file-a'
          ? { tag: 'permanent', val: { status: 400, reason: 'bad' } }
          : ok(1n),
    })
    await harness.looper.start()
    await harness.looper.enqueue('file-a', save('a'))
    await harness.looper.enqueue('file-b', save('b'))
    await harness.looper.whenIdle()
    const { summary } = harness.looper.snapshot()
    expect(summary.lanes.map((lane) => lane.fileId)).toEqual(['file-a'])
    expect(summary.lanesNeedingAttention).toBe(1)
    expect(harness.sent.map((request) => request.fileId)).toContain('file-b')
  })

  it('restores pending messages after a restart', async () => {
    const store = createMemoryOutboxStore()
    const offline = makeHarness({
      store,
      send: async () => ({ tag: 'transient', val: { reason: 'offline' } }),
    })
    await offline.looper.start()
    await offline.looper.enqueue('file-a', save('kept'))
    await offline.looper.whenIdle()
    offline.looper.stop()

    const restarted = makeHarness({ store })
    await restarted.looper.start()
    await settle()
    await restarted.looper.whenIdle()
    expect(restarted.sent[0]?.message).toEqual(save('kept'))
    expect(restarted.looper.snapshot().summary.hasUnsynced).toBe(false)
  })

  it('halts on an unsupported version without touching stored data', async () => {
    const store = createMemoryOutboxStore()
    await store.apply({
      puts: [{ key: 'meta:version', value: '999' }],
      deletes: [],
    })
    const before = await store.loadAll()
    const harness = makeHarness({ store })
    await harness.looper.start()
    await settle()
    expect(harness.looper.snapshot().halted?.tag).toBe('unsupported-version')
    await expect(harness.looper.enqueue('file-a', save('x'))).rejects.toThrow()
    expect(harness.sent).toHaveLength(0)
    expect(await store.loadAll()).toEqual(before)
  })

  it('leaves records it does not own in the store', async () => {
    const store = createMemoryOutboxStore()
    await store.apply({
      puts: [{ key: 'base:file-a', value: 'snapshot' }],
      deletes: [],
    })
    const harness = makeHarness({ store })
    await harness.looper.start()
    await harness.looper.enqueue('file-a', save('x'))
    await harness.looper.whenIdle()
    const keys = (await store.loadAll()).map((record) => record.key)
    expect(keys).toContain('base:file-a')
  })

  it('notifies subscribers with the summary', async () => {
    const harness = makeHarness()
    const seen: boolean[] = []
    await harness.looper.start()
    harness.looper.subscribe((snapshot) => {
      seen.push(snapshot.summary.hasUnsynced)
    })
    await harness.looper.enqueue('file-a', save('x'))
    await harness.looper.whenIdle()
    expect(seen).toContain(true)
    expect(seen.at(-1)).toBe(false)
  })
})
