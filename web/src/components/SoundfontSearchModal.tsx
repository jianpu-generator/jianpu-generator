import * as Dialog from '@radix-ui/react-dialog'
import { useState } from 'react'
import type { SoundfontValue } from '../types'
import { GM_INSTRUMENTS } from '../utils/gmInstruments'
import { GM_PERCUSSION } from '../utils/gmPercussion'
import { SoundfontSearchRow } from './SoundfontSearchRow'
import {
  type ActiveTag,
  instrumentFuzzyScore,
  percussionFuzzyScore,
  tagKey,
} from './soundfontSearchHelpers'

export function SoundfontSearchModal({
  open,
  onOpenChange,
  mode,
  currentValue,
  onSelect,
  previewInstrument,
  previewPercussion,
  stopPreviewInstrument,
  previewAudioPlaying,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  mode: 'instrument' | 'percussion'
  currentValue: SoundfontValue | null
  onSelect: (value: SoundfontValue | null) => void
  previewInstrument: (programNumber: number) => void
  previewPercussion: (key: number) => void
  stopPreviewInstrument: () => void
  previewAudioPlaying: boolean
}) {
  const [query, setQuery] = useState('')
  const [activeTags, setActiveTags] = useState<Map<string, ActiveTag>>(
    new Map(),
  )
  const [previewingNumber, setPreviewingNumber] = useState<number | null>(null)

  function toggleTag(tag: ActiveTag) {
    const key = tagKey(tag)
    setActiveTags((prev) => {
      const next = new Map(prev)
      if (next.has(key)) {
        next.delete(key)
      } else {
        next.set(key, tag)
      }
      return next
    })
  }

  const filteredInstruments =
    mode === 'instrument'
      ? GM_INSTRUMENTS.flatMap((instrument) => {
          for (const tag of activeTags.values()) {
            if (tag.kind === 'category' && instrument.category !== tag.value)
              return []
            if (tag.kind === 'source' && instrument.source !== tag.value)
              return []
            if (tag.kind === 'role' && instrument.role !== tag.value) return []
            if (
              tag.kind === 'articulation' &&
              instrument.articulation !== tag.value
            )
              return []
          }
          if (query.trim() === '') return [{ instrument, score: 0 }]
          const score = instrumentFuzzyScore(query, instrument)
          if (score === 0) return []
          return [{ instrument, score }]
        }).sort((a, b) => b.score - a.score)
      : []

  const filteredPercussion =
    mode === 'percussion'
      ? GM_PERCUSSION.flatMap((entry) => {
          if (query.trim() === '') return [{ entry, score: 0 }]
          const score = percussionFuzzyScore(query, entry)
          if (score === 0) return []
          return [{ entry, score }]
        }).sort((a, b) => b.score - a.score)
      : []

  function handlePlay(value: SoundfontValue) {
    const number = parseInt(value.split(':')[0], 10)
    if (previewingNumber === number && previewAudioPlaying) {
      stopPreviewInstrument()
      setPreviewingNumber(null)
    } else {
      setPreviewingNumber(number)
      if (mode === 'percussion') {
        previewPercussion(number)
      } else {
        previewInstrument(number)
      }
    }
  }

  function handleOpenChange(nextOpen: boolean) {
    if (nextOpen) {
      setQuery('')
      setActiveTags(new Map())
    }
    onOpenChange(nextOpen)
  }

  return (
    <Dialog.Root open={open} onOpenChange={handleOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay
          style={{
            position: 'fixed',
            inset: 0,
            background: 'rgba(0,0,0,0.35)',
            zIndex: 1100,
          }}
        />
        <Dialog.Content
          style={{
            position: 'fixed',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            background: '#fff',
            border: '1px solid #ddd',
            borderRadius: '6px',
            boxShadow: '0 8px 32px rgba(0,0,0,0.16)',
            zIndex: 1101,
            width: '60vw',
            maxWidth: '90vw',
            minWidth: '400px',
            maxHeight: '80vh',
            display: 'flex',
            flexDirection: 'column',
            fontFamily: 'var(--mono, monospace)',
          }}
        >
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              padding: '12px 16px',
              borderBottom: '1px solid #eee',
            }}
          >
            <Dialog.Title
              style={{ margin: 0, fontSize: '14px', fontWeight: 600 }}
            >
              {mode === 'percussion'
                ? 'Select percussion sound'
                : 'Select soundfont'}
            </Dialog.Title>
            <Dialog.Close
              style={{
                background: 'none',
                border: 'none',
                cursor: 'pointer',
                fontSize: '16px',
                color: '#666',
                lineHeight: 1,
                padding: '2px 4px',
              }}
            >
              ×
            </Dialog.Close>
          </div>

          <div style={{ padding: '8px 12px', borderBottom: '1px solid #eee' }}>
            <input
              type="text"
              placeholder="Search..."
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              style={{
                width: '100%',
                boxSizing: 'border-box',
                fontSize: '13px',
                fontFamily: 'var(--mono, monospace)',
                border: '1px solid #cbd5e0',
                borderRadius: '3px',
                padding: '4px 8px',
                outline: 'none',
              }}
            />
          </div>

          <div style={{ overflowY: 'auto', flex: 1 }}>
            <SoundfontSearchRow
              label="default sound"
              tags={null}
              activeTags={activeTags}
              isSelected={currentValue === null}
              isPreviewing={false}
              onPlay={null}
              onSelect={() => onSelect(null)}
              onTagClick={toggleTag}
            />
            {filteredInstruments.map(({ instrument }) => {
              const programNumber = parseInt(instrument.value.split(':')[0], 10)
              return (
                <SoundfontSearchRow
                  key={instrument.value}
                  label={instrument.value}
                  tags={instrument}
                  activeTags={activeTags}
                  isSelected={currentValue === instrument.value}
                  isPreviewing={
                    previewingNumber === programNumber && previewAudioPlaying
                  }
                  onPlay={() => handlePlay(instrument.value)}
                  onSelect={() => onSelect(instrument.value)}
                  onTagClick={toggleTag}
                />
              )
            })}
            {filteredPercussion.map(({ entry }) => (
              <SoundfontSearchRow
                key={entry.value}
                label={entry.value}
                tags={null}
                activeTags={activeTags}
                isSelected={currentValue === entry.value}
                isPreviewing={
                  previewingNumber === entry.key && previewAudioPlaying
                }
                onPlay={() => handlePlay(entry.value)}
                onSelect={() => onSelect(entry.value)}
                onTagClick={toggleTag}
              />
            ))}
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
