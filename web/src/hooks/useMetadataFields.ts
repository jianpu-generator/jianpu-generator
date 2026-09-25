import { useEffect, useMemo, useState } from 'react'
import { jianpuWasm, type MetadataFields } from '../jianpuWasm'
import { ensureWasmInit } from '../wasmInit'

/** The `# metadata` section's field values, parsed by the Rust metadata
 * parser. `null` while `enabled` is false (so a closed Edit Metadata modal
 * costs nothing per keystroke) or until the main-thread wasm component is
 * ready. */
export function useMetadataFields(
  source: string,
  enabled: boolean,
): MetadataFields | null {
  const [wasmReady, setWasmReady] = useState(false)

  useEffect(() => {
    if (!enabled) return
    let cancelled = false
    void ensureWasmInit().then(() => {
      if (!cancelled) setWasmReady(true)
    })
    return () => {
      cancelled = true
    }
  }, [enabled])

  return useMemo(
    () =>
      enabled && wasmReady ? jianpuWasm().parseMetadataFields(source) : null,
    [enabled, wasmReady, source],
  )
}
