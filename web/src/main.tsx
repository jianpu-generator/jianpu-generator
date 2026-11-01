import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import { AppEntry } from './appEntry.tsx'
import { MIGRATE_QUERY_PARAM } from './originMigration/originMigrationPayload.ts'

const root = document.getElementById('root')
if (root == null) {
  throw new Error('missing #root element')
}

function render(container: HTMLElement) {
  createRoot(container).render(
    <StrictMode>
      <AppEntry />
    </StrictMode>,
  )
}

// Data handed over from the app's old origin must be saved before the app
// first reads storage (see "Origin migration" in ARCHITECTURE.md). Every
// other visit renders synchronously, without loading the import's modules:
// deferring the first render on every visit broke startup across the e2e
// suite (soundfont readiness, editor load).
if (new URLSearchParams(window.location.search).has(MIGRATE_QUERY_PARAM)) {
  void import('./originMigration/importOriginMigration.ts')
    .then(({ importOriginMigration }) => importOriginMigration())
    .then(() => render(root))
} else {
  render(root)
}
