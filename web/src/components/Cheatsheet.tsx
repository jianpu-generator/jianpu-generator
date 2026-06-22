import * as Dialog from '@radix-ui/react-dialog'
import { useEffect, useState } from 'react'
import { cheatsheetSections } from '../cheatsheetData'
import { getSnippetSvg, onSnippetUpdate } from '../cheatsheetSvgs'
import './Cheatsheet.css'

interface CheatsheetDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function CheatsheetDialog({
  open,
  onOpenChange,
}: CheatsheetDialogProps) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="cheatsheet-overlay" />
        <Dialog.Content className="cheatsheet-content">
          <Dialog.Title className="cheatsheet-title">
            Syntax Cheatsheet
          </Dialog.Title>
          <Dialog.Close className="cheatsheet-close" aria-label="Close">
            ✕
          </Dialog.Close>
          <CheatsheetBody />
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}

const sectionStartIndices: number[] = []
let runningIndex = 0
for (const section of cheatsheetSections) {
  sectionStartIndices.push(runningIndex)
  runningIndex += section.examples.length
}

function CheatsheetBody() {
  const [, forceUpdate] = useState(0)
  useEffect(() => {
    return onSnippetUpdate(() => forceUpdate((n) => n + 1))
  }, [])
  return (
    <>
      {cheatsheetSections.map((section, sectionIdx) => (
        <div key={section.title} className="cheatsheet-section">
          <h3 className="cheatsheet-section-title">{section.title}</h3>
          <table className="cheatsheet-table">
            <thead>
              <tr>
                <th>Description</th>
                <th>Syntax</th>
                <th>Preview</th>
              </tr>
            </thead>
            <tbody>
              {section.examples.map((example, exIdx) => {
                const flatIdx = sectionStartIndices[sectionIdx] + exIdx
                const svg = getSnippetSvg(flatIdx)
                return (
                  // biome-ignore lint/suspicious/noArrayIndexKey: cheatsheet rows are static and never reordered
                  <tr key={exIdx}>
                    <td>{example.description}</td>
                    <td className="cheatsheet-syntax">{example.syntax}</td>
                    <td className="cheatsheet-svg-cell">
                      {svg != null ? (
                        <span
                          // biome-ignore lint/security/noDangerouslySetInnerHtml: trusted SVG from local WASM renderer
                          dangerouslySetInnerHTML={{ __html: svg }}
                        />
                      ) : (
                        <div className="cheatsheet-svg-placeholder" />
                      )}
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        </div>
      ))}
    </>
  )
}
