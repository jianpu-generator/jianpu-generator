import { Server } from 'partyserver'
import type { Connection, ConnectionContext } from 'partyserver'
import type {
  LiveClientMessage,
  LiveEndedMessage,
  LivePresenceMessage,
  LiveRole,
  LiveSyncMessage,
} from './protocol'
import { resolveRole } from './resolveRole'

interface ConnectionState {
  role: LiveRole
}

interface RoomDoc {
  filename: string
  content: string
  revision: number
}

const EMPTY_DOC: RoomDoc = { filename: '', content: '', revision: 0 }

export class LiveShareServer extends Server {
  async onConnect(connection: Connection, ctx: ConnectionContext) {
    const incomingToken = new URL(ctx.request.url).searchParams.get(
      'ownerToken',
    )
    const storedOwnerToken =
      (await this.ctx.storage.get<string>('ownerToken')) ?? null
    const role = resolveRole(storedOwnerToken, incomingToken)

    if (role === 'owner' && storedOwnerToken === null && incomingToken) {
      await this.ctx.storage.put('ownerToken', incomingToken)
    }
    connection.setState({ role } satisfies ConnectionState)

    // Deliberately *not* cleared just because the owner (re)connected —
    // the owner's socket can auto-reconnect on its own (e.g. a Durable
    // Object cold-start blip) without the user ever pressing "Go Live"
    // again. Only a genuine `update` (see `onMessage`) proves the owner is
    // actually broadcasting again, so only that clears `ended`.
    const ended = (await this.ctx.storage.get<boolean>('ended')) ?? false
    const doc = (await this.ctx.storage.get<RoomDoc>('doc')) ?? EMPTY_DOC
    const sync: LiveSyncMessage = {
      type: 'sync',
      role,
      ended,
      filename: doc.filename,
      content: doc.content,
      revision: doc.revision,
    }
    connection.send(JSON.stringify(sync))
    this.broadcastPresence()
  }

  async onMessage(connection: Connection, message: string) {
    const state = connection.state as ConnectionState | null
    if (state?.role !== 'owner') return

    let parsed: LiveClientMessage
    try {
      parsed = JSON.parse(message)
    } catch {
      return
    }

    if (parsed.type === 'stop') {
      await this.ctx.storage.put('ended', true)
      const ended: LiveEndedMessage = { type: 'ended' }
      this.broadcast(JSON.stringify(ended), [connection.id])
      return
    }

    if (parsed.type !== 'update') return

    const doc: RoomDoc = {
      filename: parsed.filename,
      content: parsed.content,
      revision: parsed.revision,
    }
    await this.ctx.storage.put('doc', doc)
    await this.ctx.storage.put('ended', false)
    this.broadcast(JSON.stringify(parsed), [connection.id])
  }

  onClose() {
    this.broadcastPresence()
  }

  private broadcastPresence() {
    const presence: LivePresenceMessage = {
      type: 'presence',
      connectionCount: [...this.getConnections()].length,
    }
    this.broadcast(JSON.stringify(presence))
  }
}
