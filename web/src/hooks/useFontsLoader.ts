import type { AssetStatus } from './useAssetLoader'
import { useAssetLoader } from './useAssetLoader'

export interface FontsLoaderState {
  fonts: { sc: Uint8Array; tc: Uint8Array; mono: Uint8Array } | null
  status: AssetStatus
  loadedBytes: number
  totalBytes: number
}

export function useFontsLoader(): FontsLoaderState {
  const sc = useAssetLoader('/fonts/SourceHanSansSC-Regular.otf')
  const tc = useAssetLoader('/fonts/SourceHanSansTC-Regular.otf')
  const mono = useAssetLoader('/fonts/NotoSansMono-Regular.ttf')

  const status: AssetStatus =
    sc.status === 'error' || tc.status === 'error' || mono.status === 'error'
      ? 'error'
      : sc.status === 'ready' &&
          tc.status === 'ready' &&
          mono.status === 'ready'
        ? 'ready'
        : 'loading'

  const fonts =
    sc.bytes && tc.bytes && mono.bytes
      ? { sc: sc.bytes, tc: tc.bytes, mono: mono.bytes }
      : null

  return {
    fonts,
    status,
    loadedBytes: sc.loadedBytes + tc.loadedBytes + mono.loadedBytes,
    totalBytes: sc.totalBytes + tc.totalBytes + mono.totalBytes,
  }
}
