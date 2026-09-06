import { ChevronDownIcon, Link2Icon, UpdateIcon } from '@radix-ui/react-icons'
import * as Toast from '@radix-ui/react-toast'
import { useCallback, useState } from 'react'
import { ResponsiveMenu } from './ResponsiveMenu'

interface SyncedShareButtonProps {
  isSynced: boolean
  syncedShareLink: string | null
  onStartSync: () => string
  onStopSync: () => void
  className?: string
}

export function SyncedShareButton({
  isSynced,
  syncedShareLink,
  onStartSync,
  onStopSync,
  className = 'preview-export-btn',
}: SyncedShareButtonProps) {
  const [toastOpen, setToastOpen] = useState(false)
  const [menuOpen, setMenuOpen] = useState(false)

  const copyUrl = useCallback(async (url: string) => {
    try {
      await navigator.clipboard.writeText(url)
      setToastOpen(true)
    } catch {
      window.prompt('Copy this link to share:', url)
    }
  }, [])

  return (
    <div className="export-menu">
      {isSynced ? (
        // Radix's DropdownMenuTrigger opens the menu on `pointerdown` and
        // `preventDefault()`s it whenever the menu is currently closed —
        // which, per spec, suppresses the `click` event that would
        // otherwise follow. So a Trigger can only ever toggle its own
        // menu; it can't also carry a "start sync" click handler for the
        // not-synced state below. Rendering the plain "Sync" button
        // outside any Trigger, and only wrapping this "Synced" button in
        // one once there's a menu for it to open, sidesteps that entirely.
        <ResponsiveMenu.Root open={menuOpen} onOpenChange={setMenuOpen}>
          <ResponsiveMenu.Trigger asChild>
            <button
              type="button"
              className={className}
              data-testid="synced-share-button"
              aria-label="Synced share options"
            >
              <UpdateIcon aria-hidden="true" />
              Synced
              <ChevronDownIcon
                className="export-menu-caret"
                aria-hidden="true"
              />
            </button>
          </ResponsiveMenu.Trigger>
          <ResponsiveMenu.Content
            className="export-menu-list"
            align="end"
            sideOffset={4}
            title="Synced share options"
          >
            <ResponsiveMenu.Item
              className="export-menu-item"
              data-testid="copy-synced-share-link-button"
              onSelect={() => {
                if (syncedShareLink) void copyUrl(syncedShareLink)
              }}
            >
              <Link2Icon aria-hidden="true" />
              Copy Synced Link
            </ResponsiveMenu.Item>
            <ResponsiveMenu.Item
              className="export-menu-item"
              data-testid="stop-sync-button"
              onSelect={onStopSync}
            >
              <svg
                width="15"
                height="15"
                viewBox="0 0 15 15"
                fill="currentColor"
                aria-hidden="true"
              >
                <rect x="2" y="2" width="11" height="11" rx="1.5" />
              </svg>
              Stop Sync
            </ResponsiveMenu.Item>
          </ResponsiveMenu.Content>
        </ResponsiveMenu.Root>
      ) : (
        <button
          type="button"
          className={className}
          data-testid="synced-share-button"
          aria-label="Sync"
          title="Anyone with this link can view your latest saved score after reloading. Don't share it publicly."
          onClick={() => void copyUrl(onStartSync())}
        >
          <UpdateIcon aria-hidden="true" />
          Sync
        </button>
      )}
      <Toast.Provider swipeDirection="right" duration={3000}>
        <Toast.Root
          className="export-audio-toast"
          data-testid="synced-share-link-copied-toast"
          open={toastOpen}
          onOpenChange={setToastOpen}
        >
          <Toast.Description>Synced link copied</Toast.Description>
        </Toast.Root>
        <Toast.Viewport className="export-audio-toast-viewport" />
      </Toast.Provider>
    </div>
  )
}
