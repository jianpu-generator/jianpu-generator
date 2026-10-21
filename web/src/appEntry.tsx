import './monacoSetup.ts'
import App from './App.tsx'
import { injectFontFaces } from './injectFontFaces.ts'
import { injectPreviewInteractionStyles } from './injectPreviewInteractionStyles.ts'

// The editor app's entry, rendered by `main.tsx`. Its one-time document
// setup runs at import time, before `main.tsx` renders `<AppEntry/>`, so
// there's no FOUC window.
injectFontFaces()
injectPreviewInteractionStyles()

export function AppEntry() {
  return <App />
}
