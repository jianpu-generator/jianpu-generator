export interface Env {
  GITHUB_CLIENT_ID: string
  GITHUB_CLIENT_SECRET: string
  SESSION_SECRET: string
}

export interface SessionData {
  accessToken: string
  username: string
  repo: string
}

export interface OAuthPendingData {
  state: string
  codeVerifier: string
}

export interface SessionResponse {
  connected: boolean
  username?: string
  repo?: string
}
