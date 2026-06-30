import type { PagesFunction } from '@cloudflare/workers-types'
import {
  clearCookie,
  OAUTH_COOKIE_NAME,
  parseCookies,
} from '../../_lib/cookies'
import {
  ensureScoresRepository,
  exchangeCodeForToken,
  fetchGitHubUsername,
  SCORES_REPO_NAME,
} from '../../_lib/github'
import { errorRedirect } from '../../_lib/response'
import { writeSessionCookie } from '../../_lib/session'
import type { Env, OAuthPendingData } from '../../_lib/types'

export const onRequestGet: PagesFunction<Env> = async (context) => {
  const requestUrl = new URL(context.request.url)
  const code = requestUrl.searchParams.get('code')
  const state = requestUrl.searchParams.get('state')
  const cookies = parseCookies(context.request.headers.get('Cookie'))
  const pendingRaw = cookies[OAUTH_COOKIE_NAME]

  const clearOAuthCookie = clearCookie(OAUTH_COOKIE_NAME, requestUrl)

  if (!code || !state || !pendingRaw) {
    return errorRedirect(requestUrl, 'missing_oauth_params')
  }

  let pending: OAuthPendingData
  try {
    pending = JSON.parse(pendingRaw) as OAuthPendingData
  } catch {
    return errorRedirect(requestUrl, 'invalid_oauth_state')
  }

  if (pending.state !== state) {
    return errorRedirect(requestUrl, 'state_mismatch')
  }

  const redirectUri = `${requestUrl.origin}/api/github/callback`
  const tokenResult = await exchangeCodeForToken(
    context.env,
    code,
    redirectUri,
    pending.codeVerifier,
  )

  if (!tokenResult.accessToken) {
    return errorRedirect(
      requestUrl,
      tokenResult.error ?? 'token_exchange_failed',
    )
  }

  const username = await fetchGitHubUsername(tokenResult.accessToken)
  if (!username) {
    return errorRedirect(requestUrl, 'user_fetch_failed')
  }

  try {
    await ensureScoresRepository(tokenResult.accessToken, SCORES_REPO_NAME)
  } catch {
    return errorRedirect(requestUrl, 'repo_create_failed')
  }

  const sessionCookieHeader = await writeSessionCookie(
    {
      accessToken: tokenResult.accessToken,
      username,
      repo: SCORES_REPO_NAME,
    },
    requestUrl,
    context.env,
  )

  const headers = new Headers()
  headers.set('Location', `${requestUrl.origin}/`)
  headers.append('Set-Cookie', clearOAuthCookie)
  headers.append('Set-Cookie', sessionCookieHeader)
  return new Response(null, { status: 302, headers })
}
