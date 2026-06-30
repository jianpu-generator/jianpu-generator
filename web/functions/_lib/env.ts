import type { Env } from './types'

export function readEnv(env: Env, name: keyof Env): string | undefined {
  const value = env[name]
  if (value) {
    return value
  }

  for (const key of Object.keys(env)) {
    if (key.trim() === name && env[key as keyof Env]) {
      return env[key as keyof Env]
    }
  }

  return undefined
}

export function requireEnv(env: Env, name: keyof Env): string {
  const value = readEnv(env, name)
  if (!value) {
    throw new Error(`Missing required environment variable: ${name}`)
  }
  return value
}
