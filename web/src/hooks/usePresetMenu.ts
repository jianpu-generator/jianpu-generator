import { useCallback, useMemo } from 'react'
import type { PresetMenuProps } from '../components/PresetMenu'
import { jianpuWasm, type Notation, type PresetSummary } from '../jianpuWasm'
import type { PartInfo, ResolvedPartVisibility } from '../types'
import type { StaffNotation } from './useStaffNotation'

interface PresetMenuInputs {
  source: string
  readOnly: boolean
  wasmReady: boolean
  parts: PartInfo[]
  visibility: ResolvedPartVisibility
  staffNotation: StaffNotation
  setChosenPreset: (presetName: string | null) => void
  handlePartToggle: (abbreviation: string, enabled: boolean) => void
  handleSourceChange: (value: string) => void
}

/** The preset menu's props, and the part pill handler: under a preset in a
 * file you can edit, a pill rewrites the preset's `visible_parts`; on
 * Default or in a read-only file it only changes this browser's toggles.
 * The notation menu works the same way on the preset's `notation` line. */
export function usePresetMenu({
  source,
  readOnly,
  wasmReady,
  parts,
  visibility,
  staffNotation,
  setChosenPreset,
  handlePartToggle,
  handleSourceChange,
}: PresetMenuInputs) {
  const presets = useMemo<PresetSummary[]>(
    () => (wasmReady ? jianpuWasm().listPresets(source) : []),
    [wasmReady, source],
  )
  const presetInForce = visibility.presetName ?? null

  const handlePartPillToggle = useCallback(
    (abbreviation: string, enabled: boolean) => {
      if (presetInForce == null || readOnly) {
        handlePartToggle(abbreviation, enabled)
        return
      }
      // A part hidden in this browser before the preset was chosen stays
      // off until shown again, so showing it also clears that.
      if (enabled) handlePartToggle(abbreviation, true)
      handleSourceChange(
        jianpuWasm().setPresetPartShown(
          source,
          presetInForce,
          parts.map((part) => part.abbreviation),
          { abbreviation, shown: enabled },
        ),
      )
    },
    [
      presetInForce,
      readOnly,
      source,
      parts,
      handlePartToggle,
      handleSourceChange,
    ],
  )

  const { setNotation } = staffNotation
  const handleNotationChange = useCallback(
    (notation: Notation) => {
      if (presetInForce == null || readOnly) {
        setNotation(notation)
        return
      }
      handleSourceChange(
        jianpuWasm().updatePresetField(source, presetInForce, {
          tag: 'notation',
          val: notation,
        }),
      )
    },
    [presetInForce, readOnly, source, setNotation, handleSourceChange],
  )

  const presetMenuProps: PresetMenuProps = {
    presets,
    presetInForce,
    onChoose: setChosenPreset,
    nameError: (name) => jianpuWasm().newPresetNameError(source, name),
    onCreatePreset: readOnly
      ? undefined
      : (name) => {
          const hidden = new Set(visibility.hiddenParts)
          handleSourceChange(
            jianpuWasm().addPresetSection(source, {
              name,
              visibleParts: parts
                .map((part) => part.abbreviation)
                .filter((abbreviation) => !hidden.has(abbreviation)),
              notation: staffNotation.notation,
            }),
          )
          setChosenPreset(name)
        },
  }

  return {
    handlePartPillToggle,
    presetMenuProps,
    staffNotation: { ...staffNotation, setNotation: handleNotationChange },
  }
}
