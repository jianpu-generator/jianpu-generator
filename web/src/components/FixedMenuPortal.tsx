import type { ReactNode } from 'react'
import { createPortal } from 'react-dom'

interface FixedMenuPortalProps {
  children: ReactNode
}

/** Renders a `position: fixed` dropdown menu into `document.body`, outside
 * its trigger's DOM subtree.
 *
 * The app header (and other toolbars) scroll horizontally via
 * `overflow-x: auto` plus `-webkit-overflow-scrolling: touch`. That last
 * property pins the header to its own compositing layer in Safari, and
 * WebKit then paints `position: fixed` descendants inside that layer
 * instead of letting them escape to the viewport's stacking context —
 * so the menu renders but ends up visually behind whatever comes after
 * the header in the DOM (e.g. the second toolbar row). Chromium doesn't
 * do this, which is why the bug only shows up in Safari/WKWebView.
 * Portaling to `document.body` sidesteps the whole layer/stacking
 * question by never nesting the menu inside the scrollable ancestor. */
export function FixedMenuPortal({ children }: FixedMenuPortalProps) {
  return createPortal(children, document.body)
}
