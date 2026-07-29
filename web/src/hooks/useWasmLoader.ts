import { useEffect, useState } from 'react'
import { ensureWasmModule, subscribeWasmProgress } from '../wasmInit'
import type { AssetStatus } from './useAssetLoader'

export interface WasmLoaderState {
  status: AssetStatus
  loadedBytes: number
  totalBytes: number
}

export function useWasmLoader(): WasmLoaderState {
  const [status, setStatus] = useState<AssetStatus>('loading')
  const [loadedBytes, setLoadedBytes] = useState(0)
  const [totalBytes, setTotalBytes] = useState(0)

  useEffect(() => {
    let cancelled = false
    const unsubscribe = subscribeWasmProgress((loaded, total) => {
      if (cancelled) return
      setLoadedBytes(loaded)
      setTotalBytes(total)
    })

    ensureWasmModule()
      .then(() => {
        if (!cancelled) setStatus('ready')
      })
      .catch(() => {
        if (!cancelled) setStatus('error')
      })

    return () => {
      cancelled = true
      unsubscribe()
    }
  }, [])

  return { status, loadedBytes, totalBytes }
}
