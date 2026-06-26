import * as Dialog from '@radix-ui/react-dialog'
import { Pause, Play } from 'lucide-react'
import { useState } from 'react'
import type {
  InstrumentArticulation,
  InstrumentCategory,
  InstrumentEntry,
  InstrumentRole,
  InstrumentSource,
} from '../utils/gmInstruments'
import { GM_INSTRUMENTS } from '../utils/gmInstruments'
import type { SoundfontValue } from '../utils/partSource'

type ActiveTag =
  | { kind: 'category'; value: InstrumentCategory }
  | { kind: 'source'; value: InstrumentSource }
  | { kind: 'role'; value: InstrumentRole }
  | { kind: 'articulation'; value: InstrumentArticulation }

function tagKey(tag: ActiveTag): string {
  return `${tag.kind}:${tag.value}`
}

function fuzzyScore(query: string, target: string): number {
  const q = query.toLowerCase()
  const t = target.toLowerCase()
  if (t.includes(q)) return 1000
  let score = 0
  let qi = 0
  let consecutive = 0
  for (let ti = 0; ti < t.length && qi < q.length; ti++) {
    if (t[ti] === q[qi]) {
      score += 1 + consecutive * 2
      consecutive++
      qi++
    } else {
      consecutive = 0
    }
  }
  return qi === q.length ? score : 0
}

function instrumentFuzzyScore(
  query: string,
  instrument: InstrumentEntry,
): number {
  return Math.max(
    fuzzyScore(query, instrument.value),
    fuzzyScore(query, instrument.category),
    fuzzyScore(query, instrument.source),
    fuzzyScore(query, instrument.role),
    fuzzyScore(query, instrument.articulation),
  )
}

function InlineTag({
  label,
  active,
  onClick,
}: {
  label: string
  active: boolean
  onClick: (e: React.MouseEvent) => void
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      style={{
        fontSize: '10px',
        padding: '1px 5px',
        borderRadius: '8px',
        border: active ? '1px solid #3b82f6' : '1px solid #cbd5e0',
        background: active ? '#dbeafe' : '#f1f5f9',
        color: active ? '#1d4ed8' : '#777',
        cursor: 'pointer',
        fontFamily: 'var(--mono, monospace)',
        whiteSpace: 'nowrap',
        flexShrink: 0,
      }}
    >
      #{label}
    </button>
  )
}

function SoundfontSearchRow({
  label,
  tags,
  activeTags,
  isSelected,
  isPreviewing,
  onPlay,
  onSelect,
  onTagClick,
}: {
  label: string
  tags: {
    category: InstrumentCategory
    source: InstrumentSource
    role: InstrumentRole
    articulation: InstrumentArticulation
  } | null
  activeTags: Map<string, ActiveTag>
  isSelected: boolean
  isPreviewing: boolean
  onPlay: (() => void) | null
  onSelect: () => void
  onTagClick: (tag: ActiveTag) => void
}) {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        padding: '4px 8px',
        background: isSelected ? '#e8f0fe' : undefined,
        fontSize: '12px',
        fontFamily: 'var(--mono, monospace)',
        gap: '6px',
      }}
    >
      {onPlay !== null ? (
        <button
          type="button"
          onClick={onPlay}
          title={isPreviewing ? 'Pause preview' : 'Preview instrument'}
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            justifyContent: 'center',
            width: '24px',
            height: '24px',
            borderRadius: '50%',
            border: 'none',
            background: isPreviewing ? '#dbeafe' : 'transparent',
            cursor: 'pointer',
            color: isPreviewing ? '#1d4ed8' : '#888',
            flexShrink: 0,
            padding: 0,
            transition: 'background 0.15s, color 0.15s',
          }}
        >
          {isPreviewing ? (
            <Pause size={13} fill="currentColor" strokeWidth={0} />
          ) : (
            <Play size={13} fill="currentColor" strokeWidth={0} />
          )}
        </button>
      ) : (
        <span
          style={{ display: 'inline-block', width: '24px', flexShrink: 0 }}
        />
      )}
      <button
        type="button"
        onClick={onSelect}
        style={{
          background: 'none',
          border: 'none',
          cursor: 'pointer',
          fontSize: '12px',
          fontFamily: 'var(--mono, monospace)',
          textAlign: 'left',
          padding: 0,
          flex: 1,
          minWidth: 0,
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          whiteSpace: 'nowrap',
        }}
      >
        {label}
      </button>
      {tags !== null && (
        <div style={{ display: 'flex', gap: '3px', flexShrink: 0 }}>
          <InlineTag
            label={tags.category}
            active={activeTags.has(
              tagKey({ kind: 'category', value: tags.category }),
            )}
            onClick={(e) => {
              e.stopPropagation()
              onTagClick({ kind: 'category', value: tags.category })
            }}
          />
          <InlineTag
            label={tags.source}
            active={activeTags.has(
              tagKey({ kind: 'source', value: tags.source }),
            )}
            onClick={(e) => {
              e.stopPropagation()
              onTagClick({ kind: 'source', value: tags.source })
            }}
          />
          <InlineTag
            label={tags.role}
            active={activeTags.has(tagKey({ kind: 'role', value: tags.role }))}
            onClick={(e) => {
              e.stopPropagation()
              onTagClick({ kind: 'role', value: tags.role })
            }}
          />
          <InlineTag
            label={tags.articulation}
            active={activeTags.has(
              tagKey({ kind: 'articulation', value: tags.articulation }),
            )}
            onClick={(e) => {
              e.stopPropagation()
              onTagClick({ kind: 'articulation', value: tags.articulation })
            }}
          />
        </div>
      )}
    </div>
  )
}

