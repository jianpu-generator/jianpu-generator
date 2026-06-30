import type { PagesFunction } from '@cloudflare/workers-types'
import { oauthCookie } from '../../_lib/cookies'
import { generateCodeChallenge, generateRandomString } from '../../_lib/crypto'
import { readEnv } from '../../_lib/env'
import { errorRedirect, redirectResponse } from '../../_lib/response'
import type { Env, OAuthPendingData } from '../../_lib/types'

export const onRequestGet: PagesFunction<Env> = async (context) => {
  const requestUrl = new URL(context.request.url)
  const clientId = readEnv(context.env, 'GITHUB_CLIENT_ID')
  if (!clientId) {
    return errorRedirect(requestUrl, 'not_configured')
  }

  const state = crypto.randomUUID()
  const codeVerifier = generateRandomString(32)
  const codeChallenge = await generateCodeChallenge(codeVerifier)
  const redirectUri = `${requestUrl.origin}/api/github/callback`

  const pending: OAuthPendingData = { state, codeVerifier }
  const authorizeUrl = new URL('https://github.com/login/oauth/authorize')
  authorizeUrl.searchParams.set('client_id', clientId)
  authorizeUrl.searchParams.set('redirect_uri', redirectUri)
  authorizeUrl.searchParams.set('scope', 'repo')
  authorizeUrl.searchParams.set('state', state)
  authorizeUrl.searchParams.set('code_challenge', codeChallenge)
  authorizeUrl.searchParams.set('code_challenge_method', 'S256')
  authorizeUrl.searchParams.set('allow_signup', 'true')

  return redirectResponse(authorizeUrl.toString(), {
    'Set-Cookie': oauthCookie(JSON.stringify(pending), requestUrl),
  })
}
