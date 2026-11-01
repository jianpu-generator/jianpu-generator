import { describe, expect, it } from 'vitest'
import {
  baseNameFromActiveFile,
  midiFilenameFromActiveFile,
  mp3FilenameFromActiveFile,
  pdfFilenameFromActiveFile,
  wavFilenameFromActiveFile,
  zipFilenameFromActiveFile,
} from './workerHelpers'

describe('export filenames', () => {
  it('drop the .jianpu extension', () => {
    expect(baseNameFromActiveFile('song.jianpu')).toBe('song')
    expect(pdfFilenameFromActiveFile('song.jianpu')).toBe('song.pdf')
  })

  it('keep a name that has no .jianpu extension', () => {
    expect(baseNameFromActiveFile('song')).toBe('song')
    expect(midiFilenameFromActiveFile('song')).toBe('song.mid')
  })

  it('drop the folders the file lives in', () => {
    const path = 'hymns/advent/song.jianpu'
    expect(baseNameFromActiveFile(path)).toBe('song')
    expect(pdfFilenameFromActiveFile(path)).toBe('song.pdf')
    expect(midiFilenameFromActiveFile(path)).toBe('song.mid')
    expect(wavFilenameFromActiveFile(path)).toBe('song.wav')
    expect(mp3FilenameFromActiveFile(path)).toBe('song.mp3')
    expect(zipFilenameFromActiveFile(path)).toBe('song.zip')
  })

  it('still mark the enabled parts', () => {
    expect(
      pdfFilenameFromActiveFile('hymns/song.jianpu', ['Soprano', 'Alto']),
    ).toBe('song (Soprano, Alto).pdf')
    expect(zipFilenameFromActiveFile('hymns/song.jianpu', 'split')).toBe(
      'song (split).zip',
    )
  })
})