export function SoundfontSearchModal({
  open,
  onOpenChange,
  currentValue,
  onSelect,
  previewInstrument,
  stopPreviewInstrument,
  previewAudioPlaying,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  currentValue: SoundfontValue | null
  onSelect: (value: SoundfontValue | null) => void
  previewInstrument: (programNumber: number) => void
  stopPreviewInstrument: () => void
  previewAudioPlaying: boolean
}) {
  const [query, setQuery] = useState('')
  const [activeTags, setActiveTags] = useState<Map<string, ActiveTag>>(
    new Map(),
  )
  const [previewingProgramNumber, setPreviewingProgramNumber] = useState<
    number | null
  >(null)

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

  const filtered = GM_INSTRUMENTS.flatMap((instrument) => {
    for (const tag of activeTags.values()) {
      if (tag.kind === 'category' && instrument.category !== tag.value)
        return []
      if (tag.kind === 'source' && instrument.source !== tag.value) return []
      if (tag.kind === 'role' && instrument.role !== tag.value) return []
      if (tag.kind === 'articulation' && instrument.articulation !== tag.value)
        return []
    }
    if (query.trim() === '') return [{ instrument, score: 0 }]
    const score = instrumentFuzzyScore(query, instrument)
    if (score === 0) return []
    return [{ instrument, score }]
  }).sort((a, b) => b.score - a.score)

  function handlePlay(instrument: InstrumentEntry) {
    const programNumber = parseInt(instrument.value.split(':')[0], 10)
    if (previewingProgramNumber === programNumber && previewAudioPlaying) {
      stopPreviewInstrument()
      setPreviewingProgramNumber(null)
    } else {
      setPreviewingProgramNumber(programNumber)
      previewInstrument(programNumber)
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
              Select soundfont
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
            {filtered.map(({ instrument }) => {
              const programNumber = parseInt(instrument.value.split(':')[0], 10)
              return (
                <SoundfontSearchRow
                  key={instrument.value}
                  label={instrument.value}
                  tags={instrument}
                  activeTags={activeTags}
                  isSelected={currentValue === instrument.value}
                  isPreviewing={
                    previewingProgramNumber === programNumber &&
                    previewAudioPlaying
                  }
                  onPlay={() => handlePlay(instrument)}
                  onSelect={() => onSelect(instrument.value)}
                  onTagClick={toggleTag}
                />
              )
            })}
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
