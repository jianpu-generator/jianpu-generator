import { describe, expect, it } from 'vitest'
import { DEMO_FILE_NAME, type FileStoreState, renameFile } from '../fileStore'
import {
  binPath,
  fileStoreToManifest,
  manifestAndFilesToFileStore,
  scorePath,
} from './manifest'

const sampleState: FileStoreState = {
  active: 'song.jianpu',
  userFiles: {
    'song.jianpu': '# notes\n1 2 3',
    'other.jianpu': '# notes\n4 5 6',
  },
  bin: {
    'deleted.jianpu': '# notes\n0',
  },
  fileIds: {
    'song.jianpu': 'id-song',
    'other.jianpu': 'id-other',
    'deleted.jianpu': 'id-deleted',
  },
}

function roundTrip(state: FileStoreState): FileStoreState {
  const manifest = fileStoreToManifest(state)
  const scoreFiles = Object.fromEntries(
    Object.entries(state.userFiles).filter(([name]) => name !== DEMO_FILE_NAME),
  )
  const binFiles = Object.fromEntries(
    Object.entries(state.bin).filter(([name]) => name !== DEMO_FILE_NAME),
  )
  return manifestAndFilesToFileStore(manifest, scoreFiles, binFiles)
}

describe('scorePath and binPath', () => {
  it('maps file names to repo paths', () => {
    expect(scorePath('song.jianpu')).toBe('scores/song.jianpu')
    expect(binPath('deleted.jianpu')).toBe('bin/deleted.jianpu')
  })
})

describe('fileStoreToManifest', () => {
  it('excludes the demo file from manifest metadata', () => {
    const state: FileStoreState = {
      active: DEMO_FILE_NAME,
      userFiles: { 'mine.jianpu': 'content' },
      bin: {},
      fileIds: { 'mine.jianpu': 'id-mine' },
    }

    const manifest = fileStoreToManifest(state)

    expect(manifest).toEqual({
      active: 'mine.jianpu',
      fileIds: { 'mine.jianpu': 'id-mine' },
      bin: [],
    })
    expect(manifest.fileIds[DEMO_FILE_NAME]).toBeUndefined()
  })

  it('lists binned file names without content', () => {
    const manifest = fileStoreToManifest(sampleState)

    expect(manifest.bin).toEqual(['deleted.jianpu'])
    expect(manifest.fileIds['deleted.jianpu']).toBe('id-deleted')
  })

  it('preserves the active user file', () => {
    const manifest = fileStoreToManifest(sampleState)

    expect(manifest.active).toBe('song.jianpu')
  })
})

describe('manifest round-trip', () => {
  it('restores user files, bin, fileIds, and active file', () => {
    expect(roundTrip(sampleState)).toEqual(sampleState)
  })

  it('preserves stable fileIds across a rename', () => {
    const renamed = renameFile(sampleState, 'song.jianpu', 'renamed.jianpu')
    const manifest = fileStoreToManifest(renamed)

    expect(manifest.fileIds['renamed.jianpu']).toBe('id-song')
    expect(manifest.fileIds['song.jianpu']).toBeUndefined()
    expect(manifest.active).toBe('renamed.jianpu')

    const scoreFiles = {
      'renamed.jianpu': '# notes\n1 2 3',
      'other.jianpu': '# notes\n4 5 6',
    }
    const restored = manifestAndFilesToFileStore(manifest, scoreFiles, {
      'deleted.jianpu': '# notes\n0',
    })

    expect(restored.fileIds['renamed.jianpu']).toBe('id-song')
    expect(restored.active).toBe('renamed.jianpu')
  })

  it('falls back to the demo file when the repo has no user scores', () => {
    const restored = manifestAndFilesToFileStore(
      { active: '', fileIds: {}, bin: [] },
      {},
      {},
    )

    expect(restored).toEqual({
      active: DEMO_FILE_NAME,
      userFiles: {},
      bin: {},
      fileIds: {},
    })
  })
})
