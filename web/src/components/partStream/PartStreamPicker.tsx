import type { StreamablePart } from '../../jianpuWasm'

interface PartStreamPickerProps {
  /** `null` while the wasm is still loading. */
  parts: StreamablePart[] | null
  /** Why the last chosen part could not be extracted (Rust's message). */
  extractError: string | null
  onSelect: (part: StreamablePart) => void
}

/** Every declared part, in declaration order; parts Rust marks ineligible
 * are listed but disabled, with Rust's reason shown. */
export function PartStreamPicker({
  parts,
  extractError,
  onSelect,
}: PartStreamPickerProps) {
  if (parts === null) {
    return <p className="part-stream-picker__empty">Loading parts…</p>
  }
  if (parts.length === 0) {
    return <p className="part-stream-picker__empty">No parts declared.</p>
  }
  return (
    <div className="part-stream-picker">
      {extractError !== null && (
        <p className="part-stream-picker__error" role="alert">
          {extractError}
        </p>
      )}
      <ul className="part-stream-picker__list">
        {parts.map((part) => (
          <li key={part.abbreviation}>
            <button
              type="button"
              className="part-stream-picker__row"
              data-testid="part-stream-part-row"
              data-abbreviation={part.abbreviation}
              disabled={part.ineligibleReason !== undefined}
              onClick={() => onSelect(part)}
            >
              <span className="part-stream-picker__name">
                {part.displayName} [{part.abbreviation}]
              </span>
              <span className="part-stream-picker__kind">{part.kind}</span>
              {part.ineligibleReason !== undefined && (
                <span className="part-stream-picker__reason">
                  Not editable: {part.ineligibleReason}
                </span>
              )}
            </button>
          </li>
        ))}
      </ul>
    </div>
  )
}
