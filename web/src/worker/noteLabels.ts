import type { NoteLabels } from '../jianpuWasm'
import type { Notation } from './messages'

/** What the jianpu layout's note heads read as for the preview's notation:
 * the ABC view swaps each degree for its letter name. */
export function noteLabelsFor(notation: Notation): NoteLabels {
  return notation === 'abc' ? 'letter-names' : 'degrees'
}
