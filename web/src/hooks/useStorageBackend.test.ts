import { describe, expect, it } from 'vitest'
import {
  shouldScheduleAutosave,
  shouldWarnBeforeUnload,
} from './useStorageBackend'

describe('shouldScheduleAutosave', () => {
  it('never schedules a save for the local backend', () => {
    expect(
      shouldScheduleAutosave(
        'local',
        { active: 'a.jianpu', content: 'old' },
        { active: 'a.jianpu', content: 'new' },
      ),
    ).toBe(false)
  })

  it('does not schedule on the very first render (no previous snapshot)', () => {
    expect(
      shouldScheduleAutosave('cloud', null, {
        active: 'a.jianpu',
        content: 'new',
      }),
    ).toBe(false)
  })

  it('does not schedule when switching the active file, even though content differs', () => {
    expect(
      shouldScheduleAutosave(
        'cloud',
        { active: 'a.jianpu', content: 'content of a' },
        { active: 'b.jianpu', content: 'content of b' },
      ),
    ).toBe(false)
  })

  it('does not schedule when the same active file has unchanged content', () => {
    expect(
      shouldScheduleAutosave(
        'cloud',
        { active: 'a.jianpu', content: 'same' },
        { active: 'a.jianpu', content: 'same' },
      ),
    ).toBe(false)
  })

  it('schedules a save when the same active file has edited content on the cloud backend', () => {
    expect(
      shouldScheduleAutosave(
        'cloud',
        { active: 'a.jianpu', content: 'old' },
        { active: 'a.jianpu', content: 'new' },
      ),
    ).toBe(true)
  })
})

describe('shouldWarnBeforeUnload', () => {
  it('never warns for the local backend, even mid-save', () => {
    expect(shouldWarnBeforeUnload('local', true, 'saving')).toBe(false)
  })

  it('does not warn on the cloud backend when idle with nothing pending', () => {
    expect(shouldWarnBeforeUnload('cloud', false, 'idle')).toBe(false)
  })

  it('does not warn on the cloud backend once a save has landed', () => {
    expect(shouldWarnBeforeUnload('cloud', false, 'saved')).toBe(false)
  })

  it('warns on the cloud backend while a debounced save is still armed', () => {
    expect(shouldWarnBeforeUnload('cloud', true, 'idle')).toBe(true)
  })

  it('warns on the cloud backend while a save request is in flight', () => {
    expect(shouldWarnBeforeUnload('cloud', false, 'saving')).toBe(true)
  })
})
