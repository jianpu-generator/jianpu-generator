import { useLocalStorage } from 'usehooks-ts'
import {
  applyShareIfPresent,
  deserializeFileStore,
  FILE_STORE_KEY,
  type FileStoreState,
  readInitialFileStore,
} from '../fileStore'

function readInitialStore(): FileStoreState {
  return applyShareIfPresent(readInitialFileStore())
}

function deserializeStore(raw: string): FileStoreState {
  return applyShareIfPresent(deserializeFileStore(raw))
}

export function useLocalFileStore() {
  return useLocalStorage<FileStoreState>(FILE_STORE_KEY, readInitialStore, {
    deserializer: deserializeStore,
  })
}

/** @deprecated Use `useLocalFileStore` instead. */
export const useFileStore = useLocalFileStore
