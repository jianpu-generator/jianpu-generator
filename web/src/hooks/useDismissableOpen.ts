import { type RefObject, useEffect, useState } from 'react'

/** Open/close state for a dropdown-like panel that dismisses itself on an
 * outside click or Escape. */
export function useDismissableOpen(
  containerRef: RefObject<HTMLElement | null>,
): [boolean, (next: boolean | ((prev: boolean) => boolean)) => void] {
  const [open, setOpen] = useState(false)

  useEffect(() => {
    if (!open) return
    const handleClickOutside = (event: MouseEvent) => {
      if (!containerRef.current?.contains(event.target as Node)) {
        setOpen(false)
      }
    }
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setOpen(false)
    }
    document.addEventListener('mousedown', handleClickOutside)
    document.addEventListener('keydown', handleKeyDown)
    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
      document.removeEventListener('keydown', handleKeyDown)
    }
  }, [open, containerRef])

  return [open, setOpen]
}
