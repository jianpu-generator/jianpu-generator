import gmPercussionData from '../data/gmPercussion.json'
import type { SoundfontValue } from '../types'

export interface PercussionEntry {
  value: SoundfontValue
  key: number
}

// Source of truth is data/gmPercussion.json, shared with the Rust side
// (see soundfont_program_to_label in src/lib.rs) so the two never drift.
export const GM_PERCUSSION: PercussionEntry[] = gmPercussionData.map(
  ({ key, name }) => ({ value: `${key}: ${name}` as SoundfontValue, key }),
)
