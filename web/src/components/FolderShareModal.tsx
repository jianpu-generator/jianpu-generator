import type { FileStoreState } from '../fileStore'
import { folderShareInfo } from '../fileStoreQueries'
import type { FolderShares } from '../hooks/useFolderShares'
import { useFolderStaticShareLink } from '../hooks/useStaticShareLink'
import type { SyncedShareGithubAuthResult } from '../storage/accountAuthPopup'
import { ShareModal, type Tab } from './ShareModal'

export interface FolderShareTarget {
  folder: string
  /** The tab shown first: "Share folder" opens the static one, "Sharing
   * settings" the synced one. */
  tab: Tab
}

interface FolderShareModalProps {
  store: FileStoreState
  /** The folder being shared; `null` while the dialog is closed. */
  target: FolderShareTarget | null
  onClose: () => void
  folderShares: FolderShares
  isGithubConnected: boolean
  githubLogin: string | null
  onSignInWithGithub: () => Promise<SyncedShareGithubAuthResult>
}

/** The Share dialog of one folder: wires `ShareModal` to the owner's share
 * overview and the folder's static snapshot link. What it says (counts,
 * warnings, whether the folder is already shared) all comes from Rust. */
export function FolderShareModal({
  store,
  target,
  onClose,
  folderShares,
  isGithubConnected,
  githubLogin,
  onSignInWithGithub,
}: FolderShareModalProps) {
  const folder = target?.folder ?? null
  const info =
    folder === null
      ? undefined
      : folderShareInfo(store, folderShares.overview, folder)
  const staticLink = useFolderStaticShareLink(target !== null, store, folder)
  return (
    <ShareModal
      open={target !== null}
      onOpenChange={(open) => {
        if (!open) onClose()
      }}
      kind="folder"
      name={info?.name ?? ''}
      staticLink={staticLink}
      initialTab={target?.tab ?? 'static'}
      startWarning={info?.ownLinkWarning ?? null}
      canSync={folderShares.canShare}
      isSynced={info?.shareId !== undefined}
      syncedShareLink={
        info?.shareId === undefined
          ? null
          : folderShares.linkFor(info.shareId, info.name)
      }
      isGithubConnected={isGithubConnected}
      githubLogin={githubLogin}
      onStartSync={() =>
        folder === null
          ? Promise.resolve(null)
          : folderShares.startSync(folder, info?.name ?? '')
      }
      onStopSync={() => {
        if (folder !== null) void folderShares.stopSync(folder)
      }}
      onSignInWithGithub={onSignInWithGithub}
    />
  )
}
