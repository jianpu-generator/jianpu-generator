import {
  callWorker,
  type createWorkerClient,
} from '../syncedShare/workerClient'
import type { CloudBackend } from './cloudBackendTypes'

/** A cloud file's version history: the versions the worker keeps for it,
 * and one version rebuilt with its diff against the current text. */
export function createVersionCalls(
  client: ReturnType<typeof createWorkerClient>,
  identityToken: string,
): Pick<CloudBackend, 'listVersions' | 'fetchVersion'> {
  return {
    async listVersions(fileId) {
      const response = await callWorker(
        client.POST('/files/{id}/versions', {
          params: { path: { id: fileId } },
          body: { identityToken },
        }),
      )
      return response.versions
    },

    fetchVersion: (fileId, revision) =>
      callWorker(
        client.POST('/files/{id}/versions/get', {
          params: { path: { id: fileId } },
          body: { identityToken, revision },
        }),
      ),
  }
}
