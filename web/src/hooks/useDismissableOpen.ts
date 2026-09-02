import { type RefObject, useEffect, useState } from 'react'

/** Open/close state for a dropdown-like panel that dismisses itself on an
 * outside click or Escape.
 *
 * `menuRef` is optional and only needed when the panel's content is
 * rendered via `FixedMenuPortal` (i.e. outside `containerRef`'s own DOM
 * subtree, in `document.body`) — without it, a click inside the portaled
 * menu would look like an outside click and close the menu before its own
 * `onClick` runs. */
export function useDismissableOpen(
  containerRef: RefObject<HTMLElement | null>,
  menuRef?: RefObject<HTMLElement | null>,
): [boolean, (next: boolean | ((prev: boolean) => boolean)) => void] {
  const [open, setOpen] = useState(false)

  useEffect(() => {
    if (!open) return
    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as Node
      if (
        !containerRef.current?.contains(target) &&
        !menuRef?.current?.contains(target)
      ) {
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
  }, [open, containerRef, menuRef])

  return [open, setOpen]
}
