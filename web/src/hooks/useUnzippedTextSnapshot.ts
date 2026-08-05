import { extract_unzipped_text } from 'jianpu-wasm'
import type { RefObject } from 'react'
import { useEffect } from 'react'
import { ensureWasmInit } from '../wasmInit'

/** Snapshots `extract_unzipped_text(source)` exactly when Unzipped view is
 * switched on, so subsequent keystrokes in the Unzipped editor aren't
 * overwritten by re-extraction while the user is still typing. `sourceRef`
 * is read only at the moment `unzippedView` flips to `true`, not on every
 * edit. */
export function useUnzippedTextSnapshot(
  unzippedView: boolean,
  sourceRef: RefObject<string>,
  setUnzippedText: (text: string) => void,
) {
  // biome-ignore lint/correctness/useExhaustiveDependencies: sourceRef/setUnzippedText read only at the moment unzippedView flips, not on every edit
  useEffect(() => {
    if (!unzippedView) return
    let cancelled = false
    ensureWasmInit().then(() => {
      if (cancelled) return
      const result = extract_unzipped_text(sourceRef.current)
      setUnzippedText(result.status === 'ok' ? result.text : '')
    })
    return () => {
      cancelled = true
    }
  }, [unzippedView])
}
