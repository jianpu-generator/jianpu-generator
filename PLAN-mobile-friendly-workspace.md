# Mobile-friendly workspace

The app has no responsive design today — `grep -r "media" web/src --include="*.css"` returns
zero matches. Editor and preview are always side-by-side, and the toolbars above them wrap
instead of scrolling, which is especially bad with many sections/sequence entries/parts on a
narrow screen. Three independent tasks, each implementable on its own.

## Task 1: Responsive breakpoint — stack panes instead of side-by-side

**Where:** `web/src/App.css` (`.workspace`, `.pane`, `.pane--editor`, `.pane-divider`),
`web/src/components/AppWorkspace.tsx`

**Context:** `.workspace` is `display: flex` (row) with two `flex: 1` panes
(`App.css:91-134`). Below some width threshold (e.g. `max-width: 768px`), this needs to become
a vertical stack (`flex-direction: column`) or a tabbed view where only one pane is visible at a
time. The existing `editorCollapsed` state/toggle (`pane-divider-toggle`) already proves the
"show one pane at a time" interaction works — the mobile breakpoint should likely reuse that
same collapse mechanism rather than invent a new one, just make it the default/forced state
below the breakpoint instead of a manual toggle. Need to decide: pure CSS media query flip, or
does `AppWorkspace.tsx` need a `useMediaQuery`-style hook to drive `editorCollapsed` state and
swap divider behavior for a tab-switcher UI. Also check `Preview.tsx`/`PreviewSvgRenderer.tsx`
for any fixed-width assumptions in the SVG scaling.

## Task 2: Toolbars scroll horizontally instead of wrapping

**Where:** `web/src/components/PartToggles.css` (`.part-toggles-list`), `web/src/App.css`
(`.workspace-toolbar-sections`), consumed by `PartToggles.tsx`, `SequenceJumpToolbar.tsx`,
`SectionJumpToolbar.tsx`

**Context:** All three toolbars use `flex-wrap: wrap` (`PartToggles.css:32-39`,
`App.css:83-89`). With many parts/sections/sequence entries this grows tall and pushes the panes
down, worst on mobile where there's least vertical room to begin with. Fix is
`flex-wrap: nowrap` + `overflow-x: auto` + `-webkit-overflow-scrolling: touch`, with
`flex-shrink: 0` on the individual pill/button items so they don't get crushed. Should apply
uniformly to all three toolbars since they share the pattern — worth checking if they should
share a CSS class instead of duplicating the scroll rules three times. Verify Radix tooltip
content (`.part-toggle-tooltip-content`) doesn't get clipped by the new `overflow-x` container.

## Task 3: Touch support for sequence-entry range selection

**Where:** `web/src/components/SequenceJumpToolbar.tsx` (drag handlers, lines 59-93), possibly
`SectionJumpToolbar.tsx` if it has similar logic

**Context:** Range selection is driven entirely by `onMouseDown`/`onMouseEnter`/`onMouseUp`/
`onMouseLeave` — no touch equivalents, so the feature is silently broken on touchscreens (touch
doesn't fire `mouseenter` per-element the way a real drag does). Needs `onTouchStart`/
`onTouchMove`/`onTouchEnd`, with `onTouchMove` using
`document.elementFromPoint(touch.clientX, touch.clientY)` to figure out which button the finger
is currently over (since touchmove targets stay pinned to the element where the touch started,
unlike mouse). Also needs `touch-action: none` on the toolbar to prevent the page from scrolling
while dragging. Check `SectionJumpToolbar.tsx` for the same pattern before assuming it's
SequenceJumpToolbar-only.
