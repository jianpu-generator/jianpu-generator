import type { SvgDocumentOut } from 'jianpu-wasm'
import type { RefObject } from 'react'
import type {
  Diagnostic,
  DiagnosticViewZone,
  MeasureSpan,
  PartDeclaration,
  PartInfo,
  SectionRange,
} from '../types'
import type { WorkerResponse } from '../worker/jianpu.worker'
import {
  downloadMidi,
  downloadPdf,
  downloadZip,
  midiFilenameFromActiveFile,
  pdfFilenameFromActiveFile,
  zipFilenameFromActiveFile,
} from './workerHelpers'

export interface WorkerMessageHandlerDeps {
  audioAvailableRef: RefObject<boolean>
  setAudioAvailable: (value: boolean) => void
  setPdfAvailable: (value: boolean) => void
  setMidiAvailable: (value: boolean) => void
  latestPartsIdRef: RefObject<number>
  setPartsLoading: (value: boolean) => void
  setParts: (value: PartInfo[]) => void
  setPartDeclarations: (value: PartDeclaration[]) => void
  latestUpdatePartDeclarationIdRef: RefObject<number>
  pendingPartDeclarationUpdatesRef: RefObject<
    Map<number, (source: string) => void>
  >
  latestPdfIdRef: RefObject<number>
  setPdfExporting: (value: boolean) => void
  activeFileRef: RefObject<string>
  enabledPartNamesRef: RefObject<string[] | undefined>
  setDiagnostics: (value: Diagnostic[]) => void
  latestSplitPdfIdRef: RefObject<number>
  setSplitPdfExporting: (value: boolean) => void
  latestMidiIdRef: RefObject<number>
  setMidiExporting: (value: boolean) => void
  latestSplitMidiIdRef: RefObject<number>
  setSplitMidiExporting: (value: boolean) => void
  latestSplitWavIdRef: RefObject<number>
  setSplitWavExporting: (value: boolean) => void
  latestRenderIdRef: RefObject<number>
  setRendering: (value: boolean) => void
  setDocuments: (value: SvgDocumentOut[]) => void
  setDiagnosticViewZones: (value: DiagnosticViewZone[]) => void
  latestAudioIdRef: RefObject<number>
  setAudioGenerating: (value: boolean) => void
  setNextWavUrl: (value: string | null) => void
  latestMeasureAudioIdRef: RefObject<number>
  setMeasureAudioGenerating: (value: boolean) => void
  setNextMeasureWavUrl: (value: string | null) => void
  latestHighlightRenderIdRef: RefObject<number>
  setHighlightedDocuments: (value: SvgDocumentOut[]) => void
  latestMeasureSpansIdRef: RefObject<number>
  setMeasureSpans: (value: MeasureSpan[]) => void
  setSectionRanges: (value: SectionRange[]) => void
  latestPreviewAudioIdRef: RefObject<number>
  currentPreviewAudioRef: RefObject<HTMLAudioElement | null>
  setPreviewAudioPlaying: (value: boolean) => void
}

