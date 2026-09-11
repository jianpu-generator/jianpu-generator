import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './monacoSetup.ts'
import './index.css'
import App from './App.tsx'
import { SyncedShareGithubCallbackPage } from './components/SyncedShareGithubCallbackPage.tsx'
import { injectFontFaces } from './injectFontFaces.ts'
import { syncedShareGithubCallbackPathname } from './storage/syncedShareGithubAuth.ts'

injectFontFaces()

const root = document.getElementById('root')
if (root == null) {
  throw new Error('missing #root element')
}

// The Synced Share GitHub sign-in popup redirects here once GitHub approves
// or denies the request (see `syncedShareGithubAuth.ts`) -- this path is
// never the main app, so it short-circuits straight to the completion page
// instead of mounting `<App/>`. Compared against the base-path-aware
// pathname (not the bare `SYNCED_SHARE_GITHUB_REDIRECT_PATH`), since this
// app is served under a `/jianpu-generator/` subpath on GitHub Pages.
const isSyncedShareGithubCallback =
  window.location.pathname === syncedShareGithubCallbackPathname()

createRoot(root).render(
  <StrictMode>
    {isSyncedShareGithubCallback ? <SyncedShareGithubCallbackPage /> : <App />}
  </StrictMode>,
)
