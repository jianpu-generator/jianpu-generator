import { ChevronDownIcon } from '@radix-ui/react-icons'
import { useRef } from 'react'
import { DEMO_FILE_NAMES } from '../fileStore'
import { useDismissableOpen } from '../hooks/useDismissableOpen'

export interface DemoFileSwitcherProps {
  active: string
  onSelect: (name: string) => void
}

/** Dropdown listing the read-only demo files under `demo/` — kept separate
 * from `FileSwitcher`'s "My Files" dropdown so the reference material reads
 * as its own folder rather than being interleaved with the user's files. */
export function DemoFileSwitcher({ active, onSelect }: DemoFileSwitcherProps) {
  const isActive = DEMO_FILE_NAMES.includes(active)
  const containerRef = useRef<HTMLDivElement>(null)
  const [open, setOpen] = useDismissableOpen(containerRef)

  return (
    <div className="export-menu" ref={containerRef}>
      <button
        type="button"
        className="preview-export-btn"
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label="Demo files"
        onClick={() => setOpen((prev) => !prev)}
      >
        {isActive ? active : 'Demo'}
        <ChevronDownIcon className="export-menu-caret" aria-hidden="true" />
      </button>
      {open ? (
        <div className="export-menu-list file-tab-bar-files-list">
          <p className="file-tab-bar-hint">
            Demo is read-only — duplicate to edit.
          </p>
          <ul className="file-tabs" aria-label="Demo files">
            {DEMO_FILE_NAMES.map((name) => (
              <li
                key={name}
                className={`file-tab${name === active ? ' file-tab--active' : ''}`}
              >
                <button
                  type="button"
                  className="file-tab-name"
                  aria-current={name === active ? 'true' : undefined}
                  onClick={() => {
                    onSelect(name)
                    setOpen(false)
                  }}
                >
                  {name}
                </button>
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </div>
  )
}
