import { describe, expect, it } from 'vitest'
import { HttpStatusError, NetworkFailure } from '../storage/cloudBackendHttp'
import { cloudLoadRetryDelayMs, isAuthRejection } from './useCloudStoreLoader'

describe('cloudLoadRetryDelayMs', () => {
  it('backs off exponentially from one second', () => {
    expect([0, 1, 2, 3].map(cloudLoadRetryDelayMs)).toEqual([
      1_000, 2_000, 4_000, 8_000,
    ])
  })

  it('caps the delay at thirty seconds', () => {
    expect(cloudLoadRetryDelayMs(10)).toBe(30_000)
  })
})

describe('isAuthRejection', () => {
  it('treats a 401 as a rejected token', () => {
    expect(isAuthRejection(new HttpStatusError(401, null))).toBe(true)
  })

  it('treats server errors and network failures as transient', () => {
    expect(isAuthRejection(new HttpStatusError(500, null))).toBe(false)
    expect(isAuthRejection(new NetworkFailure('fetch failed'))).toBe(false)
  })
})
