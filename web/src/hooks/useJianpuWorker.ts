import type { RefObject } from 'react'
import type { FontBytesByFamily } from '../jianpuWasm'
import type { PartToggleState } from '../types'
import { useJianpuWorkerActions } from './useJianpuWorkerActions'
import { useJianpuWorkerState } from './useJianpuWorkerState'
import type { JianpuWorkerState } from './useJianpuWorkerTypes'

export type { JianpuWorkerState } from './useJianpuWorkerTypes'

export function useJianpuWorker(
  source: string,
  partToggles: PartToggleState,
  wasmReady: boolean,
  activeFile: string,
  soundfontBytes: Uint8Array | null,
  fontBytes: FontBytesByFamily | null,
  /**
   * Owned by the caller (`useAppController`, which also feeds it to its own
   * `useSequenceNavigation` call) rather than this hook, since
   * `useMeasureAudioPlayback` below needs it before `useSequenceNavigation`
   * can run — that hook needs `notifySelection`, which this hook only
   * produces further down. See `useSequenceNavigation`'s matching parameter
   * doc comment.
   */
  selectedSequenceRangeRef: RefObject<{
    start: number
    end: number
    entryStartIndex: number
    entryEndIndex: number
  } | null>,
  /**
   * Owned by the caller (`useAppController`) for the same reason as
   * `selectedSequenceRangeRef` above — `useMeasureAudioPlayback` needs it
   * before `useAppSelectionAndNavigation`'s `useMeasureRangeSelection` (which
   * derives it) can run. See `useAppSelectionAndNavigation`'s matching
   * parameter doc comment.
   */
  measureRangeSelectedPartNamesRef: RefObject<string[] | undefined>,
  debounceMs = 300,
): JianpuWorkerState {
  const state = useJianpuWorkerState(source, activeFile, partToggles, wasmReady)
  const {
    parts,
    visibility,
    partDeclarations,
    partsLoading,
    documents,
    pendingDownload,
    requestDownload,
    confirmPendingDownload,
    cancelPendingDownload,
    wavUrl,
    wavFilename,
    mp3Url,
    mp3Filename,
    noteTimings,
    audioAvailable,
    pdfAvailable,
    pdfExporting,
    splitPdfExporting,
    midiAvailable,
    midiExporting,
    splitMidiExporting,
    splitWavExporting,
    mp3Available,
    mp3Exporting,
    splitMp3Exporting,
    diagnostics,
    diagnosticViewZones,
    rendering,
    audioGenerating,
    selectedMeasureRange,
    highlightedDocuments,
    measureSpans,
    noteSpans,
    lyricSpans,
    sectionRanges,
    sequenceEntries,
  } = state

  const actions = useJianpuWorkerActions({
    state,
    selectedSequenceRangeRef,
    measureRangeSelectedPartNamesRef,
    source,
    activeFile,
    soundfontBytes,
    fontBytes,
    debounceMs,
  })

  return {
    parts,
    visibility,
    partDeclarations,
    partsLoading,
    documents,
    pendingDownload,
    requestDownload,
    confirmPendingDownload,
    cancelPendingDownload,
    wavUrl,
    wavFilename,
    mp3Url,
    mp3Filename,
    noteTimings,
    audioAvailable,
    pdfAvailable,
    pdfExporting,
    splitPdfExporting,
    midiAvailable,
    midiExporting,
    splitMidiExporting,
    splitWavExporting,
    mp3Available,
    mp3Exporting,
    splitMp3Exporting,
    diagnostics,
    diagnosticViewZones,
    rendering,
    audioGenerating,
    exportPdf: actions.exportPdf,
    exportSplitPdf: actions.exportSplitPdf,
    exportMidi: actions.exportMidi,
    exportSplitMidi: actions.exportSplitMidi,
    exportSplitWav: actions.exportSplitWav,
    exportMp3: actions.exportMp3,
    exportSplitMp3: actions.exportSplitMp3,
    generateFullAudio: actions.generateFullAudio,
    selectedMeasureRange,
    measureAudioGenerating: actions.measureAudioGenerating,
    measureAudioPlaying: actions.measureAudioPlaying,
    measureAudioNoteTimings: actions.measureAudioNoteTimings,
    measureAudioElement: actions.measureAudioElement,
    notifySelection: actions.notifySelection,
    playSelectedMeasures: actions.playSelectedMeasures,
    playFromCurrentMeasure: actions.playFromCurrentMeasure,
    playNoteSelection: actions.playNoteSelection,
    playAll: actions.playAll,
    stopMeasurePlayback: actions.stopMeasurePlayback,
    highlightedDocuments,
    measureSpans,
    noteSpans,
    lyricSpans,
    sectionRanges,
    sequenceEntries,
    previewInstrument: actions.previewInstrument,
    previewPercussion: actions.previewPercussion,
    stopPreviewInstrument: actions.stopPreviewInstrument,
    previewAudioPlaying: actions.previewAudioPlaying,
    updatePartDeclaration: actions.updatePartDeclaration,
    formatScore: actions.formatScore,
    shiftPartOctave: actions.shiftPartOctave,
    editRange: actions.editRange,
    describeSelection: actions.describeSelection,
    importFromFile: actions.importFromFile,
  }
}
