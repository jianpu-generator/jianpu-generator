import { useEffect, useRef, useState } from 'react'
import type { FileStoreState } from '../fileStore'
import { selectFile } from '../fileStore'
import { parseLiveShareFromHash } from '../liveShareUrl'
import { readFileNameFromUrl, writeFileNameToUrl } from '../urlFileParam'

/**
 * Keeps the active file and the `?file=` URL param in sync: on first load,
 * selects whichever file the URL names (if any), and thereafter keeps the
 * URL updated to whichever file is active, so reloading the page lands back
 * on the same file instead of the picker/default file.
 *
 * Initial selection is deferred until `isLoadingGithub` is false (relevant
 * for the GitHub backend, whose file list isn't known synchronously) so the
 * file is actually found rather than silently skipped — `selectFile`
 * no-ops for an unknown name. The "ready" flag is set in the same effect
 * invocation as the `setStore` call, so React batches them into a single
 * re-render: the URL-sync effect then only ever observes the *resolved*
 * active file, never a stale default it would otherwise briefly (or, if the
 * URL's file turns out not to exist, permanently) overwrite the URL with.
 *
 * Skipped entirely while a `#live=` link is open: a viewer's `store.active`
 * is just whatever local file happened to be selected before/independent of
 * the live session, and it must not leak into the address bar as `?file=`.
 */
export function useUrlFileSync(
  store: FileStoreState,
  setStore: (
    value: FileStoreState | ((prev: FileStoreState) => FileStoreState),
  ) => void,
  isLoadingGithub: boolean,
): void {
  const initialUrlFileAppliedRef = useRef(false)
  const [initialSelectionReady, setInitialSelectionReady] = useState(false)
  const isLiveViewer = parseLiveShareFromHash() !== null

  useEffect(() => {
    if (initialUrlFileAppliedRef.current || isLoadingGithub || isLiveViewer)
      return
    initialUrlFileAppliedRef.current = true
    const urlFile = readFileNameFromUrl()
    if (urlFile) setStore((prev) => selectFile(prev, urlFile))
    setInitialSelectionReady(true)
  }, [isLoadingGithub, isLiveViewer, setStore])

  useEffect(() => {
    if (!initialSelectionReady || isLiveViewer) return
    writeFileNameToUrl(store.active)
  }, [initialSelectionReady, isLiveViewer, store.active])
}
