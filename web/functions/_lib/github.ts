import { readEnv } from './env'
import type { Env } from './types'

export const GITHUB_USER_AGENT = 'jianpu-generator'
export const SCORES_REPO_NAME = 'jianpu-scores'

interface TokenExchangeResult {
  accessToken?: string
  error?: string
}

interface GitHubUser {
  login: string
}

export async function exchangeCodeForToken(
  env: Env,
  code: string,
  redirectUri: string,
  codeVerifier: string,
): Promise<TokenExchangeResult> {
  const clientId = readEnv(env, 'GITHUB_CLIENT_ID')
  const clientSecret = readEnv(env, 'GITHUB_CLIENT_SECRET')
  if (!clientId || !clientSecret) {
    return { error: 'not_configured' }
  }

  const response = await fetch('https://github.com/login/oauth/access_token', {
    method: 'POST',
    headers: {
      Accept: 'application/json',
      'Content-Type': 'application/json',
      'User-Agent': GITHUB_USER_AGENT,
    },
    body: JSON.stringify({
      client_id: clientId,
      client_secret: clientSecret,
      code,
      redirect_uri: redirectUri,
      code_verifier: codeVerifier,
    }),
  })

  const data = (await response.json()) as {
    access_token?: string
    error?: string
    error_description?: string
  }

  if (!response.ok || data.error || !data.access_token) {
    return {
      error: data.error ?? data.error_description ?? 'token_exchange_failed',
    }
  }

  return { accessToken: data.access_token }
}

export async function fetchGitHubUsername(
  accessToken: string,
): Promise<string | null> {
  const response = await fetch('https://api.github.com/user', {
    headers: {
      Accept: 'application/vnd.github+json',
      Authorization: `Bearer ${accessToken}`,
      'User-Agent': GITHUB_USER_AGENT,
    },
  })

  if (!response.ok) {
    return null
  }

  const user = (await response.json()) as GitHubUser
  return user.login ?? null
}

export async function ensureScoresRepository(
  accessToken: string,
  repoName: string,
): Promise<void> {
  const response = await fetch('https://api.github.com/user/repos', {
    method: 'POST',
    headers: {
      Accept: 'application/vnd.github+json',
      Authorization: `Bearer ${accessToken}`,
      'Content-Type': 'application/json',
      'User-Agent': GITHUB_USER_AGENT,
    },
    body: JSON.stringify({
      name: repoName,
      private: true,
      auto_init: false,
    }),
  })

  if (response.ok || response.status === 422) {
    return
  }

  const body = await response.text()
  throw new Error(`Failed to ensure repository: ${response.status} ${body}`)
}
