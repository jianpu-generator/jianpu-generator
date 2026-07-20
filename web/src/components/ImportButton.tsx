import { UploadIcon } from '@radix-ui/react-icons'
import { useRef } from 'react'

interface ImportButtonProps {
  disabled?: boolean
  importing?: boolean
  onImportFile: (file: File) => void
}

/** Lets the user pick a previously exported `.svg`/`.pdf` file and recover
 * its embedded `.jianpu` source (see `extract_embedded_source`). Rendered as
 * a menu item inside the file-actions ("⋯") dropdown. The dropdown must stay
 * open (not close-on-click like the other menu items) until a file is
 * actually chosen — closing it eagerly unmounts the file `<input>` before
 * the OS picker has resolved, discarding the selection. */
export function ImportButton({
  disabled = false,
  importing = false,
  onImportFile,
}: ImportButtonProps) {
  const inputRef = useRef<HTMLInputElement>(null)

  return (
    <>
      <button
        type="button"
        role="menuitem"
        className="export-menu-item"
        disabled={disabled || importing}
        onClick={() => inputRef.current?.click()}
      >
        <UploadIcon aria-hidden="true" />
        {importing ? 'Importing…' : 'Import'}
      </button>
      <input
        ref={inputRef}
        type="file"
        accept=".svg,.pdf"
        style={{ display: 'none' }}
        onChange={(event) => {
          const file = event.target.files?.[0]
          event.target.value = ''
          if (file) onImportFile(file)
        }}
      />
    </>
  )
}
