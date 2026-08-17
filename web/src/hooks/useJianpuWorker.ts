import type { RefObject } from 'react'
import type { EditorHandle } from '../types'
import { useJianpuWorkerActions } from './useJianpuWorkerActions'
import { useJianpuWorkerState } from './useJianpuWorkerState'
import type { JianpuWorkerState } from './useJianpuWorkerTypes'
import { useSequenceNavigation } from './useSequenceNavigation'
import { useUnzippedTextFormat } from './useUnzippedTextFormat'
import { useUnzippedTextSnapshot } from './useUnzippedTextSnapshot'

export type { JianpuWorkerState } from './useJianpuWorkerTypes'

export function useJianpuWorker(
  source: string,
  disabledParts: ReadonlySet<string>,
  disabledLyrics: ReadonlySet<string>,
  soloedParts: ReadonlySet<string>,
  activeFile: string,
  soundfontBytes: Uint8Array | null,
  fontBytes: { sc: Uint8Array; tc: Uint8Array; mono: Uint8Array } | null,
  unzippedView: boolean,
  editorRef: RefObject<EditorHandle | null>,
  debounceMs = 300,
): JianpuWorkerState {
  const state = useJianpuWorkerState(
    source,
    activeFile,
    disabledParts,
    disabledLyrics,
    soloedParts,
  )
  const {
    parts,
    partDeclarations,
    partsLoading,
    documents,
    wavUrl,
    wavFilename,
    noteTimings,
    audioAvailable,
    pdfAvailable,
    pdfExporting,
    splitPdfExporting,
    midiAvailable,
    midiExporting,
    splitMidiExporting,
    splitWavExporting,
    diagnostics,
    diagnosticViewZones,
    rendering,
    audioGenerating,
    selectedMeasureRange,
    highlightedDocuments,
    measureSpans,
    noteSpans,
    unzippedText,
    setUnzippedText,
    partMeasureRanges,
    lyricsVerseRanges,
    sectionRanges,
    sequenceEntries,
    sourceRef,
  } = state

  const sequenceNav = useSequenceNavigation(sequenceEntries)
  useUnzippedTextSnapshot(unzippedView, sourceRef, setUnzippedText)

  const actions = useJianpuWorkerActions({
    state,
    sequenceNav,
    source,
    activeFile,
    soundfontBytes,
    fontBytes,
    unzippedView,
    debounceMs,
  })

  const formatUnzippedText = useUnzippedTextFormat({
    source,
    unzippedText,
    editorRef,
    setUnzippedText,
  })

  return {
    parts,
    partDeclarations,
    partsLoading,
    documents,
    wavUrl,
    wavFilename,
    noteTimings,
    audioAvailable,
    pdfAvailable,
    pdfExporting,
    splitPdfExporting,
    midiAvailable,
    midiExporting,
    splitMidiExporting,
    splitWavExporting,
    diagnostics,
    diagnosticViewZones,
    rendering,
    audioGenerating,
    exportPdf: actions.exportPdf,
    exportSplitPdf: actions.exportSplitPdf,
    exportMidi: actions.exportMidi,
    exportSplitMidi: actions.exportSplitMidi,
    exportSplitWav: actions.exportSplitWav,
    generateFullAudio: actions.generateFullAudio,
    selectedMeasureRange,
    measureAudioGenerating: actions.measureAudioGenerating,
    measureAudioPlaying: actions.measureAudioPlaying,
    measureAudioNoteTimings: actions.measureAudioNoteTimings,
    measureAudioElement: actions.measureAudioElement,
    notifySelection: actions.notifySelection,
    notifyUnzippedSelection: actions.notifyUnzippedSelection,
    unzippedText,
    partMeasureRanges,
    lyricsVerseRanges,
    playSelectedMeasures: actions.playSelectedMeasures,
    playFromCurrentMeasure: actions.playFromCurrentMeasure,
    playNoteSelection: actions.playNoteSelection,
    playAll: actions.playAll,
    stopMeasurePlayback: actions.stopMeasurePlayback,
    highlightedDocuments,
    measureSpans,
    noteSpans,
    sectionRanges,
    sequenceEntries,
    selectedSequenceRange: sequenceNav.selectedSequenceRange,
    sequenceJumpToolbarProps: sequenceNav.sequenceJumpToolbarProps,
    previewInstrument: actions.previewInstrument,
    previewPercussion: actions.previewPercussion,
    stopPreviewInstrument: actions.stopPreviewInstrument,
    previewAudioPlaying: actions.previewAudioPlaying,
    updatePartDeclaration: actions.updatePartDeclaration,
    formatScore: actions.formatScore,
    shiftPartOctave: actions.shiftPartOctave,
    formatUnzippedText,
    importFromFile: actions.importFromFile,
  }
}
