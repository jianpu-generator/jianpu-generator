/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_ENABLE_GITHUB_SYNC?: string
  readonly VITE_CLOUDFLARE_APP_URL?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
