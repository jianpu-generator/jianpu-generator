import {
  DiscIcon,
  DownloadIcon,
  FileTextIcon,
  SpeakerLoudIcon,
} from '@radix-ui/react-icons'
import {
  ExportMenuButton,
  type ExportMenuItem,
  type ExportMenuSection,
} from './ExportMenuButton'

interface ExportControlsProps {
  hasDocuments: boolean
  rendering: boolean
  audioGenerating?: boolean
  wavUrl?: string | null
  soundfontReady?: boolean
  onGenerateAudio?: () => void
  pdfAvailable?: boolean
  pdfFontsReady?: boolean
  pdfExporting?: boolean
  onExportPdf?: () => void
  splitPdfExporting?: boolean
  onExportSplitPdf?: () => void
  midiAvailable?: boolean
  midiExporting?: boolean
  onExportMidi?: () => void
  splitMidiExporting?: boolean
  onExportSplitMidi?: () => void
  audioAvailable?: boolean
  splitWavExporting?: boolean
  onExportSplitWav?: () => void
  mp3Available?: boolean
  mp3Exporting?: boolean
  mp3Url?: string | null
  onExportMp3?: () => void
  splitMp3Exporting?: boolean
  onExportSplitMp3?: () => void
  partsCount?: number
  isLoadingGithub?: boolean
}

export function ExportControls({
  hasDocuments,
  rendering,
  audioGenerating = false,
  wavUrl = null,
  soundfontReady = false,
  onGenerateAudio,
  pdfAvailable = false,
  pdfFontsReady = false,
  pdfExporting = false,
  onExportPdf,
  splitPdfExporting = false,
  onExportSplitPdf,
  midiAvailable = false,
  midiExporting = false,
  onExportMidi,
  splitMidiExporting = false,
  onExportSplitMidi,
  audioAvailable = false,
  splitWavExporting = false,
  onExportSplitWav,
  mp3Available = false,
  mp3Exporting = false,
  mp3Url = null,
  onExportMp3,
  splitMp3Exporting = false,
  onExportSplitMp3,
  partsCount = 0,
  isLoadingGithub = false,
}: ExportControlsProps) {
  const exporting =
    pdfExporting || splitPdfExporting || midiExporting || splitMidiExporting
  const canExportPdf =
    pdfAvailable &&
    pdfFontsReady &&
    hasDocuments &&
    !rendering &&
    !exporting &&
    !isLoadingGithub
  const canExportSplitPdf =
    pdfAvailable &&
    pdfFontsReady &&
    partsCount > 0 &&
    !rendering &&
    !exporting &&
    !isLoadingGithub
  const canExportMidi =
    midiAvailable &&
    hasDocuments &&
    !rendering &&
    !exporting &&
    !isLoadingGithub
  const canExportSplitMidi =
    midiAvailable &&
    partsCount > 0 &&
    !rendering &&
    !exporting &&
    !isLoadingGithub
  const canExportWav =
    audioAvailable && soundfontReady && !audioGenerating && !isLoadingGithub
  const canExportSplitWav =
    audioAvailable &&
    soundfontReady &&
    partsCount > 0 &&
    !splitWavExporting &&
    !audioGenerating &&
    !isLoadingGithub
  const canExportMp3 =
    mp3Available && soundfontReady && !mp3Exporting && !isLoadingGithub
  const canExportSplitMp3 =
    mp3Available &&
    soundfontReady &&
    partsCount > 0 &&
    !splitMp3Exporting &&
    !isLoadingGithub

  const canExport =
    pdfAvailable || midiAvailable || audioAvailable || mp3Available
  const canExportParts = canExport && (partsCount > 1 || isLoadingGithub)

  const exportItems: ExportMenuItem[] = [
    ...(pdfAvailable
      ? [
          {
            key: 'pdf',
            label: 'PDF',
            busyLabel: 'Exporting PDF…',
            busy: pdfExporting,
            disabled: !canExportPdf,
            onSelect: () => onExportPdf?.(),
            icon: <FileTextIcon aria-hidden="true" />,
          },
        ]
      : []),
    ...(audioAvailable
      ? [
          {
            key: 'wav',
            label: wavUrl ? 'WAV (regenerate)' : 'WAV',
            busyLabel: 'Generating WAV…',
            busy: audioGenerating,
            disabled: !canExportWav,
            onSelect: () => onGenerateAudio?.(),
            icon: <SpeakerLoudIcon aria-hidden="true" />,
          },
        ]
      : []),
    ...(midiAvailable
      ? [
          {
            key: 'midi',
            label: 'MIDI',
            busyLabel: 'Exporting MIDI…',
            busy: midiExporting,
            disabled: !canExportMidi,
            onSelect: () => onExportMidi?.(),
            icon: <DiscIcon aria-hidden="true" />,
          },
        ]
      : []),
    ...(mp3Available
      ? [
          {
            key: 'mp3',
            label: mp3Url ? 'MP3 (regenerate)' : 'MP3',
            busyLabel: 'Generating MP3…',
            busy: mp3Exporting,
            disabled: !canExportMp3,
            onSelect: () => onExportMp3?.(),
            icon: <SpeakerLoudIcon aria-hidden="true" />,
          },
        ]
      : []),
  ]

  const exportPartsItems: ExportMenuItem[] = [
    ...(pdfAvailable
      ? [
          {
            key: 'pdf-parts',
            label: 'PDF (ZIP)',
            busyLabel: 'Exporting…',
            busy: splitPdfExporting,
            disabled: !canExportSplitPdf,
            onSelect: () => onExportSplitPdf?.(),
            icon: <FileTextIcon aria-hidden="true" />,
          },
        ]
      : []),
    ...(audioAvailable
      ? [
          {
            key: 'wav-parts',
            label: 'WAV (ZIP)',
            busyLabel: 'Exporting…',
            busy: splitWavExporting,
            disabled: !canExportSplitWav,
            onSelect: () => onExportSplitWav?.(),
            icon: <SpeakerLoudIcon aria-hidden="true" />,
          },
        ]
      : []),
    ...(midiAvailable
      ? [
          {
            key: 'midi-parts',
            label: 'MIDI (ZIP)',
            busyLabel: 'Exporting…',
            busy: splitMidiExporting,
            disabled: !canExportSplitMidi,
            onSelect: () => onExportSplitMidi?.(),
            icon: <DiscIcon aria-hidden="true" />,
          },
        ]
      : []),
    ...(mp3Available
      ? [
          {
            key: 'mp3-parts',
            label: 'MP3 (ZIP)',
            busyLabel: 'Exporting…',
            busy: splitMp3Exporting,
            disabled: !canExportSplitMp3,
            onSelect: () => onExportSplitMp3?.(),
            icon: <SpeakerLoudIcon aria-hidden="true" />,
          },
        ]
      : []),
  ]

  const sections: ExportMenuSection[] = [
    { title: canExportParts ? 'Visible Parts' : undefined, items: exportItems },
    ...(canExportParts
      ? [{ title: 'All Parts', items: exportPartsItems }]
      : []),
  ]
  const exportDisabled = sections.every((section) =>
    section.items.every((item) => item.disabled),
  )

  return (
    <div className="export-controls">
      {canExport ? (
        <ExportMenuButton
          label="Export"
          icon={<DownloadIcon aria-hidden="true" />}
          sections={sections}
          disabled={exportDisabled}
        />
      ) : null}
    </div>
  )
}
