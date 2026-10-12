import { renderToString } from 'react-dom/server'
import { describe, expect, it } from 'vitest'
import { SaveStatusBadge } from './SaveStatusBadge'

const render = (props: Parameters<typeof SaveStatusBadge>[0]) =>
  renderToString(<SaveStatusBadge {...props} />)

describe('SaveStatusBadge', () => {
  it('renders the outbox label under the unchanged test id', () => {
    const html = render({
      status: 'waiting',
      autosaveDeadline: null,
      label: '3 changes waiting',
    })
    expect(html).toContain('data-testid="save-status-badge"')
    expect(html).toContain('3 changes waiting')
  })

  it('falls back to the status text without a label', () => {
    expect(render({ status: 'saved', autosaveDeadline: null })).toContain(
      'Saved',
    )
  })

  it('renders nothing for idle', () => {
    expect(render({ status: 'idle', autosaveDeadline: null })).toBe('')
  })
})
