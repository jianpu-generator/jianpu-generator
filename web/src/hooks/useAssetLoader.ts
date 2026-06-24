import { useEffect, useState } from 'react'

export type AssetStatus = 'loading' | 'ready' | 'error'

export interface AssetLoaderState {
  bytes: Uint8Array | null
  status: AssetStatus
  loadedBytes: number
  totalBytes: number
}

const CACHE_NAME = 'jianpu-assets-v1'

function resolveAssetUrl(path: string): string {
  if (path.startsWith('http://') || path.startsWith('https://')) {
    return path
  }
  const relative = path.startsWith('/') ? path.slice(1) : path
  return `${import.meta.env.BASE_URL}${relative}`
}

export function useAssetLoader(url: string): AssetLoaderState {
  const [bytes, setBytes] = useState<Uint8Array | null>(null)
  const [status, setStatus] = useState<AssetStatus>('loading')
  const [loadedBytes, setLoadedBytes] = useState(0)
  const [totalBytes, setTotalBytes] = useState(0)

  useEffect(() => {
    let cancelled = false
    const resolvedUrl = resolveAssetUrl(url)

    async function load() {
      try {
        const cache = await caches.open(CACHE_NAME)
        const cached = await cache.match(resolvedUrl)
        if (cached) {
          const buffer = await cached.arrayBuffer()
          const cachedBytes = new Uint8Array(buffer)
          if (!cancelled) {
            setBytes(cachedBytes)
            setLoadedBytes(cachedBytes.byteLength)
            setTotalBytes(cachedBytes.byteLength)
            setStatus('ready')
          }
          return
        }

        const response = await fetch(resolvedUrl)
        const total = Number(response.headers.get('content-length') ?? 0)
        if (!cancelled) setTotalBytes(total)

        const reader = response.body?.getReader()
        if (!reader) throw new Error('Response body is not readable')
        const chunks: Uint8Array[] = []
        let received = 0

        while (true) {
          const { done, value } = await reader.read()
          if (done) break
          chunks.push(value)
          received += value.byteLength
          if (!cancelled) setLoadedBytes(received)
        }

        const merged = new Uint8Array(received)
        let offset = 0
        for (const chunk of chunks) {
          merged.set(chunk, offset)
          offset += chunk.byteLength
        }

        await cache
          .put(
            resolvedUrl,
            new Response(merged, {
              headers: { 'Content-Type': 'application/octet-stream' },
            }),
          )
          .catch(() => {})

        if (!cancelled) {
          setBytes(merged)
          setStatus('ready')
        }
      } catch {
        if (!cancelled) setStatus('error')
      }
    }

    load()
    return () => {
      cancelled = true
    }
  }, [url])

  return { bytes, status, loadedBytes, totalBytes }
}
