import { describe, expect, it } from 'vitest'
import type { FileStoreState } from '../../fileStore'
import { withServerContent } from './withServerContent'

const store: FileStoreState = {
  active: 'a.jianpu',
  userFiles: { 'a.jianpu': 'mine' },
  bin: {},
  fileIds: { 'a.jianpu': 'id-a' },
}

describe('withServerContent', () => {
  it('replaces the text of the file with that id', () => {
    expect(withServerContent(store, 'id-a', 'theirs').userFiles).toEqual({
      'a.jianpu': 'theirs',
    })
  })

  it('leaves the store alone for an unknown id', () => {
    expect(withServerContent(store, 'id-z', 'theirs')).toBe(store)
  })
})
