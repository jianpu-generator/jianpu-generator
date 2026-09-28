import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import { syncedShareGithubCallbackPathname } from './storage/accountAuthPopup.ts'

const root = document.getElementById('root')
if (root == null) {
  throw new Error('missing #root element')
}

// The Synced Share GitHub sign-in popup redirects here once GitHub approves
// or denies the request (see `accountAuthPopup.ts`) -- this path is
// never the main app, so it short-circuits straight to the completion page
// instead of mounting `<App/>`. Compared against the base-path-aware
// pathname (not the bare `SYNCED_SHARE_GITHUB_REDIRECT_PATH`), since this
// app is served under a `/jianpu-generator/` subpath on GitHub Pages.
const isSyncedShareGithubCallback =
  window.location.pathname === syncedShareGithubCallbackPathname()

// Each route's tree is imported dynamically so the popup only fetches the
// handful of modules it needs to finish the token exchange, not the whole
// editor (Monaco, the wasm bindings, every `<App/>` module). The popup
// shares its opener's per-host connection pool, which the opener is still
// using to download its own modules and assets -- so the more the popup
// fetches, the longer the opener's "Signing in…" button waits.
const page = isSyncedShareGithubCallback
  ? import('./components/SyncedShareGithubCallbackPage.tsx').then(
      ({ SyncedShareGithubCallbackPage }) => <SyncedShareGithubCallbackPage />,
    )
  : import('./appEntry.tsx').then(({ AppEntry }) => <AppEntry />)

void page.then((element) => {
  createRoot(root).render(<StrictMode>{element}</StrictMode>)
})
