import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './monacoSetup.ts'
import './index.css'
import App from './App.tsx'
import { SyncedShareGithubCallbackPage } from './components/SyncedShareGithubCallbackPage.tsx'
import { injectFontFaces } from './injectFontFaces.ts'
import { SYNCED_SHARE_GITHUB_REDIRECT_PATH } from './storage/syncedShareGithubAuth.ts'

injectFontFaces()

const root = document.getElementById('root')
if (root == null) {
  throw new Error('missing #root element')
}

// The Synced Share GitHub sign-in popup redirects here once GitHub approves
// or denies the request (see `syncedShareGithubAuth.ts`) -- this path is
// never the main app, so it short-circuits straight to the completion page
// instead of mounting `<App/>`.
const isSyncedShareGithubCallback =
  window.location.pathname === SYNCED_SHARE_GITHUB_REDIRECT_PATH

createRoot(root).render(
  <StrictMode>
    {isSyncedShareGithubCallback ? <SyncedShareGithubCallbackPage /> : <App />}
  </StrictMode>,
)
