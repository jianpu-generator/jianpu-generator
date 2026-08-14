import { describe, expect, it } from 'vitest'
import { clearStaleHighlights, PLAYBACK_CURSOR_FILL } from './usePlaybackCursor'

// Minimal duck-typed stand-ins for the DOM shapes `clearStaleHighlights`
// touches (`Element.closest`/`getAttribute`/`setAttribute`,
// `container.querySelectorAll`) — no jsdom in this project's test setup, and
// none is needed since the function only calls these few methods.
class FakeGroup {
  attrs: Record<string, string>
  constructor(attrs: Record<string, string>) {
    this.attrs = attrs
  }
  getAttribute(name: string): string | null {
    return this.attrs[name] ?? null
  }
}

class FakeRect {
  fill: string
  group: FakeGroup | null
  constructor(fill: string, group: FakeGroup | null) {
    this.fill = fill
    this.group = group
  }
  closest(_selector: string): FakeGroup | null {
    return this.group
  }
  setAttribute(name: string, value: string): void {
    if (name === 'fill') this.fill = value
  }
}

class FakeContainer {
  rects: FakeRect[]
  constructor(rects: FakeRect[]) {
    this.rects = rects
  }
  querySelectorAll(_selector: string): FakeRect[] {
    return this.rects
  }
}

describe('clearStaleHighlights', () => {
  it('clears a leftover highlight even when the DOM node was reused for a different note', () => {
    // Reproduces the leftover-cursor bug: React keys the note `<g>` groups
    // by array index (see `renderSvgElement` in `PreviewSvgRenderer.tsx`),
    // and always renders `fill="transparent"` as a literal prop on
    // `playbackCursorRect`. If a rect got imperatively painted red while
    // highlighting one note, then the score reshuffles so the same DOM node
    // now represents a *different* note that was never actually played, the
    // stale red fill must still be detected and cleared — even though
    // nothing ever explicitly turned this note's highlight on.
    const group = new FakeGroup({
      'data-part-index': '0',
      'data-note-id': '7',
    })
    const rect = new FakeRect(PLAYBACK_CURSOR_FILL, group)
    const container = new FakeContainer([rect])

    clearStaleHighlights(container, new Set())

    expect(rect.fill).toBe('transparent')
  })

  it('leaves a highlight alone when its note is still active', () => {
    const group = new FakeGroup({
      'data-part-index': '0',
      'data-note-id': '7',
    })
    const rect = new FakeRect(PLAYBACK_CURSOR_FILL, group)
    const container = new FakeContainer([rect])

    clearStaleHighlights(container, new Set(['0:7']))

    expect(rect.fill).toBe(PLAYBACK_CURSOR_FILL)
  })
})
