import { describe, expect, it } from 'vitest'
import { resolveRole } from '../src/resolveRole'

describe('resolveRole', () => {
  it('treats a connection with no token as a viewer', () => {
    expect(resolveRole(null, null)).toBe('viewer')
    expect(resolveRole('existing-token', null)).toBe('viewer')
  })

  it('lets the first token-bearing connection claim ownership', () => {
    expect(resolveRole(null, 'fresh-token')).toBe('owner')
  })

  it('treats a reconnect with the same token as the owner', () => {
    expect(resolveRole('owner-token', 'owner-token')).toBe('owner')
  })

  it('treats a mismatched token as a viewer', () => {
    expect(resolveRole('owner-token', 'someone-elses-token')).toBe('viewer')
  })
})
