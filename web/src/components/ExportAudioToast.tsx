import * as Toast from '@radix-ui/react-toast'
import './ExportAudioToast.css'

/** Shows a toast with a spinner while a WAV/MP3 export is running in the
 *  background. Needed because the export dropdown closes as soon as an item
 *  is clicked, so its own busy label (e.g. "Generating MP3…") isn't visible
 *  during export. */
export function ExportAudioToast({
  open,
  label,
}: {
  open: boolean
  label: string
}) {
  return (
    <Toast.Provider swipeDirection="right" duration={Infinity}>
      <Toast.Root
        className="export-audio-toast"
        data-testid="export-audio-toast"
        open={open}
      >
        <span className="file-tab-bar-spinner" aria-hidden="true" />
        <Toast.Description>{label}</Toast.Description>
      </Toast.Root>
      <Toast.Viewport className="export-audio-toast-viewport" />
    </Toast.Provider>
  )
}
