import type { NoteTimingOut } from 'jianpu-wasm'
import { useEffect } from 'react'
import {
  groupNoteTimingsByPart,
  resolveActiveNotes,
} from './noteHighlightPosition'

const HIGHLIGHT_FILL = 'rgba(220,38,38,0.25)'

function noteKey(sourcePartIndex: number, noteId: number): string {
  return `${sourcePartIndex}:${noteId}`
}

/**
 * Imperatively toggles the `fill` of each currently-sounding note/rest's
 * background rect (`data-tag="note"` groups, see `PreviewSvgRenderer.tsx`)
 * in sync with `audio`'s playback position, independently per part — the
 * per-note replacement for the old measure-level playhead. Runs outside
 * React state/rendering (rAF, direct attribute writes) since it updates
 * every animation frame.
 */
export function useNoteHighlight(
  containerRef: React.RefObject<HTMLDivElement | null>,
  audio: HTMLAudioElement | null | undefined,
  noteTimings: NoteTimingOut[] | undefined,
) {
  useEffect(() => {
    const container = containerRef.current
    if (!audio || !container || !noteTimings || noteTimings.length === 0) {
      return
    }

    const timingsByPart = groupNoteTimingsByPart(noteTimings)
    let activeKeys = new Set<string>()
    let rafId: number | null = null

    const setHighlight = (
      sourcePartIndex: number,
      noteId: number,
      on: boolean,
    ) => {
      const rects = container.querySelectorAll<SVGRectElement>(
        `[data-tag="note"][data-part-index="${sourcePartIndex}"][data-note-id="${noteId}"] rect[data-variant="note-highlight-rect"]`,
      )
      for (const rect of rects) {
        rect.setAttribute('fill', on ? HIGHLIGHT_FILL : 'transparent')
      }
    }

    const clearActive = () => {
      for (const key of activeKeys) {
        const [partIndex, noteId] = key.split(':')
        setHighlight(Number(partIndex), Number(noteId), false)
      }
      activeKeys = new Set()
    }

    const updatePosition = () => {
      const active = resolveActiveNotes(audio.currentTime, timingsByPart)
      const nextKeys = new Set(
        active.map((a) => noteKey(a.sourcePartIndex, a.noteId)),
      )
      for (const key of activeKeys) {
        if (!nextKeys.has(key)) {
          const [partIndex, noteId] = key.split(':')
          setHighlight(Number(partIndex), Number(noteId), false)
        }
      }
      for (const a of active) {
        const key = noteKey(a.sourcePartIndex, a.noteId)
        if (!activeKeys.has(key)) {
          setHighlight(a.sourcePartIndex, a.noteId, true)
        }
      }
      activeKeys = nextKeys
    }

    const tick = () => {
      updatePosition()
      rafId = requestAnimationFrame(tick)
    }
    const start = () => {
      if (rafId === null) rafId = requestAnimationFrame(tick)
    }
    const stop = () => {
      if (rafId !== null) {
        cancelAnimationFrame(rafId)
        rafId = null
      }
      clearActive()
    }

    audio.addEventListener('play', start)
    audio.addEventListener('pause', stop)
    audio.addEventListener('ended', stop)
    if (!audio.paused) start()

    return () => {
      audio.removeEventListener('play', start)
      audio.removeEventListener('pause', stop)
      audio.removeEventListener('ended', stop)
      stop()
    }
  }, [containerRef, audio, noteTimings])
}
