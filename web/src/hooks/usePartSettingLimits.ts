import { useEffect, useState } from 'react'
import { jianpuWasm, type PartSettingLimits } from '../jianpuWasm'
import { ensureWasmInit } from '../wasmInit'

let cachedPartSettingLimits: PartSettingLimits | null = null

function partSettingLimits(): PartSettingLimits {
  cachedPartSettingLimits ??= jianpuWasm().getPartSettingLimits()
  return cachedPartSettingLimits
}

/** The valid volume and octave-offset ranges of a part, owned by Rust.
 * `null` while `enabled` is false (so a closed modal costs nothing) or until
 * the main-thread wasm component is ready. */
export function usePartSettingLimits(
  enabled: boolean,
): PartSettingLimits | null {
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

  return enabled && wasmReady ? partSettingLimits() : null
}
