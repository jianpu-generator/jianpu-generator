import { routePartykitRequest } from 'partyserver'
import { LiveShareServer } from './server'

export { LiveShareServer }

interface Env {
  Main: DurableObjectNamespace<LiveShareServer>
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    return (
      (await routePartykitRequest(request, env)) ??
      new Response('Not Found', { status: 404 })
    )
  },
} satisfies ExportedHandler<Env>
