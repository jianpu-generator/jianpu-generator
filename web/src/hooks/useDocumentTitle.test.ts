import { describe, expect, it } from 'vitest'
import { documentTitleFor } from './useDocumentTitle'

describe('documentTitleFor', () => {
  it('strips the .jianpu extension from the active file name', () => {
    expect(documentTitleFor('twinkle.jianpu')).toBe('twinkle · 簡譜')
  })

  it('leaves a name without the .jianpu extension untouched', () => {
    expect(documentTitleFor('untitled')).toBe('untitled · 簡譜')
  })
})
