import * as Dialog from '@radix-ui/react-dialog'
import * as Select from '@radix-ui/react-select'
import * as Slider from '@radix-ui/react-slider'
import { useState } from 'react'
import type {
  PartDeclaration,
  PartInfo,
  PartMode,
  SoundfontValue,
} from '../types'
import { SoundfontSearchModal } from './SoundfontSearchModal'

export interface EditPartsModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  partDeclarations: PartDeclaration[]
  allParts: PartInfo[]
  onPartDeclarationChange: (
    abbreviation: string,
    mode: PartMode,
    followTarget: string | null,
    soundfont: SoundfontValue | null,
    volume: number | null,
    octaveOffset: number | null,
  ) => void
  previewInstrument: (programNumber: number) => void
  stopPreviewInstrument: () => void
  previewAudioPlaying: boolean
}

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

function PartRow({
  declaration,
  allParts,
  isFirstPart,
  onPartDeclarationChange,
  rowIndex,
  previewInstrument,
  stopPreviewInstrument,
  previewAudioPlaying,
}: {
  declaration: PartDeclaration
  allParts: PartInfo[]
  isFirstPart: boolean
  onPartDeclarationChange: EditPartsModalProps['onPartDeclarationChange']
  rowIndex: number
  previewInstrument: (programNumber: number) => void
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
    const mode = newMode as PartMode
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
        mode as PartMode,
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
          currentValue={declaration.soundfont ?? null}
          onSelect={(value) => {
            handleSoundfontChange(value ?? '')
            setSearchOpen(false)
          }}
          previewInstrument={previewInstrument}
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

function RadixSelectItem({
  value,
  children,
}: {
  value: string
  children: React.ReactNode
}) {
  return (
    <Select.Item
      value={value}
      style={{
        padding: '4px 10px',
        cursor: 'pointer',
        outline: 'none',
        userSelect: 'none',
      }}
      onMouseEnter={(e) => {
        ;(e.currentTarget as HTMLElement).style.background = '#e8f0fe'
      }}
      onMouseLeave={(e) => {
        ;(e.currentTarget as HTMLElement).style.background = ''
      }}
    >
      <Select.ItemText>{children}</Select.ItemText>
    </Select.Item>
  )
}

function RadixSelect({
  value,
  onValueChange,
  placeholder,
  children,
  testId,
}: {
  value: string
  onValueChange: (value: string) => void
  placeholder: string
  children: React.ReactNode
  testId?: string
}) {
  return (
    <Select.Root value={value} onValueChange={onValueChange}>
      <Select.Trigger style={selectTriggerStyle} data-testid={testId}>
        <Select.Value placeholder={placeholder} />
        <Select.Icon style={{ marginLeft: '4px', color: '#666' }}>
          ▾
        </Select.Icon>
      </Select.Trigger>
      <Select.Portal>
        <Select.Content
          style={selectContentStyle}
          position="popper"
          sideOffset={4}
        >
          <Select.ScrollUpButton style={scrollButtonStyle}>
            ▲
          </Select.ScrollUpButton>
          <Select.Viewport>{children}</Select.Viewport>
          <Select.ScrollDownButton style={scrollButtonStyle}>
            ▼
          </Select.ScrollDownButton>
        </Select.Content>
      </Select.Portal>
    </Select.Root>
  )
}

const tdStyle: React.CSSProperties = {
  padding: '6px 10px',
  borderBottom: '1px solid #eee',
  verticalAlign: 'middle',
  fontSize: '13px',
}

const thStyle: React.CSSProperties = {
  padding: '6px 10px',
  textAlign: 'left',
  fontWeight: 600,
  fontSize: '12px',
  color: '#444',
  borderBottom: '2px solid #ddd',
  background: '#f5f5f5',
}

const selectTriggerStyle: React.CSSProperties = {
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
}

const selectContentStyle: React.CSSProperties = {
  background: '#fff',
  border: '1px solid #ccc',
  borderRadius: '4px',
  boxShadow: '0 4px 12px rgba(0,0,0,0.12)',
  fontFamily: 'var(--mono, monospace)',
  fontSize: '12px',
  zIndex: 9999,
  maxHeight: '260px',
  overflow: 'hidden',
}

const scrollButtonStyle: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  height: '20px',
  background: '#f5f5f5',
  cursor: 'default',
  fontSize: '10px',
  color: '#666',
}

export function EditPartsModal({
  open,
  onOpenChange,
  partDeclarations,
  allParts,
  onPartDeclarationChange,
  previewInstrument,
  stopPreviewInstrument,
  previewAudioPlaying,
}: EditPartsModalProps) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay
          style={{
            position: 'fixed',
            inset: 0,
            background: 'rgba(0,0,0,0.35)',
            zIndex: 1000,
          }}
        />
        <Dialog.Content
          data-testid="edit-parts-modal"
          style={{
            position: 'fixed',
            inset: 0,
            background: '#fff',
            zIndex: 1001,
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
              Edit Parts
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
          <div style={{ overflowY: 'auto', flex: 1 }}>
            <table
              style={{
                width: '100%',
                borderCollapse: 'collapse',
                tableLayout: 'fixed',
              }}
            >
              <colgroup>
                <col style={{ width: '16%' }} />
                <col style={{ width: '8%' }} />
                <col style={{ width: '24%' }} />
                <col style={{ width: '22%' }} />
                <col style={{ width: '18%' }} />
                <col style={{ width: '12%' }} />
              </colgroup>
              <thead>
                <tr>
                  <th style={thStyle}>Name</th>
                  <th style={thStyle}>Abbr</th>
                  <th style={thStyle}>Kind / Follow</th>
                  <th style={thStyle}>Soundfont</th>
                  <th style={thStyle}>Volume</th>
                  <th style={thStyle}>Octave</th>
                </tr>
              </thead>
              <tbody>
                {partDeclarations.map((declaration, index) => (
                  <PartRow
                    key={declaration.abbreviation}
                    declaration={declaration}
                    allParts={allParts}
                    isFirstPart={index === 0}
                    onPartDeclarationChange={onPartDeclarationChange}
                    rowIndex={index}
                    previewInstrument={previewInstrument}
                    stopPreviewInstrument={stopPreviewInstrument}
                    previewAudioPlaying={previewAudioPlaying}
                  />
                ))}
              </tbody>
            </table>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
