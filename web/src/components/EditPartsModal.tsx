import * as Dialog from '@radix-ui/react-dialog'
import * as Select from '@radix-ui/react-select'
import type { PartInfo } from '../types'
import type {
  ParsedPartDeclaration,
  PartMode,
  SoundfontValue,
} from '../utils/partSource'

const GM_INSTRUMENTS: SoundfontValue[] = [
  '0: Acoustic Grand Piano',
  '1: Bright Acoustic Piano',
  '2: Electric Grand Piano',
  '3: Honky-tonk Piano',
  '4: Electric Piano 1',
  '5: Electric Piano 2',
  '6: Harpsichord',
  '7: Clavi',
  '8: Celesta',
  '9: Glockenspiel',
  '10: Music Box',
  '11: Vibraphone',
  '12: Marimba',
  '13: Xylophone',
  '14: Tubular Bells',
  '15: Dulcimer',
  '16: Drawbar Organ',
  '17: Percussive Organ',
  '18: Rock Organ',
  '19: Church Organ',
  '20: Reed Organ',
  '21: Accordion',
  '22: Harmonica',
  '23: Tango Accordion',
  '24: Acoustic Guitar (nylon)',
  '25: Acoustic Guitar (steel)',
  '26: Electric Guitar (jazz)',
  '27: Electric Guitar (clean)',
  '28: Electric Guitar (muted)',
  '29: Overdriven Guitar',
  '30: Distortion Guitar',
  '31: Guitar Harmonics',
  '32: Acoustic Bass',
  '33: Electric Bass (finger)',
  '34: Electric Bass (pick)',
  '35: Fretless Bass',
  '36: Slap Bass 1',
  '37: Slap Bass 2',
  '38: Synth Bass 1',
  '39: Synth Bass 2',
  '40: Violin',
  '41: Viola',
  '42: Cello',
  '43: Contrabass',
  '44: Tremolo Strings',
  '45: Pizzicato Strings',
  '46: Orchestral Harp',
  '47: Timpani',
  '48: String Ensemble 1',
  '49: String Ensemble 2',
  '50: Synth Strings 1',
  '51: Synth Strings 2',
  '52: Choir Aahs',
  '53: Voice Oohs',
  '54: Synth Voice',
  '55: Orchestra Hit',
  '56: Trumpet',
  '57: Trombone',
  '58: Tuba',
  '59: Muted Trumpet',
  '60: French Horn',
  '61: Brass Section',
  '62: Synth Brass 1',
  '63: Synth Brass 2',
  '64: Soprano Sax',
  '65: Alto Sax',
  '66: Tenor Sax',
  '67: Baritone Sax',
  '68: Oboe',
  '69: English Horn',
  '70: Bassoon',
  '71: Clarinet',
  '72: Piccolo',
  '73: Flute',
  '74: Recorder',
  '75: Pan Flute',
  '76: Blown Bottle',
  '77: Shakuhachi',
  '78: Whistle',
  '79: Ocarina',
  '80: Lead 1 (square)',
  '81: Lead 2 (sawtooth)',
  '82: Lead 3 (calliope)',
  '83: Lead 4 (chiff)',
  '84: Lead 5 (charang)',
  '85: Lead 6 (voice)',
  '86: Lead 7 (fifths)',
  '87: Lead 8 (bass + lead)',
  '88: Pad 1 (new age)',
  '89: Pad 2 (warm)',
  '90: Pad 3 (polysynth)',
  '91: Pad 4 (choir)',
  '92: Pad 5 (bowed)',
  '93: Pad 6 (metallic)',
  '94: Pad 7 (halo)',
  '95: Pad 8 (sweep)',
  '96: FX 1 (rain)',
  '97: FX 2 (soundtrack)',
  '98: FX 3 (crystal)',
  '99: FX 4 (atmosphere)',
  '100: FX 5 (brightness)',
  '101: FX 6 (goblins)',
  '102: FX 7 (echoes)',
  '103: FX 8 (sci-fi)',
  '104: Sitar',
  '105: Banjo',
  '106: Shamisen',
  '107: Koto',
  '108: Kalimba',
  '109: Bag Pipe',
  '110: Fiddle',
  '111: Shanai',
  '112: Tinkle Bell',
  '113: Agogo',
  '114: Steel Drums',
  '115: Woodblock',
  '116: Taiko Drum',
  '117: Melodic Tom',
  '118: Synth Drum',
  '119: Reverse Cymbal',
  '120: Guitar Fret Noise',
  '121: Breath Noise',
  '122: Seashore',
  '123: Bird Tweet',
  '124: Telephone Ring',
  '125: Helicopter',
  '126: Applause',
  '127: Gunshot',
]

export interface EditPartsModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  partDeclarations: ParsedPartDeclaration[]
  allParts: PartInfo[]
  onPartDeclarationChange: (
    abbreviation: string,
    mode: PartMode,
    followTarget: string | null,
    soundfont: SoundfontValue | null,
  ) => void
}

function PartRow({
  declaration,
  allParts,
  isFirstPart,
  onPartDeclarationChange,
  rowIndex,
}: {
  declaration: ParsedPartDeclaration
  allParts: PartInfo[]
  isFirstPart: boolean
  onPartDeclarationChange: EditPartsModalProps['onPartDeclarationChange']
  rowIndex: number
}) {
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
        declaration.soundfont,
      )
    } else {
      onPartDeclarationChange(
        declaration.abbreviation,
        mode as PartMode,
        null,
        declaration.soundfont,
      )
    }
  }

  function handleFollowTargetChange(target: string) {
    onPartDeclarationChange(
      declaration.abbreviation,
      'follow',
      target,
      declaration.soundfont,
    )
  }

  function handleSoundfontChange(value: string) {
    const newSoundfont = value === '' ? null : (value as SoundfontValue)
    onPartDeclarationChange(
      declaration.abbreviation,
      declaration.mode,
      declaration.followTarget,
      newSoundfont,
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
        <RadixSelect
          value={declaration.soundfont ?? ''}
          onValueChange={handleSoundfontChange}
          placeholder="default sound"
          testId={`soundfont-select-${declaration.abbreviation}`}
        >
          <RadixSelectItem value="">default sound</RadixSelectItem>
          {GM_INSTRUMENTS.map((instrument) => (
            <RadixSelectItem key={instrument} value={instrument}>
              {instrument}
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
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            background: '#fff',
            border: '1px solid #ddd',
            borderRadius: '6px',
            boxShadow: '0 8px 32px rgba(0,0,0,0.16)',
            zIndex: 1001,
            minWidth: '560px',
            maxWidth: '90vw',
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
                <col style={{ width: '22%' }} />
                <col style={{ width: '14%' }} />
                <col style={{ width: '34%' }} />
                <col style={{ width: '30%' }} />
              </colgroup>
              <thead>
                <tr>
                  <th style={thStyle}>Name</th>
                  <th style={thStyle}>Abbr</th>
                  <th style={thStyle}>Kind / Follow</th>
                  <th style={thStyle}>Soundfont</th>
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
