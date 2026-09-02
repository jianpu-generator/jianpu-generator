import { ChevronDownIcon, Link2Icon, VideoIcon } from '@radix-ui/react-icons'
import * as Toast from '@radix-ui/react-toast'
import { useCallback, useRef, useState } from 'react'
import { useDismissableOpen } from '../hooks/useDismissableOpen'
import { useFixedMenuPosition } from '../hooks/useFixedMenuPosition'
import { FixedMenuPortal } from './FixedMenuPortal'

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
  const containerRef = useRef<HTMLDivElement>(null)
  const buttonRef = useRef<HTMLButtonElement>(null)
  const menuRef = useRef<HTMLDivElement>(null)
  const [menuOpen, setMenuOpen] = useDismissableOpen(containerRef, menuRef)
  const menuStyle = useFixedMenuPosition(buttonRef, isLive && menuOpen)

  const copyUrl = useCallback(async (url: string) => {
    try {
      await navigator.clipboard.writeText(url)
      setToastOpen(true)
    } catch {
      window.prompt('Copy this link to share:', url)
    }
  }, [])

  const handleTriggerClick = useCallback(() => {
    if (isLive) {
      setMenuOpen((prev) => !prev)
    } else {
      void copyUrl(onStartLive())
    }
  }, [isLive, onStartLive, copyUrl, setMenuOpen])

  return (
    <div className="export-menu" ref={containerRef}>
      <button
        type="button"
        ref={buttonRef}
        className={className}
        data-testid="go-live-button"
        aria-haspopup={isLive ? 'menu' : undefined}
        aria-expanded={isLive ? menuOpen : undefined}
        aria-label={isLive ? 'Live options' : 'Go live'}
        title={
          isLive
            ? undefined
            : "Anyone with this link can view your score live. Don't share it publicly."
        }
        onClick={handleTriggerClick}
      >
        <VideoIcon aria-hidden="true" />
        {isLive ? 'Live' : 'Go Live'}
        {isLive && (
          <ChevronDownIcon className="export-menu-caret" aria-hidden="true" />
        )}
      </button>
      {isLive && menuOpen ? (
        <FixedMenuPortal>
          <div
            className="export-menu-list"
            role="menu"
            style={menuStyle}
            ref={menuRef}
          >
            <button
              type="button"
              role="menuitem"
              className="export-menu-item"
              data-testid="copy-live-link-button"
              onClick={() => {
                setMenuOpen(false)
                if (liveUrl) void copyUrl(liveUrl)
              }}
            >
              <Link2Icon aria-hidden="true" />
              Copy Live Link
            </button>
            <button
              type="button"
              role="menuitem"
              className="export-menu-item"
              data-testid="stop-live-button"
              onClick={() => {
                setMenuOpen(false)
                onStopLive()
              }}
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
            </button>
          </div>
        </FixedMenuPortal>
      ) : null}
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
