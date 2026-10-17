import { type ChildProcess, spawn } from 'node:child_process'
import { connect } from 'node:net'
import { webServers } from '../e2e/webServers.ts'

function isUp(server: { url?: string; port?: number }): Promise<boolean> {
  if (server.url) {
    return fetch(server.url).then(
      () => true,
      () => false,
    )
  }
  return new Promise((done) => {
    const socket = connect(server.port ?? 0, 'localhost')
    socket.once('connect', () => {
      socket.destroy()
      done(true)
    })
    socket.once('error', () => done(false))
  })
}

// Playwright tears down the web servers it started when its process exits, so
// without this every pass would cold-start Vite, the mock OAuth server and
// `cf dev` (D1 migrations + worker build) all over again. Instead this script
// starts any that aren't already up once, and stops them when it is done;
// each pass's Playwright then just reuses them (`reuseExistingServer`).
export async function startWebServers(): Promise<ChildProcess[]> {
  const started: ChildProcess[] = []
  for (const server of webServers) {
    if (await isUp(server)) continue
    const child = spawn(server.command, {
      shell: true,
      cwd: server.cwd,
      env: { ...process.env, ...server.env },
      stdio: 'ignore',
      detached: true,
    })
    started.push(child)
    const deadline = Date.now() + (server.timeout ?? 60_000)
    while (!(await isUp(server))) {
      if (Date.now() > deadline || child.exitCode !== null) {
        stopWebServers(started)
        console.error(`e2e: web server did not come up: ${server.command}`)
        process.exit(1)
      }
      await new Promise((r) => setTimeout(r, 500))
    }
  }
  return started
}

export function stopWebServers(children: ChildProcess[]): void {
  for (const child of children) {
    try {
      if (child.pid) process.kill(-child.pid)
    } catch {
      // Already gone.
    }
  }
}
