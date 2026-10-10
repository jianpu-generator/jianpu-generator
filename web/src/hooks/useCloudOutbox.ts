import { useEffect, useState } from 'react'
import type { CloudBackend } from '../storage/cloudBackendTypes'
import type { OutboxSnapshot } from '../storage/outbox/outboxLooper'
import type { StorageBackend } from '../storage/types'

/**
 * The cloud backend's outbox state (null on other backends). Also releases the
 * backend's outbox leader lock when the backend is replaced; without that the
 * replacement never becomes leader and nothing is ever delivered.
 */
export function useCloudOutbox(
  backend: StorageBackend | CloudBackend,
): OutboxSnapshot | null {
  const [snapshot, setSnapshot] = useState<OutboxSnapshot | null>(null)
  useEffect(() => {
    if (backend.kind !== 'cloud') {
      setSnapshot(null)
      return
    }
    const cloud = backend as CloudBackend
    const looper = cloud.outbox()
    setSnapshot(looper.snapshot())
    const unsubscribe = looper.subscribe(setSnapshot)
    return () => {
      unsubscribe()
      cloud.dispose()
    }
  }, [backend])
  return snapshot
}
