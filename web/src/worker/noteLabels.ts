import type { NoteLabels } from '../jianpuWasm'
import type { Notation } from './messages'

/** What the jianpu layout's note heads read as for the preview's notation:
 * the ABC view swaps each degree for its letter name, and the guitar view
 * keeps degrees but draws a chord diagram at each chord symbol. */
export function noteLabelsFor(notation: Notation): NoteLabels {
  switch (notation) {
    case 'abc':
      return 'letter-names'
    case 'guitar':
      return 'guitar-chords'
    default:
      return 'degrees'
  }
}
