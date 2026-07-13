import { Pause, Play } from 'lucide-react'
import type {
  InstrumentArticulation,
  InstrumentCategory,
  InstrumentRole,
  InstrumentSource,
} from '../utils/gmInstruments'
import type { ActiveTag } from './soundfontSearchHelpers'
import { tagKey } from './soundfontSearchHelpers'

export function InlineTag({
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

export function SoundfontSearchRow({
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
