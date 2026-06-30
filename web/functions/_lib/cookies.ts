export const OAUTH_COOKIE_NAME = 'jianpu_github_oauth'
export const SESSION_COOKIE_NAME = 'jianpu_github_session'

const OAUTH_MAX_AGE_SECONDS = 600
const SESSION_MAX_AGE_SECONDS = 60 * 60 * 24 * 30

export function parseCookies(
  cookieHeader: string | null,
): Record<string, string> {
  if (!cookieHeader) {
    return {}
  }

  const cookies: Record<string, string> = {}
  for (const part of cookieHeader.split(';')) {
    const trimmed = part.trim()
    if (!trimmed) {
      continue
    }
    const separatorIndex = trimmed.indexOf('=')
    if (separatorIndex === -1) {
      continue
    }
    const name = trimmed.slice(0, separatorIndex)
    const value = trimmed.slice(separatorIndex + 1)
    cookies[name] = decodeURIComponent(value)
  }
  return cookies
}

function cookieAttributes(requestUrl: URL, maxAgeSeconds: number): string {
  const secure = requestUrl.protocol === 'https:' ? '; Secure' : ''
  return `Path=/; HttpOnly; SameSite=Lax; Max-Age=${maxAgeSeconds}${secure}`
}

export function setCookie(
  name: string,
  value: string,
  requestUrl: URL,
  maxAgeSeconds: number,
): string {
  return `${name}=${encodeURIComponent(value)}; ${cookieAttributes(requestUrl, maxAgeSeconds)}`
}

export function clearCookie(name: string, requestUrl: URL): string {
  const secure = requestUrl.protocol === 'https:' ? '; Secure' : ''
  return `${name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0${secure}`
}

export function oauthCookie(value: string, requestUrl: URL): string {
  return setCookie(OAUTH_COOKIE_NAME, value, requestUrl, OAUTH_MAX_AGE_SECONDS)
}

export function sessionCookie(value: string, requestUrl: URL): string {
  return setCookie(
    SESSION_COOKIE_NAME,
    value,
    requestUrl,
    SESSION_MAX_AGE_SECONDS,
  )
}
