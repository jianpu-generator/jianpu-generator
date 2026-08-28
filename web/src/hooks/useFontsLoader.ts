import fontsManifest from '../../../fonts/fonts.json'
import type { AssetStatus } from './useAssetLoader'
import { useAssetLoader } from './useAssetLoader'

export interface FontsLoaderState {
  fonts: {
    sc: Uint8Array
    tc: Uint8Array
    mono: Uint8Array
  } | null
  status: AssetStatus
  loadedBytes: number
  totalBytes: number
}

export function useFontsLoader(): FontsLoaderState {
  // `sc` holds the `title` role's font — the song title/subtitle/author/
  // lyric font (see `FontFamily::Title` in src/compositor/types.rs),
  // despite the name; `tc` holds the `sansSerif` role's font, the separate
  // default/body font for everything else (directive line, part legend,
  // footer) — currently Source Han Sans SC, a different file from `sc`'s
  // Zhuque Fangsong. Filenames/family names come from `fonts/fonts.json`,
  // the single source of truth for which font backs each role — see its own
  // comments and `src/fonts.rs` on the Rust side.
  const sc = useAssetLoader(`/fonts/${fontsManifest.title.filename}`)
  const tc = useAssetLoader(`/fonts/${fontsManifest.sansSerif.filename}`)
  const mono = useAssetLoader(`/fonts/${fontsManifest.monospace.filename}`)

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
