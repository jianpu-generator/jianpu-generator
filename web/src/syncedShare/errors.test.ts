import { describe, expect, it } from 'vitest'
import {
  buildSyncedShareNetworkFailure,
  buildSyncedShareResponseFailure,
  redactSecrets,
} from './errors'

describe('redactSecrets', () => {
  it('redacts recognizable GitHub token prefixes', () => {
    expect(redactSecrets('token=gho_abcdefghijklmnopqrstuvwxyz012345')).toBe(
      'token=[redacted]',
    )
    expect(
      redactSecrets('pat github_pat_11ABCDEFG0abcdefghijklmnopqrstuvwxyz'),
    ).toBe('pat [redacted]')
  })

  it('redacts long hex runs that look like a token hash', () => {
    const hash = 'a'.repeat(64)
    expect(redactSecrets(`hash=${hash}`)).toBe('hash=[redacted]')
  })

  it('leaves ordinary text untouched', () => {
    expect(redactSecrets('verification failed: timeout after 3 attempts')).toBe(
      'verification failed: timeout after 3 attempts',
    )
  })
})

describe('buildSyncedShareResponseFailure', () => {
  it('parses the worker VerificationFailure (401) body verbosely', async () => {
    const body = JSON.stringify({
      reason: 'GitHub verification failed: 401 Unauthorized',
      failedAt: 1_700_000_000_000,
      attempts: 3,
    })
    const response = new Response(body, {
      status: 401,
      statusText: 'Unauthorized',
    })
    const failure = await buildSyncedShareResponseFailure('update', response)
    expect(failure).toEqual({
      operation: 'update',
      reason: 'GitHub verification failed: 401 Unauthorized',
      failedAt: 1_700_000_000_000,
      attempts: 3,
      httpStatus: 401,
      httpStatusText: 'Unauthorized',
    })
  })

  it('redacts a token-shaped value even inside a recognized VerificationFailure body', async () => {
    const body = JSON.stringify({
      reason:
        'unexpected token gho_abcdefghijklmnopqrstuvwxyz012345 in response',
      failedAt: 1,
      attempts: 1,
    })
    const response = new Response(body, { status: 401 })
    const failure = await buildSyncedShareResponseFailure('update', response)
    expect(failure.reason).not.toContain('gho_')
    expect(failure.reason).toContain('[redacted]')
  })

  it('falls back to raw status/body (redacted) for an unrecognized failure shape', async () => {
    const response = new Response('internal error, hash=' + 'b'.repeat(40), {
      status: 500,
      statusText: 'Internal Server Error',
    })
    const failure = await buildSyncedShareResponseFailure('stop', response)
    expect(failure.operation).toBe('stop')
    expect(failure.httpStatus).toBe(500)
    expect(failure.httpStatusText).toBe('Internal Server Error')
    expect(failure.attempts).toBeUndefined()
    expect(failure.rawResponseBody).toContain('[redacted]')
    expect(failure.rawResponseBody).not.toContain('b'.repeat(40))
  })
})

describe('buildSyncedShareNetworkFailure', () => {
  it('describes a thrown Error', () => {
    const failure = buildSyncedShareNetworkFailure(
      'update',
      new TypeError('Failed to fetch'),
    )
    expect(failure.operation).toBe('update')
    expect(failure.reason).toContain('Failed to fetch')
    expect(failure.attempts).toBeUndefined()
  })

  it('redacts a token-shaped value from a thrown error message', () => {
    const failure = buildSyncedShareNetworkFailure(
      'stop',
      new Error('leaked gho_abcdefghijklmnopqrstuvwxyz012345 in error'),
    )
    expect(failure.reason).not.toContain('gho_')
  })
})
