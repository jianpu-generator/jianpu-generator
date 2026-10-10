import type { OutboxChangeChannel } from './outboxLooper'

/** Tabs of one account tell each other when they change the stored queue,
 * so the leader tab delivers messages queued from a follower tab. Without
 * `BroadcastChannel` (or in tests) there is nobody to tell. */
export function broadcastChanges(
  account: string,
): OutboxChangeChannel | undefined {
  if (typeof BroadcastChannel === 'undefined') return undefined
  const name = `jianpu-outbox:${account}`
  const sender = new BroadcastChannel(name)
  return {
    announce: () => sender.postMessage('changed'),
    listen: (onChange) => {
      const receiver = new BroadcastChannel(name)
      receiver.onmessage = onChange
      return () => receiver.close()
    },
  }
}
