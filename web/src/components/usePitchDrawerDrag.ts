import type { PointerEvent, RefObject } from 'react'
import { useRef, useState } from 'react'

/** Drag-down-to-close for the pitch drawer's handle. Pointer events, so
 * touch and mouse share one path. Releasing past half the drawer's height
 * calls `onDismiss`; a shorter drag snaps back. */
export function usePitchDrawerDrag(
  drawerRef: RefObject<HTMLElement | null>,
  onDismiss: () => void,
) {
  // How far the drawer is dragged down, in px; `null` when not dragging.
  const [dragOffset, setDragOffset] = useState<number | null>(null)
  const startYRef = useRef(0)

  const endDrag = () => {
    if (dragOffset === null) return
    const height = drawerRef.current?.offsetHeight ?? 0
    if (dragOffset > height / 2) onDismiss()
    setDragOffset(null)
  }

  return {
    dragOffset,
    handleProps: {
      onPointerDown: (e: PointerEvent<HTMLElement>) => {
        e.currentTarget.setPointerCapture(e.pointerId)
        startYRef.current = e.clientY
        setDragOffset(0)
      },
      onPointerMove: (e: PointerEvent<HTMLElement>) => {
        if (dragOffset === null) return
        setDragOffset(Math.max(0, e.clientY - startYRef.current))
      },
      onPointerUp: endDrag,
      onPointerCancel: endDrag,
    },
  }
}
