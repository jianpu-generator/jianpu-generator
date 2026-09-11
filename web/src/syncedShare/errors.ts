// Structured failure info for a Synced Share write (owner side), surfaced by
// `useSyncedShareOwner.ts` and rendered by `SyncedShareErrorDialog.tsx`.
// Implements task 9 in TODO-synced-share-rust-d1-migration.md: errors here
// are meant to be maximally verbose (per §0) EXCEPT for one hard exception --
// the GitHub OAuth token and its stored hash must never appear. The worker's
// `401` `VerificationFailure` body (`crates/live-share-worker/src/verification.rs`)
// has no field for either by construction, so the common case is already
// safe by the time it reaches here; `redactSecrets` below is a defensive
// backstop for any other failure shape (a raw non-2xx body, a thrown network
// error) that might somehow echo one back.

export type SyncedShareOperation = 'create' | 'update' | 'stop'

export interface SyncedShareFailure {
  operation: SyncedShareOperation
  reason: string
  /** Milliseconds since the epoch. From the worker's `VerificationFailure`
   * when available, otherwise the time this client observed the failure. */
  failedAt: number
  /** Present only for a parsed `VerificationFailure` (a `401`). */
  attempts?: number
  httpStatus?: number
  httpStatusText?: string
  /** Redacted (see `redactSecrets`) raw response body, kept only for
   * failure shapes this client doesn't specifically recognize. */
  rawResponseBody?: string
}

/** Mirrors the Rust worker's `verification::VerificationFailure` JSON body
 * (reason/failedAt/attempts, `#[serde(rename_all = "camelCase")]`) --
 * returned as a `401` when GitHub verification fails after retries. That
 * struct has no token/hash field by construction (see its doc comment), so
 * a value of this shape is safe to render in full. */
interface SyncedShareVerificationFailureBody {
  reason: string
  failedAt: number
  attempts: number
}

const REDACTED = '[redacted]'

/** GitHub token prefixes (fine-grained and classic PATs, OAuth/installation
 * tokens) recognizable regardless of context. */
const GITHUB_TOKEN_PATTERN =
  /\b(?:gh[oprsu]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,})\b/g

/** A long run of hex characters is what a SHA-256 token hash (64 hex chars,
 * per `identity.rs`'s hashing of the Synced Share token) looks like -- redact
 * generically even without knowing the exact value in scope. */
const HEX_HASH_PATTERN = /\b[0-9a-f]{32,}\b/gi

/**
 * Defensively redacts anything that looks like a GitHub token or a
 * token hash from `text` before it is ever rendered in the error dialog.
 * This is a backstop, not the primary guarantee -- the primary guarantee is
 * that `VerificationFailure` (the worker's actual failure payload) has no
 * such field to begin with.
 */
export function redactSecrets(text: string): string {
  return text
    .replace(GITHUB_TOKEN_PATTERN, REDACTED)
    .replace(HEX_HASH_PATTERN, REDACTED)
}

function isVerificationFailureBody(
  value: unknown,
): value is SyncedShareVerificationFailureBody {
  if (value === null || typeof value !== 'object') return false
  const record = value as Record<string, unknown>
  return (
    typeof record.reason === 'string' &&
    typeof record.failedAt === 'number' &&
    typeof record.attempts === 'number'
  )
}

async function safeReadText(response: Response): Promise<string> {
  try {
    return await response.text()
  } catch {
    return ''
  }
}

/**
 * Builds structured, dialog-ready failure info from a non-ok `Response` to a
 * Synced Share write. Recognizes the worker's `401` `VerificationFailure`
 * body; falls back to the raw status/body (redacted) for any other failure
 * shape, so the dialog stays useful even for a status this client doesn't
 * specifically know about (per §0's "maximally verbose" decision).
 */
export async function buildSyncedShareResponseFailure(
  operation: SyncedShareOperation,
  response: Response,
): Promise<SyncedShareFailure> {
  const rawBody = await safeReadText(response)
  let parsed: unknown = null
  try {
    parsed = rawBody ? JSON.parse(rawBody) : null
  } catch {
    parsed = null
  }
  if (isVerificationFailureBody(parsed)) {
    return {
      operation,
      reason: redactSecrets(parsed.reason),
      failedAt: parsed.failedAt,
      attempts: parsed.attempts,
      httpStatus: response.status,
      httpStatusText: response.statusText,
    }
  }
  return {
    operation,
    reason: `Synced Share "${operation}" request failed`,
    failedAt: Date.now(),
    httpStatus: response.status,
    httpStatusText: response.statusText,
    rawResponseBody: redactSecrets(rawBody),
  }
}

/** Builds failure info for a network-level error (fetch rejecting: offline,
 * DNS, CORS, etc.) rather than an HTTP error response. */
export function buildSyncedShareNetworkFailure(
  operation: SyncedShareOperation,
  error: unknown,
): SyncedShareFailure {
  const message = error instanceof Error ? error.message : String(error)
  return {
    operation,
    reason: redactSecrets(`Network error: ${message}`),
    failedAt: Date.now(),
  }
}
