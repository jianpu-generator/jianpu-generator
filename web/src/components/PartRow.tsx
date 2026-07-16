import * as Slider from '@radix-ui/react-slider'
import { useState } from 'react'
import type { PartDeclaration, PartInfo, SoundfontValue } from '../types'
import type { EditPartsModalProps } from './EditPartsModal'
import { RadixSelect, RadixSelectItem } from './RadixTableSelect'
import { SoundfontSearchModal } from './SoundfontSearchModal'

const OCTAVE_OPTIONS = [
  { value: '4', label: '+4' },
  { value: '3', label: '+3' },
  { value: '2', label: '+2' },
  { value: '1', label: '+1' },
  { value: '0', label: '0' },
  { value: '-1', label: '-1' },
  { value: '-2', label: '-2' },
  { value: '-3', label: '-3' },
  { value: '-4', label: '-4' },
] as const

export function PartRow({
  declaration,
  allParts,
  isFirstPart,
  onPartDeclarationChange,
  rowIndex,
  previewInstrument,
  previewPercussion,
  stopPreviewInstrument,
  previewAudioPlaying,
}: {
  declaration: PartDeclaration
  allParts: PartInfo[]
  isFirstPart: boolean
  onPartDeclarationChange: EditPartsModalProps['onPartDeclarationChange']
  rowIndex: number
  previewInstrument: (programNumber: number) => void
  previewPercussion: (key: number) => void
  stopPreviewInstrument: () => void
  previewAudioPlaying: boolean
}) {
  const [searchOpen, setSearchOpen] = useState(false)
  const partInfo = allParts.find(
    (p) => p.abbreviation === declaration.abbreviation,
  )
  const precedingParts = allParts.slice(
    0,
    allParts.findIndex((p) => p.abbreviation === declaration.abbreviation),
  )

  function handleModeChange(newMode: string) {
    const mode = newMode as PartDeclaration['mode']
    if (mode === 'follow') {
      const defaultTarget = precedingParts[0]?.abbreviation ?? null
      onPartDeclarationChange(
        declaration.abbreviation,
        mode,
        defaultTarget,
        declaration.soundfont ?? null,
        declaration.volume ?? null,
        declaration.octaveOffset ?? null,
      )
    } else {
      onPartDeclarationChange(
        declaration.abbreviation,
        mode,
        null,
        declaration.soundfont ?? null,
        declaration.volume ?? null,
        declaration.octaveOffset ?? null,
      )
    }
  }

  function handleFollowTargetChange(target: string) {
    onPartDeclarationChange(
      declaration.abbreviation,
      'follow',
      target,
      declaration.soundfont ?? null,
      declaration.volume ?? null,
      declaration.octaveOffset ?? null,
    )
  }

  function handleSoundfontChange(value: string) {
    const newSoundfont = value === '' ? null : (value as SoundfontValue)
    onPartDeclarationChange(
      declaration.abbreviation,
      declaration.mode,
      declaration.followTarget ?? null,
      newSoundfont,
      declaration.volume ?? null,
      declaration.octaveOffset ?? null,
    )
  }

  function handleVolumeChange(value: number) {
    const newVolume = value === 100 ? null : value
    onPartDeclarationChange(
      declaration.abbreviation,
      declaration.mode,
      declaration.followTarget ?? null,
      declaration.soundfont ?? null,
      newVolume,
      declaration.octaveOffset ?? null,
    )
  }

  function handleOctaveChange(value: string) {
    const parsed = parseInt(value, 10)
    const newOctaveOffset = parsed === 0 ? null : parsed
    onPartDeclarationChange(
      declaration.abbreviation,
      declaration.mode,
      declaration.followTarget ?? null,
      declaration.soundfont ?? null,
      declaration.volume ?? null,
      newOctaveOffset,
    )
  }

  const rowBg = rowIndex % 2 === 0 ? '#fafafa' : '#fff'

  return (
    <tr style={{ background: rowBg }}>
      <td style={tdStyle}>
        {partInfo?.display_name ?? declaration.abbreviation}
      </td>
      <td style={tdStyle}>
        <span
          style={{ fontFamily: 'var(--mono)', fontSize: '12px', color: '#666' }}
        >
          {declaration.abbreviation}
        </span>
      </td>
      <td style={tdStyle}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <RadixSelect
            value={declaration.mode}
            onValueChange={handleModeChange}
            placeholder="mode"
            testId={`mode-select-${declaration.abbreviation}`}
          >
            <RadixSelectItem value="chords">chords</RadixSelectItem>
            <RadixSelectItem value="notes">notes</RadixSelectItem>
            <RadixSelectItem value="notes+lyrics">notes+lyrics</RadixSelectItem>
            <RadixSelectItem value="percussion">percussion</RadixSelectItem>
            {!isFirstPart && (
              <RadixSelectItem value="follow">follow</RadixSelectItem>
            )}
          </RadixSelect>
          {declaration.mode === 'follow' && precedingParts.length > 0 && (
            <RadixSelect
              value={declaration.followTarget ?? precedingParts[0].abbreviation}
              onValueChange={handleFollowTargetChange}
              placeholder="target"
              testId={`follow-target-select-${declaration.abbreviation}`}
            >
              {precedingParts.map((part) => (
                <RadixSelectItem
                  key={part.abbreviation}
                  value={part.abbreviation}
                >
                  {part.abbreviation}
                </RadixSelectItem>
              ))}
            </RadixSelect>
          )}
        </div>
      </td>
      <td style={tdStyle}>
        <button
          type="button"
          onClick={() => setSearchOpen(true)}
          data-testid={`soundfont-select-${declaration.abbreviation}`}
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            fontSize: '12px',
            fontFamily: 'var(--mono, monospace)',
            border: '1px solid #cbd5e0',
            borderRadius: '3px',
            background: '#fff',
            color: '#2d3748',
            padding: '2px 6px',
            cursor: 'pointer',
            height: '22px',
            whiteSpace: 'nowrap',
            minWidth: '80px',
          }}
        >
          {declaration.soundfont ?? 'default sound'}
        </button>
        <SoundfontSearchModal
          open={searchOpen}
          onOpenChange={setSearchOpen}
          mode={declaration.mode === 'percussion' ? 'percussion' : 'instrument'}
          currentValue={declaration.soundfont ?? null}
          onSelect={(value) => {
            handleSoundfontChange(value ?? '')
            setSearchOpen(false)
          }}
          previewInstrument={previewInstrument}
          previewPercussion={previewPercussion}
          stopPreviewInstrument={stopPreviewInstrument}
          previewAudioPlaying={previewAudioPlaying}
        />
      </td>
      <td style={tdStyle}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <Slider.Root
            min={1}
            max={100}
            step={1}
            value={[declaration.volume ?? 100]}
            onValueChange={([v]) => handleVolumeChange(v)}
            data-testid={`volume-slider-${declaration.abbreviation}`}
            style={{
              position: 'relative',
              display: 'flex',
              alignItems: 'center',
              userSelect: 'none',
              touchAction: 'none',
              width: '80px',
              height: '20px',
            }}
          >
            <Slider.Track
              style={{
                background: '#e2e8f0',
                position: 'relative',
                flexGrow: 1,
                borderRadius: '9999px',
                height: '4px',
              }}
            >
              <Slider.Range
                style={{
                  position: 'absolute',
                  background: '#4a90d9',
                  borderRadius: '9999px',
                  height: '100%',
                }}
              />
            </Slider.Track>
            <Slider.Thumb
              style={{
                display: 'block',
                width: '14px',
                height: '14px',
                background: '#fff',
                border: '2px solid #4a90d9',
                borderRadius: '9999px',
                cursor: 'pointer',
                outline: 'none',
              }}
            />
          </Slider.Root>
          <span
            data-testid={`volume-value-${declaration.abbreviation}`}
            style={{
              fontSize: '11px',
              color: '#666',
              fontFamily: 'var(--mono, monospace)',
              minWidth: '28px',
            }}
          >
            {declaration.volume ?? 100}%
          </span>
        </div>
      </td>
      <td style={tdStyle}>
        <RadixSelect
          value={String(declaration.octaveOffset ?? 0)}
          onValueChange={handleOctaveChange}
          placeholder="octave"
          testId={`octave-select-${declaration.abbreviation}`}
        >
          {OCTAVE_OPTIONS.map((option) => (
            <RadixSelectItem key={option.value} value={option.value}>
              {option.label}
            </RadixSelectItem>
          ))}
        </RadixSelect>
      </td>
    </tr>
  )
}

const tdStyle: React.CSSProperties = {
  padding: '6px 10px',
  borderBottom: '1px solid #eee',
  verticalAlign: 'middle',
  fontSize: '13px',
}
