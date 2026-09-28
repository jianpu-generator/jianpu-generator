import './monacoSetup.ts'
import App from './App.tsx'
import { injectFontFaces } from './injectFontFaces.ts'
import { injectPreviewInteractionStyles } from './injectPreviewInteractionStyles.ts'

// The editor app's entry, loaded by `main.tsx` for every route except the
// Synced Share GitHub sign-in callback. Its one-time document setup runs at
// import time, before `main.tsx` renders `<AppEntry/>`, so there's no extra
// FOUC window versus doing it up front in `main.tsx`.
injectFontFaces()
injectPreviewInteractionStyles()

export function AppEntry() {
  return <App />
}
