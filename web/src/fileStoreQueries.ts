// Read-only questions the file list and its dialogs ask Rust about a
// `FileStoreState`. Pure conversions around the wasm exports; all tree, path
// and naming logic lives in Rust. Needs the wasm component initialised.

import { DEMO_FILE_NAMES, DEMO_FILES } from './defaultSource'
import type { FileStoreState } from './fileStore'
import { operationContext, toWasmState } from './fileStoreWasm'
import type {
  FileActiveLabel,
  FileTreeRow,
  FileTreeViewRow,
  MoveTarget,
} from './jianpuWasm'
import { jianpuWasm } from './jianpuWasm'

/** The user's files and folders as display rows, depth-first, folders
 * before files, sorted (computed in Rust). Empty if the paths conflict. */
export function fileTreeRows(state: FileStoreState): FileTreeRow[] {
  const response = jianpuWasm().fileStoreTree(toWasmState(state))
  return response.tag === 'ok' ? response.val : []
}

/** The visible rows of the file list (collapsed folders hidden, the active
 * file's ancestors forced open, the read-only `Demo/` row last), computed in
 * Rust from the raw `expanded` set. Empty if the paths conflict. */
export function fileTreeView(
  state: FileStoreState,
  expanded: string[],
): FileTreeViewRow[] {
  const response = jianpuWasm().fileStoreTreeView(
    toWasmState(state),
    expanded,
    DEMO_FILE_NAMES,
  )
  return response.tag === 'ok' ? response.val : []
}

/** The folder choices of the "Move to" picker: the top level (empty `path`)
 * plus every folder. Pass the folder being moved as `excludedFolder` to leave
 * it and its descendants out. Computed in Rust. */
export function fileMoveTargets(
  state: FileStoreState,
  excludedFolder?: string,
): MoveTarget[] {
  return jianpuWasm().fileStoreMoveTargets(toWasmState(state), excludedFolder)
}

/** Where `source` lands when moved into `folder` (top level when omitted). */
export function moveDestination(
  source: string,
  folder?: string,
): string | undefined {
  return jianpuWasm().fileStoreMoveDestination(source, folder)
}

/** Why renaming/moving `from` to `to` would be rejected, as a message from
 * Rust; `undefined` when it would succeed. */
export function renameError(
  state: FileStoreState,
  from: string,
  to: string,
): string | undefined {
  return jianpuWasm().fileStoreRenameError(
    toWasmState(state),
    from,
    to,
    operationContext(DEMO_FILES, ''),
  )
}

/** The active file split into its folder names and own name (computed in Rust). */
export function activeFileLabel(active: string): FileActiveLabel {
  return jianpuWasm().fileStoreActiveLabel(active, DEMO_FILE_NAMES)
}
