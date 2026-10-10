import { describe, expect, it, vi } from 'vitest'
import {
  checkFocusRevision,
  type FocusRevisionDependencies,
} from './useFocusRevisionCheck'

function makeDependencies(
  overrides: Partial<FocusRevisionDependencies> = {},
): FocusRevisionDependencies {
  return {
    fetchServerFile: vi.fn(async () => ({ revision: 5, content: 'server' })),
    recordedRevision: () => 3n,
    hasPendingMessages: () => false,
    hasUnsavedEdits: () => false,
    applyServerFile: vi.fn(async () => undefined),
    ...overrides,
  }
}

describe('checkFocusRevision', () => {
  it('loads the server file when ahead and idle', async () => {
    const dependencies = makeDependencies()
    await checkFocusRevision(dependencies, 'f')
    expect(dependencies.applyServerFile).toHaveBeenCalledWith('f', 5, 'server')
  })

  it('does nothing while typed text is still waiting to be saved', async () => {
    const dependencies = makeDependencies({ hasUnsavedEdits: () => true })
    await checkFocusRevision(dependencies, 'f')
    expect(dependencies.applyServerFile).not.toHaveBeenCalled()
  })

  it('does nothing when messages are pending', async () => {
    const dependencies = makeDependencies({ hasPendingMessages: () => true })
    await checkFocusRevision(dependencies, 'f')
    expect(dependencies.applyServerFile).not.toHaveBeenCalled()
  })

  it('does nothing when the server is not ahead', async () => {
    const dependencies = makeDependencies({ recordedRevision: () => 5n })
    await checkFocusRevision(dependencies, 'f')
    expect(dependencies.applyServerFile).not.toHaveBeenCalled()
  })

  it('does nothing when the file is missing on the server', async () => {
    const dependencies = makeDependencies({
      fetchServerFile: vi.fn(async () => undefined),
    })
    await checkFocusRevision(dependencies, 'f')
    expect(dependencies.applyServerFile).not.toHaveBeenCalled()
  })
})
