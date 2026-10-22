import type { NoteLabels } from '../jianpuWasm'
import type { Notation } from './messages'

const NOTE_LABELS: Record<Notation, NoteLabels> = {
  jianpu: 'degrees',
  staff: 'degrees',
  abc: 'letter-names',
  harmonica: 'harmonica-tabs',
}

/** What the jianpu layout's note heads read as for the preview's notation:
 * the ABC view swaps each degree for its letter name, and the harmonica view
 * for its harmonica tab. */
export function noteLabelsFor(notation: Notation): NoteLabels {
  return NOTE_LABELS[notation]
}
