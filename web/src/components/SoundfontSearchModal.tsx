import * as Dialog from '@radix-ui/react-dialog'
import { useState } from 'react'
import { useSoundChoices } from '../hooks/useSoundChoices'
import { GM_INSTRUMENTS } from '../utils/gmInstruments'
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
  currentProgram,
  onSelect,
  previewInstrument,
  previewPercussion,
  stopPreviewInstrument,
  previewAudioPlaying,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  mode: 'instrument' | 'percussion'
  currentProgram: number | null
  onSelect: (program: number | null) => void
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
  const soundChoices = useSoundChoices(open)
  const instrumentLabels = new Map(
    (soundChoices?.instruments ?? []).map((choice) => [
      choice.program,
      choice.label,
    ]),
  )

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
          const label = instrumentLabels.get(instrument.program)
          if (label === undefined) return []
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
          if (query.trim() === '') return [{ instrument, label, score: 0 }]
          const score = instrumentFuzzyScore(query, instrument, label)
          if (score === 0) return []
          return [{ instrument, label, score }]
        }).sort((a, b) => b.score - a.score)
      : []

  const filteredPercussion =
    mode === 'percussion'
      ? (soundChoices?.percussion ?? [])
          .flatMap((choice) => {
            if (query.trim() === '') return [{ choice, score: 0 }]
            const score = percussionFuzzyScore(query, choice)
            if (score === 0) return []
            return [{ choice, score }]
          })
          .sort((a, b) => b.score - a.score)
      : []

  function handlePlay(number: number) {
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
              isSelected={currentProgram === null}
              isPreviewing={false}
              onPlay={null}
              onSelect={() => onSelect(null)}
              onTagClick={toggleTag}
            />
            {filteredInstruments.map(({ instrument, label }) => (
              <SoundfontSearchRow
                key={instrument.program}
                label={label}
                tags={instrument}
                activeTags={activeTags}
                isSelected={currentProgram === instrument.program}
                isPreviewing={
                  previewingNumber === instrument.program && previewAudioPlaying
                }
                onPlay={() => handlePlay(instrument.program)}
                onSelect={() => onSelect(instrument.program)}
                onTagClick={toggleTag}
              />
            ))}
            {filteredPercussion.map(({ choice }) => (
              <SoundfontSearchRow
                key={choice.program}
                label={choice.label}
                tags={null}
                activeTags={activeTags}
                isSelected={currentProgram === choice.program}
                isPreviewing={
                  previewingNumber === choice.program && previewAudioPlaying
                }
                onPlay={() => handlePlay(choice.program)}
                onSelect={() => onSelect(choice.program)}
                onTagClick={toggleTag}
              />
            ))}
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
