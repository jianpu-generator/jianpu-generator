import { describe, expect, it } from 'vitest'
import { buildSyncedShareFailure, redactSecrets } from './errors'
import {
  type ApiError,
  NetworkFailure,
  WorkerRequestError,
} from './workerClient'

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

function workerError(
  status: number,
  statusText: string,
  body: ApiError | string,
): WorkerRequestError {
  return new WorkerRequestError(
    new Response(null, { status, statusText }),
    body,
  )
}

describe('buildSyncedShareFailure: worker failure responses', () => {
  it('renders the worker unauthorized ApiError verbosely', () => {
    const failure = buildSyncedShareFailure(
      'start',
      workerError(401, 'Unauthorized', {
        code: 'unauthorized',
        reason: 'GitHub verification failed: 401 Unauthorized',
        failedAt: 1_700_000_000_000,
        attempts: 3,
      }),
    )
    expect(failure).toEqual({
      operation: 'start',
      reason: 'GitHub verification failed: 401 Unauthorized',
      failedAt: 1_700_000_000_000,
      attempts: 3,
      httpStatus: 401,
      httpStatusText: 'Unauthorized',
      authRejected: true,
    })
  })

  it('redacts a token-shaped value even inside an unauthorized ApiError', () => {
    const failure = buildSyncedShareFailure(
      'start',
      workerError(401, 'Unauthorized', {
        code: 'unauthorized',
        reason:
          'unexpected token gho_abcdefghijklmnopqrstuvwxyz012345 in response',
        failedAt: 1,
        attempts: 1,
      }),
    )
    expect(failure.reason).not.toContain('gho_')
    expect(failure.reason).toContain('[redacted]')
  })

  it('falls back to raw status/body (redacted) for any other failure', () => {
    const failure = buildSyncedShareFailure(
      'stop',
      workerError(
        500,
        'Internal Server Error',
        `internal error, hash=${'b'.repeat(40)}`,
      ),
    )
    expect(failure.operation).toBe('stop')
    expect(failure.httpStatus).toBe(500)
    expect(failure.httpStatusText).toBe('Internal Server Error')
    expect(failure.attempts).toBeUndefined()
    expect(failure.authRejected).toBe(false)
    expect(failure.rawResponseBody).toContain('[redacted]')
    expect(failure.rawResponseBody).not.toContain('b'.repeat(40))
  })
})

describe('buildSyncedShareFailure: network failures', () => {
  it('describes a network failure', () => {
    const failure = buildSyncedShareFailure(
      'start',
      new NetworkFailure('Failed to fetch'),
    )
    expect(failure.operation).toBe('start')
    expect(failure.reason).toContain('Failed to fetch')
    expect(failure.attempts).toBeUndefined()
    expect(failure.authRejected).toBe(false)
  })

  it('redacts a token-shaped value from a network failure message', () => {
    const failure = buildSyncedShareFailure(
      'stop',
      new NetworkFailure(
        'leaked gho_abcdefghijklmnopqrstuvwxyz012345 in error',
      ),
    )
    expect(failure.reason).not.toContain('gho_')
  })
})
