import { useLocalStorage } from 'usehooks-ts'
import {
  deserializeFileStore,
  FILE_STORE_KEY,
  type FileStoreState,
  readInitialFileStore,
} from '../fileStore'

export function useFileStore() {
  return useLocalStorage<FileStoreState>(FILE_STORE_KEY, readInitialFileStore, {
    deserializer: deserializeFileStore,
  })
}
