import * as Toast from '@radix-ui/react-toast'

/** Shows a toast with a spinner while WAV export is running in the background.
 *  Needed because the export dropdown closes as soon as an item is clicked,
 *  so its own "Generating WAV…" busy label isn't visible during export. */
export function ExportAudioToast({ open }: { open: boolean }) {
  return (
    <Toast.Provider swipeDirection="right" duration={Infinity}>
      <Toast.Root
        className="export-audio-toast"
        data-testid="wav-export-toast"
        open={open}
      >
        <span className="file-tab-bar-spinner" aria-hidden="true" />
        <Toast.Description>Generating WAV…</Toast.Description>
      </Toast.Root>
      <Toast.Viewport className="export-audio-toast-viewport" />
    </Toast.Provider>
  )
}