export function createWorkerMessageHandler(deps: WorkerMessageHandlerDeps) {
  return (event: MessageEvent<WorkerResponse>) => {
    const msg = event.data
    if (msg.type === 'ready') {
      deps.audioAvailableRef.current = msg.audioAvailable
      deps.setAudioAvailable(msg.audioAvailable)
      deps.setPdfAvailable(msg.pdfAvailable)
      deps.setMidiAvailable(msg.midiAvailable)
      return
    }

    if (msg.type === 'parts') {
      if (msg.id !== deps.latestPartsIdRef.current) return
      deps.setPartsLoading(false)
      deps.setParts(msg.parts)
      deps.setPartDeclarations(msg.declarations)
      return
    }

    if (msg.type === 'partDeclarationUpdated') {
      if (msg.id !== deps.latestUpdatePartDeclarationIdRef.current) return
      deps.setPartDeclarations(msg.declarations)
      deps.pendingPartDeclarationUpdatesRef.current.get(msg.id)?.(msg.source)
      deps.pendingPartDeclarationUpdatesRef.current.delete(msg.id)
      return
    }

    if (msg.type === 'pdf') {
      if (msg.id !== deps.latestPdfIdRef.current) return
      deps.setPdfExporting(false)
      downloadPdf(
        msg.pdf,
        pdfFilenameFromActiveFile(
          deps.activeFileRef.current,
          deps.enabledPartNamesRef.current,
        ),
      )
      return
    }

    if (msg.type === 'pdfErr') {
      if (msg.id !== deps.latestPdfIdRef.current) return
      deps.setPdfExporting(false)
      deps.setDiagnostics(msg.diagnostics)
      return
    }

    if (msg.type === 'splitPdf') {
      if (msg.id !== deps.latestSplitPdfIdRef.current) return
      deps.setSplitPdfExporting(false)
      downloadZip(
        msg.zip,
        zipFilenameFromActiveFile(deps.activeFileRef.current),
      )
      return
    }

    if (msg.type === 'splitPdfErr') {
      if (msg.id !== deps.latestSplitPdfIdRef.current) return
      deps.setSplitPdfExporting(false)
      deps.setDiagnostics(msg.diagnostics)
      return
    }

    if (msg.type === 'midi') {
      if (msg.id !== deps.latestMidiIdRef.current) return
      deps.setMidiExporting(false)
      downloadMidi(
        msg.midi,
        midiFilenameFromActiveFile(
          deps.activeFileRef.current,
          deps.enabledPartNamesRef.current,
        ),
      )
      return
    }

    if (msg.type === 'midiErr') {
      if (msg.id !== deps.latestMidiIdRef.current) return
      deps.setMidiExporting(false)
      deps.setDiagnostics(msg.diagnostics)
      return
    }

    if (msg.type === 'splitMidi') {
      if (msg.id !== deps.latestSplitMidiIdRef.current) return
      deps.setSplitMidiExporting(false)
      downloadZip(
        msg.zip,
        zipFilenameFromActiveFile(deps.activeFileRef.current, 'MIDI parts'),
      )
      return
    }

    if (msg.type === 'splitMidiErr') {
      if (msg.id !== deps.latestSplitMidiIdRef.current) return
      deps.setSplitMidiExporting(false)
      deps.setDiagnostics(msg.diagnostics)
      return
    }

    if (msg.type === 'splitWav') {
      if (msg.id !== deps.latestSplitWavIdRef.current) return
      deps.setSplitWavExporting(false)
      downloadZip(
        msg.zip,
        zipFilenameFromActiveFile(deps.activeFileRef.current, 'WAV parts'),
      )
      return
    }

    if (msg.type === 'splitWavErr') {
      if (msg.id !== deps.latestSplitWavIdRef.current) return
      deps.setSplitWavExporting(false)
      deps.setDiagnostics(msg.diagnostics)
      return
    }

    if (msg.type === 'ok') {
      if (msg.id !== deps.latestRenderIdRef.current) return
      deps.setRendering(false)
      deps.setDocuments(msg.documents)
      deps.setDiagnostics(msg.diagnostics)
      deps.setDiagnosticViewZones(msg.diagnosticViewZones)
      return
    }

    if (msg.type === 'audio') {
      if (msg.id !== deps.latestAudioIdRef.current) return
      deps.setAudioGenerating(false)
      const url = URL.createObjectURL(
        new Blob([msg.wav], { type: 'audio/wav' }),
      )
      deps.setNextWavUrl(url)
      return
    }

    if (msg.type === 'audioErr') {
      if (msg.id !== deps.latestAudioIdRef.current) return
      deps.setAudioGenerating(false)
      return
    }

    if (msg.type === 'measureRangeAudio') {
      if (msg.id !== deps.latestMeasureAudioIdRef.current) return
      deps.setMeasureAudioGenerating(false)
      deps.setNextMeasureWavUrl(
        URL.createObjectURL(new Blob([msg.wav], { type: 'audio/wav' })),
      )
      return
    }

    if (msg.type === 'measureRangeAudioErr') {
      if (msg.id !== deps.latestMeasureAudioIdRef.current) return
      deps.setMeasureAudioGenerating(false)
      return
    }

    if (msg.type === 'highlightRangeOk') {
      if (msg.id !== deps.latestHighlightRenderIdRef.current) return
      deps.setHighlightedDocuments(msg.documents)
      return
    }

    if (msg.type === 'highlightRangeErr') {
      if (msg.id !== deps.latestHighlightRenderIdRef.current) return
      return
    }

    if (msg.type === 'measureSpans') {
      if (msg.id !== deps.latestMeasureSpansIdRef.current) return
      if (msg.status === 'ok') {
        deps.setMeasureSpans(msg.spans)
        deps.setSectionRanges(msg.sectionRanges)
      }
      return
    }

    if (msg.type === 'instrumentPreview') {
      if (msg.id !== deps.latestPreviewAudioIdRef.current) return
      const url = URL.createObjectURL(
        new Blob([msg.wav], { type: 'audio/wav' }),
      )
      if (deps.currentPreviewAudioRef.current) {
        deps.currentPreviewAudioRef.current.pause()
      }
      const audio = new Audio(url)
      deps.currentPreviewAudioRef.current = audio
      audio.addEventListener('play', () => deps.setPreviewAudioPlaying(true))
      audio.addEventListener('ended', () => {
        deps.setPreviewAudioPlaying(false)
        URL.revokeObjectURL(url)
      })
      audio.addEventListener('pause', () => deps.setPreviewAudioPlaying(false))
      audio.play().catch(() => {})
      return
    }

    if (msg.type === 'err') {
      if (msg.id !== deps.latestRenderIdRef.current) return
      deps.setRendering(false)
      deps.setDiagnostics(msg.diagnostics)
      deps.setDiagnosticViewZones(msg.diagnosticViewZones)
    }
  }
}
