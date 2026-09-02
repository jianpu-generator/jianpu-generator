import * as Toast from '@radix-ui/react-toast'
import { useEffect, useRef, useState } from 'react'
import './ExportAudioToast.css'

// Exporting a tiny score (or even a real one, on a fast machine) can finish
// before the toast has had any real time on screen — as far as a user is
// concerned that reads as a flicker, not useful feedback. Holding it open
// for at least this long avoids that regardless of how fast the export is.
const MIN_VISIBLE_MS = 500

/** Shows a toast with a spinner while a WAV/MP3 export is running in the
 *  background. Needed because the export dropdown closes as soon as an item
 *  is clicked, so its own busy label (e.g. "Generating MP3…") isn't visible
 *  during export. */
export function ExportAudioToast({
  open,
  label,
}: {
  open: boolean
  label: string
}) {
  const [visible, setVisible] = useState(false)
  const [displayLabel, setDisplayLabel] = useState(label)
  const openedAtRef = useRef<number | null>(null)
  const hideTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => {
    if (hideTimerRef.current !== null) {
      clearTimeout(hideTimerRef.current)
      hideTimerRef.current = null
    }

    if (open) {
      openedAtRef.current = performance.now()
      setDisplayLabel(label)
      setVisible(true)
      return
    }

    if (openedAtRef.current === null) {
      setVisible(false)
      return
    }
    const elapsed = performance.now() - openedAtRef.current
    const remaining = Math.max(0, MIN_VISIBLE_MS - elapsed)
    hideTimerRef.current = setTimeout(() => {
      hideTimerRef.current = null
      setVisible(false)
    }, remaining)

    return () => {
      if (hideTimerRef.current !== null) clearTimeout(hideTimerRef.current)
    }
  }, [open, label])

  return (
    <Toast.Provider swipeDirection="right" duration={Infinity}>
      <Toast.Root
        className="export-audio-toast"
        data-testid="export-audio-toast"
        open={visible}
      >
        <span className="file-tab-bar-spinner" aria-hidden="true" />
        <Toast.Description>{displayLabel}</Toast.Description>
      </Toast.Root>
      <Toast.Viewport className="export-audio-toast-viewport" />
    </Toast.Provider>
  )
}
