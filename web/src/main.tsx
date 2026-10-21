import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import { AppEntry } from './appEntry.tsx'

const root = document.getElementById('root')
if (root == null) {
  throw new Error('missing #root element')
}

createRoot(root).render(
  <StrictMode>
    <AppEntry />
  </StrictMode>,
)
