import { useRef } from 'react'
import { useDismissableOpen } from '../hooks/useDismissableOpen'

/** Swaps a button's label for the shared spinner while `pending` is true. */
function SpinnerLabel({ pending, label }: { pending: boolean; label: string }) {
  return pending ? (
    <span className="file-tab-bar-spinner" aria-hidden="true" />
  ) : (
    label
  )
}

export interface BinMenuProps {
  binNames: string[]
  onRestore: (name: string) => void
  restoringName?: string | null
}

export function BinMenu({
  binNames,
  onRestore,
  restoringName = null,
}: BinMenuProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const [open, setOpen] = useDismissableOpen(containerRef)

  if (binNames.length === 0) return null

  return (
    <div className="export-menu file-tab-bar-bin" ref={containerRef}>
      <button
        type="button"
        className="preview-export-btn file-tab-bar-bin-trigger"
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={() => setOpen((prev) => !prev)}
      >
        Bin ({binNames.length})
        <span className="export-menu-caret" aria-hidden="true">
          ▾
        </span>
      </button>
      {open ? (
        <div className="export-menu-list file-tab-bar-bin-items" role="menu">
          {binNames.map((name) => (
            <div key={name} className="file-tab-bar-bin-item">
              <span className="file-tab-bar-bin-name">{name}</span>
              <button
                type="button"
                role="menuitem"
                className="file-tab-bar-restore"
                aria-label={`Restore ${name}`}
                onClick={() => onRestore(name)}
                disabled={restoringName === name}
              >
                <SpinnerLabel pending={restoringName === name} label="↩" />
              </button>
            </div>
          ))}
        </div>
      ) : null}
    </div>
  )
}
