import {
  type CSSProperties,
  type RefObject,
  useLayoutEffect,
  useState,
} from 'react'

/** Positions a dropdown menu as `fixed`, anchored to its trigger button's
 * live viewport rect. The app header scrolls horizontally on narrow
 * viewports (`overflow-x: auto`), which per the CSS overflow spec forces
 * its `overflow-y` to `auto` too, clipping a plain `position: absolute`
 * menu instead of letting it float over the page. `position: fixed`
 * escapes that clip entirely. */
export function useFixedMenuPosition(
  buttonRef: RefObject<HTMLElement | null>,
  open: boolean,
): CSSProperties | undefined {
  const [menuStyle, setMenuStyle] = useState<CSSProperties | null>(null)

  useLayoutEffect(() => {
    if (!open) {
      setMenuStyle(null)
      return
    }
    const updatePosition = () => {
      const rect = buttonRef.current?.getBoundingClientRect()
      if (!rect) return
      setMenuStyle({
        position: 'fixed',
        top: rect.bottom + 4,
        right: window.innerWidth - rect.right,
      })
    }
    updatePosition()
    window.addEventListener('resize', updatePosition)
    window.addEventListener('scroll', updatePosition, true)
    return () => {
      window.removeEventListener('resize', updatePosition)
      window.removeEventListener('scroll', updatePosition, true)
    }
  }, [open, buttonRef])

  return menuStyle ?? undefined
}
