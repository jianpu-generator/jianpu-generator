import { ChevronDownIcon, Link2Icon, VideoIcon } from '@radix-ui/react-icons'
import * as Toast from '@radix-ui/react-toast'
import { useCallback, useState } from 'react'
import { ResponsiveMenu } from './ResponsiveMenu'

interface GoLiveButtonProps {
  isLive: boolean
  liveUrl: string | null
  onStartLive: () => string
  onStopLive: () => void
  className?: string
}

export function GoLiveButton({
  isLive,
  liveUrl,
  onStartLive,
  onStopLive,
  className = 'preview-export-btn',
}: GoLiveButtonProps) {
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
      {isLive ? (
        // Radix's DropdownMenuTrigger opens the menu on `pointerdown` and
        // `preventDefault()`s it whenever the menu is currently closed —
        // which, per spec, suppresses the `click` event that would
        // otherwise follow. So a Trigger can only ever toggle its own
        // menu; it can't also carry a "start live" click handler for the
        // not-live state below. Rendering the plain "Go Live" button
        // outside any Trigger, and only wrapping this "Live" button in one
        // once there's a menu for it to open, sidesteps that entirely.
        <ResponsiveMenu.Root open={menuOpen} onOpenChange={setMenuOpen}>
          <ResponsiveMenu.Trigger asChild>
            <button
              type="button"
              className={className}
              data-testid="go-live-button"
              aria-label="Live options"
            >
              <VideoIcon aria-hidden="true" />
              Live
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
            title="Live options"
          >
            <ResponsiveMenu.Item
              className="export-menu-item"
              data-testid="copy-live-link-button"
              onSelect={() => {
                if (liveUrl) void copyUrl(liveUrl)
              }}
            >
              <Link2Icon aria-hidden="true" />
              Copy Live Link
            </ResponsiveMenu.Item>
            <ResponsiveMenu.Item
              className="export-menu-item"
              data-testid="stop-live-button"
              onSelect={onStopLive}
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
              Stop Live
            </ResponsiveMenu.Item>
          </ResponsiveMenu.Content>
        </ResponsiveMenu.Root>
      ) : (
        <button
          type="button"
          className={className}
          data-testid="go-live-button"
          aria-label="Go live"
          title="Anyone with this link can view your score live. Don't share it publicly."
          onClick={() => void copyUrl(onStartLive())}
        >
          <VideoIcon aria-hidden="true" />
          Go Live
        </button>
      )}
      <Toast.Provider swipeDirection="right" duration={3000}>
        <Toast.Root
          className="export-audio-toast"
          data-testid="live-link-copied-toast"
          open={toastOpen}
          onOpenChange={setToastOpen}
        >
          <Toast.Description>Live link copied</Toast.Description>
        </Toast.Root>
        <Toast.Viewport className="export-audio-toast-viewport" />
      </Toast.Provider>
    </div>
  )
}
