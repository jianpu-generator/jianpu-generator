import { useCallback } from 'react'
import type { WorkerRequest } from '../worker/jianpu.worker'
import { baseNameFromActiveFile } from './workerHelpers'

interface UseJianpuWorkerExportsParams {
  workerRef: React.RefObject<Worker | null>
  sourceRef: React.RefObject<string>
  activeFileRef: React.RefObject<string>
  enabledTracksRef: React.RefObject<string[] | undefined>
  disabledLyricsRef: React.RefObject<string[] | undefined>
  pdfExporting: boolean
  splitPdfExporting: boolean
  midiExporting: boolean
  splitMidiExporting: boolean
  splitWavExporting: boolean
  setPdfExporting: (value: boolean) => void
  setSplitPdfExporting: (value: boolean) => void
  setMidiExporting: (value: boolean) => void
  setSplitMidiExporting: (value: boolean) => void
  setSplitWavExporting: (value: boolean) => void
  pdfRequestIdRef: React.RefObject<number>
  latestPdfIdRef: React.RefObject<number>
  splitPdfRequestIdRef: React.RefObject<number>
  latestSplitPdfIdRef: React.RefObject<number>
  midiRequestIdRef: React.RefObject<number>
  latestMidiIdRef: React.RefObject<number>
  splitMidiRequestIdRef: React.RefObject<number>
  latestSplitMidiIdRef: React.RefObject<number>
  splitWavRequestIdRef: React.RefObject<number>
  latestSplitWavIdRef: React.RefObject<number>
}

export function useJianpuWorkerExports({
  workerRef,
  sourceRef,
  activeFileRef,
  enabledTracksRef,
  disabledLyricsRef,
  pdfExporting,
  splitPdfExporting,
  midiExporting,
  splitMidiExporting,
  splitWavExporting,
  setPdfExporting,
  setSplitPdfExporting,
  setMidiExporting,
  setSplitMidiExporting,
  setSplitWavExporting,
  pdfRequestIdRef,
  latestPdfIdRef,
  splitPdfRequestIdRef,
  latestSplitPdfIdRef,
  midiRequestIdRef,
  latestMidiIdRef,
  splitMidiRequestIdRef,
  latestSplitMidiIdRef,
  splitWavRequestIdRef,
  latestSplitWavIdRef,
}: UseJianpuWorkerExportsParams) {
  const exportPdf = useCallback(() => {
    const worker = workerRef.current
    if (!worker || pdfExporting || splitPdfExporting) return

    const id = ++pdfRequestIdRef.current
    latestPdfIdRef.current = id
    setPdfExporting(true)

    const payload: WorkerRequest = {
      type: 'generatePdf',
      source: sourceRef.current,
      id,
      enabledTracks: enabledTracksRef.current,
      disabledLyrics: disabledLyricsRef.current,
    }
    worker.postMessage(payload)
  }, [
    pdfExporting,
    splitPdfExporting,
    workerRef,
    pdfRequestIdRef,
    latestPdfIdRef,
    setPdfExporting,
    sourceRef,
    enabledTracksRef,
    disabledLyricsRef,
  ])

  const exportSplitPdf = useCallback(() => {
    const worker = workerRef.current
    if (!worker || pdfExporting || splitPdfExporting) return

    const id = ++splitPdfRequestIdRef.current
    latestSplitPdfIdRef.current = id
    setSplitPdfExporting(true)

    const payload: WorkerRequest = {
      type: 'generateSplitPdf',
      source: sourceRef.current,
      id,
      baseName: baseNameFromActiveFile(activeFileRef.current),
    }
    worker.postMessage(payload)
  }, [
    pdfExporting,
    splitPdfExporting,
    workerRef,
    splitPdfRequestIdRef,
    latestSplitPdfIdRef,
    setSplitPdfExporting,
    sourceRef,
    activeFileRef,
  ])

  const exportMidi = useCallback(() => {
    const worker = workerRef.current
    if (!worker || midiExporting) return

    const id = ++midiRequestIdRef.current
    latestMidiIdRef.current = id
    setMidiExporting(true)

    const payload: WorkerRequest = {
      type: 'generateMidi',
      source: sourceRef.current,
      id,
      enabledTracks: enabledTracksRef.current,
    }
    worker.postMessage(payload)
  }, [
    midiExporting,
    workerRef,
    midiRequestIdRef,
    latestMidiIdRef,
    setMidiExporting,
    sourceRef,
    enabledTracksRef,
  ])

  const exportSplitMidi = useCallback(() => {
    const worker = workerRef.current
    if (!worker || splitMidiExporting) return

    const id = ++splitMidiRequestIdRef.current
    latestSplitMidiIdRef.current = id
    setSplitMidiExporting(true)

    const payload: WorkerRequest = {
      type: 'generateSplitMidi',
      source: sourceRef.current,
      id,
      baseName: baseNameFromActiveFile(activeFileRef.current),
    }
    worker.postMessage(payload)
  }, [
    splitMidiExporting,
    workerRef,
    splitMidiRequestIdRef,
    latestSplitMidiIdRef,
    setSplitMidiExporting,
    sourceRef,
    activeFileRef,
  ])

  const exportSplitWav = useCallback(() => {
    const worker = workerRef.current
    if (!worker || splitWavExporting) return

    const id = ++splitWavRequestIdRef.current
    latestSplitWavIdRef.current = id
    setSplitWavExporting(true)

    const payload: WorkerRequest = {
      type: 'generateSplitWav',
      source: sourceRef.current,
      id,
      baseName: baseNameFromActiveFile(activeFileRef.current),
    }
    worker.postMessage(payload)
  }, [
    splitWavExporting,
    workerRef,
    splitWavRequestIdRef,
    latestSplitWavIdRef,
    setSplitWavExporting,
    sourceRef,
    activeFileRef,
  ])

  return {
    exportPdf,
    exportSplitPdf,
    exportMidi,
    exportSplitMidi,
    exportSplitWav,
  }
}
