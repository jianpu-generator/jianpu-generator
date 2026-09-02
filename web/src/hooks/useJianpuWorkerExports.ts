import { useCallback } from 'react'
import type { WorkerRequest } from '../worker/jianpu.worker'
import { baseNameFromActiveFile, postAfterPaint } from './workerHelpers'

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
  mp3Exporting: boolean
  splitMp3Exporting: boolean
  setPdfExporting: (value: boolean) => void
  setSplitPdfExporting: (value: boolean) => void
  setMidiExporting: (value: boolean) => void
  setSplitMidiExporting: (value: boolean) => void
  setSplitWavExporting: (value: boolean) => void
  setMp3Exporting: (value: boolean) => void
  setSplitMp3Exporting: (value: boolean) => void
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
  mp3RequestIdRef: React.RefObject<number>
  latestMp3IdRef: React.RefObject<number>
  splitMp3RequestIdRef: React.RefObject<number>
  latestSplitMp3IdRef: React.RefObject<number>
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
  mp3Exporting,
  splitMp3Exporting,
  setPdfExporting,
  setSplitPdfExporting,
  setMidiExporting,
  setSplitMidiExporting,
  setSplitWavExporting,
  setMp3Exporting,
  setSplitMp3Exporting,
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
  mp3RequestIdRef,
  latestMp3IdRef,
  splitMp3RequestIdRef,
  latestSplitMp3IdRef,
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
    postAfterPaint(worker, payload)
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
    postAfterPaint(worker, payload)
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
    postAfterPaint(worker, payload)
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
    postAfterPaint(worker, payload)
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
    postAfterPaint(worker, payload)
  }, [
    splitWavExporting,
    workerRef,
    splitWavRequestIdRef,
    latestSplitWavIdRef,
    setSplitWavExporting,
    sourceRef,
    activeFileRef,
  ])

  const exportMp3 = useCallback(() => {
    const worker = workerRef.current
    if (!worker || mp3Exporting) return

    const id = ++mp3RequestIdRef.current
    latestMp3IdRef.current = id
    setMp3Exporting(true)

    const payload: WorkerRequest = {
      type: 'generateMp3',
      source: sourceRef.current,
      id,
      enabledTracks: enabledTracksRef.current,
    }
    postAfterPaint(worker, payload)
  }, [
    mp3Exporting,
    workerRef,
    mp3RequestIdRef,
    latestMp3IdRef,
    setMp3Exporting,
    sourceRef,
    enabledTracksRef,
  ])

  const exportSplitMp3 = useCallback(() => {
    const worker = workerRef.current
    if (!worker || splitMp3Exporting) return

    const id = ++splitMp3RequestIdRef.current
    latestSplitMp3IdRef.current = id
    setSplitMp3Exporting(true)

    const payload: WorkerRequest = {
      type: 'generateSplitMp3',
      source: sourceRef.current,
      id,
      baseName: baseNameFromActiveFile(activeFileRef.current),
    }
    postAfterPaint(worker, payload)
  }, [
    splitMp3Exporting,
    workerRef,
    splitMp3RequestIdRef,
    latestSplitMp3IdRef,
    setSplitMp3Exporting,
    sourceRef,
    activeFileRef,
  ])

  return {
    exportPdf,
    exportSplitPdf,
    exportMidi,
    exportSplitMidi,
    exportSplitWav,
    exportMp3,
    exportSplitMp3,
  }
}
