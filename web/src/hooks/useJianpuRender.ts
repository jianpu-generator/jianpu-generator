import { useEffect, useRef, useState } from 'react'
import type { WorkerRequest, WorkerResponse } from '../worker/jianpu.worker'
import type { RenderError } from '../types'

interface RenderState {
  svgs: string[]
  error: RenderError | null
  rendering: boolean
}

export function useJianpuRender(source: string, debounceMs = 300): RenderState {
  const [svgs, setSvgs] = useState<string[]>([])
  const [error, setError] = useState<RenderError | null>(null)
  const [rendering, setRendering] = useState(false)

  const workerRef = useRef<Worker | null>(null)
  const requestIdRef = useRef(0)
  const latestIdRef = useRef(0)

  useEffect(() => {
    const worker = new Worker(
      new URL('../worker/jianpu.worker.ts', import.meta.url),
      { type: 'module' },
    )
    workerRef.current = worker

    worker.onmessage = (event: MessageEvent<WorkerResponse>) => {
      const msg = event.data
      if (msg.type === 'ready') return
      if (msg.id !== latestIdRef.current) return

      setRendering(false)
      if (msg.type === 'ok') {
        setSvgs(msg.svgs)
        setError(null)
      } else {
        setSvgs([])
        setError(msg.error)
      }
    }

    return () => {
      worker.terminate()
      workerRef.current = null
    }
  }, [])

  useEffect(() => {
    const worker = workerRef.current
    if (!worker) return

    const id = ++requestIdRef.current
    latestIdRef.current = id
    setRendering(true)

    const timer = window.setTimeout(() => {
      const payload: WorkerRequest = { type: 'render', source, id }
      worker.postMessage(payload)
    }, debounceMs)

    return () => window.clearTimeout(timer)
  }, [source, debounceMs])

  return { svgs, error, rendering }
}
