import { useState } from 'react'
import { CheatsheetDialog } from './Cheatsheet'
import './AppHeader.css'

export function AppHeader() {
  const [cheatsheetOpen, setCheatsheetOpen] = useState(false)

  return (
    <>
      <header className="app-header">
        <h1>簡譜</h1>
        <span className="app-subtitle">live preview</span>
        <div className="app-header__spacer" />
        <button
          type="button"
          className="app-header__cheatsheet-btn"
          onClick={() => setCheatsheetOpen(true)}
          aria-label="Open syntax cheatsheet"
        >
          ?
        </button>
      </header>
      <CheatsheetDialog
        open={cheatsheetOpen}
        onOpenChange={setCheatsheetOpen}
      />
    </>
  )
}
