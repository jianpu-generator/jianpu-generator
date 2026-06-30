const DEFAULT_CLOUDFLARE_APP_URL = 'https://jianpu-generator.pages.dev'

export const enableGitHubSync =
  import.meta.env.VITE_ENABLE_GITHUB_SYNC === 'true'

export const cloudflareAppUrl =
  import.meta.env.VITE_CLOUDFLARE_APP_URL ?? DEFAULT_CLOUDFLARE_APP_URL
