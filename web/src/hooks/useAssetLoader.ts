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

async function readCachedBytes(
  resolvedUrl: string,
): Promise<Uint8Array | null> {
  try {
    const cache = await caches.open(CACHE_NAME)
    const cached = await cache.match(resolvedUrl)
    if (!cached) return null
    const buffer = await cached.arrayBuffer()
    return new Uint8Array(buffer)
  } catch {
    return null
  }
}

async function writeCachedBytes(
  resolvedUrl: string,
  bytes: Uint8Array,
): Promise<void> {
  try {
    const cache = await caches.open(CACHE_NAME)
    await cache.put(
      resolvedUrl,
      new Response(new Uint8Array(bytes), {
        headers: { 'Content-Type': 'application/octet-stream' },
      }),
    )
  } catch {
    // Cache API may be unavailable (e.g. Playwright, private mode).
  }
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
        const cachedBytes = await readCachedBytes(resolvedUrl)
        if (cachedBytes) {
          if (!cancelled) {
            setBytes(cachedBytes)
            setLoadedBytes(cachedBytes.byteLength)
            setTotalBytes(cachedBytes.byteLength)
            setStatus('ready')
          }
          return
        }

        const response = await fetch(resolvedUrl)
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`)
        }

        const total = Number(response.headers.get('content-length') ?? 0)
        if (!cancelled) setTotalBytes(total)

        const buffer = await response.arrayBuffer()
        const merged = new Uint8Array(buffer)

        await writeCachedBytes(resolvedUrl, merged)

        if (!cancelled) {
          setBytes(merged)
          setLoadedBytes(merged.byteLength)
          if (total === 0) setTotalBytes(merged.byteLength)
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
