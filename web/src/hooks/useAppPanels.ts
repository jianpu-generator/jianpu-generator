import { useCallback, useState } from 'react'
import { jianpuWasm, type MetadataEdit } from '../jianpuWasm'
import type { PartSettings } from '../types'
import { useMetadataFields } from './useMetadataFields'

/** Open/closed state for the edit-parts, edit-metadata, storage-settings,
 * and bin panels, plus the handlers that route their edits back to the
 * `.jianpu` source. */
export function useAppPanels(
  source: string,
  updatePartDeclaration: (
    abbreviation: string,
    settings: PartSettings,
  ) => Promise<string>,
  shiftPartOctave: (abbreviation: string, delta: number) => Promise<string>,
  handleSourceChange: (value: string) => void,
) {
  const [editPartsOpen, setEditPartsOpen] = useState(false)
  const [editMetadataOpen, setEditMetadataOpen] = useState(false)
  const [storageSettingsOpen, setStorageSettingsOpen] = useState(false)
  const [binOpen, setBinOpen] = useState(false)

  const handlePartDeclarationChange = useCallback(
    (abbreviation: string, settings: PartSettings) => {
      void updatePartDeclaration(abbreviation, settings).then(
        handleSourceChange,
      )
    },
    [updatePartDeclaration, handleSourceChange],
  )

  const handleShiftPartOctave = useCallback(
    (abbreviation: string, delta: number) => {
      void shiftPartOctave(abbreviation, delta).then(handleSourceChange)
    },
    [shiftPartOctave, handleSourceChange],
  )

  const parsedMetadata = useMetadataFields(source, editMetadataOpen)

  // Only reachable from the Edit Metadata modal, which renders its fields
  // once `parsedMetadata` is non-null, i.e. once the wasm is ready.
  const handleMetadataFieldChange = useCallback(
    (edit: MetadataEdit) => {
      handleSourceChange(jianpuWasm().updateMetadataField(source, edit))
    },
    [source, handleSourceChange],
  )

  return {
    editPartsOpen,
    setEditPartsOpen,
    editMetadataOpen,
    setEditMetadataOpen,
    storageSettingsOpen,
    setStorageSettingsOpen,
    binOpen,
    setBinOpen,
    handlePartDeclarationChange,
    handleShiftPartOctave,
    parsedMetadata,
    handleMetadataFieldChange,
  }
}
