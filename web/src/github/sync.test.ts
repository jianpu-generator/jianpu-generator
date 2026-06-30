import { describe, expect, it } from 'vitest'
import { DEMO_FILE_NAME, type FileStoreState } from '../fileStore'
import { buildSyncPlan, syncPlanIsEmpty } from './sync'

const sampleState: FileStoreState = {
  active: 'song.jianpu',
  userFiles: {
    'song.jianpu': 'content-a',
    'other.jianpu': 'content-b',
  },
  bin: {
    'deleted.jianpu': 'content-c',
  },
  fileIds: {
    'song.jianpu': 'id-song',
    'other.jianpu': 'id-other',
    'deleted.jianpu': 'id-deleted',
  },
}

describe('buildSyncPlan', () => {
  it('returns empty plan when nothing changed', () => {
    const plan = buildSyncPlan(sampleState, sampleState)
    expect(syncPlanIsEmpty(plan)).toBe(true)
  })

  it('includes PUT for changed score content', () => {
    const current = {
      ...sampleState,
      userFiles: {
        ...sampleState.userFiles,
        'song.jianpu': 'content-a-edited',
      },
    }
    const plan = buildSyncPlan(sampleState, current)

    expect(plan.filePuts).toEqual([
      { path: 'scores/song.jianpu', content: 'content-a-edited' },
    ])
    expect(plan.manifest).toBeNull()
  })

  it('includes manifest patch when active file changes', () => {
    const current = {
      ...sampleState,
      active: 'other.jianpu',
    }
    const plan = buildSyncPlan(sampleState, current)

    expect(plan.filePuts).toEqual([])
    expect(plan.manifest).toEqual({
      active: 'other.jianpu',
      fileIds: {
        'song.jianpu': 'id-song',
        'other.jianpu': 'id-other',
        'deleted.jianpu': 'id-deleted',
      },
      bin: ['deleted.jianpu'],
    })
  })

  it('ignores demo file content', () => {
    const baseline: FileStoreState = {
      active: DEMO_FILE_NAME,
      userFiles: {},
      bin: {},
      fileIds: {},
    }
    const current: FileStoreState = {
      active: 'new.jianpu',
      userFiles: { 'new.jianpu': 'hello' },
      bin: {},
      fileIds: { 'new.jianpu': 'id-new' },
    }

    const plan = buildSyncPlan(baseline, current)

    expect(plan.filePuts).toEqual([
      { path: 'scores/new.jianpu', content: 'hello' },
    ])
    expect(plan.manifest?.active).toBe('new.jianpu')
  })
})
