import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import { AppEntry } from './appEntry.tsx'
import { importOriginMigration } from './originMigration/importOriginMigration.ts'

const root = document.getElementById('root')
if (root == null) {
  throw new Error('missing #root element')
}

// Data handed over from the app's old origin must be saved before the app
// first reads storage (see "Origin migration" in ARCHITECTURE.md).
void importOriginMigration().then(() =>
  createRoot(root).render(
    <StrictMode>
      <AppEntry />
    </StrictMode>,
  ),
)
