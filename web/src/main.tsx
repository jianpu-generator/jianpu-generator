import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './monacoSetup.ts'
import './index.css'
import App from './App.tsx'

const root = document.getElementById('root')
if (root == null) {
  throw new Error('missing #root element')
}

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
