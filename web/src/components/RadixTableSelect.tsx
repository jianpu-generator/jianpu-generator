import * as Select from '@radix-ui/react-select'

export function RadixSelectItem({
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

export function RadixSelect({
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
