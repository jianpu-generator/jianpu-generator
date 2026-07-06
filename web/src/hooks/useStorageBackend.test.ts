import { describe, expect, it } from 'vitest'
import { shouldScheduleAutosave } from './useStorageBackend'

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
      shouldScheduleAutosave('github', null, {
        active: 'a.jianpu',
        content: 'new',
      }),
    ).toBe(false)
  })

  it('does not schedule when switching the active file, even though content differs', () => {
    expect(
      shouldScheduleAutosave(
        'github',
        { active: 'a.jianpu', content: 'content of a' },
        { active: 'b.jianpu', content: 'content of b' },
      ),
    ).toBe(false)
  })

  it('does not schedule when the same active file has unchanged content', () => {
    expect(
      shouldScheduleAutosave(
        'github',
        { active: 'a.jianpu', content: 'same' },
        { active: 'a.jianpu', content: 'same' },
      ),
    ).toBe(false)
  })

  it('schedules a save when the same active file has edited content on GitHub', () => {
    expect(
      shouldScheduleAutosave(
        'github',
        { active: 'a.jianpu', content: 'old' },
        { active: 'a.jianpu', content: 'new' },
      ),
    ).toBe(true)
  })
})
