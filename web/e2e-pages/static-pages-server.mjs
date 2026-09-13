// A small, dependency-free static file server that reproduces GitHub
// Pages' actual serving behavior for the built `web/dist` artifact --
// specifically its SPA-fallback `404.html` (see `.github/workflows/
// pages.yml`'s "Add SPA-fallback 404.html" step): any request path with no
// matching file on disk gets `DIST_DIR/404.html` served verbatim with a
// real HTTP **404** status, not a `200`. Neither Vite's dev server nor
// `vite preview` (neither used anywhere else in this repo's e2e) reproduces
// that "real 404 status, but still deliver the SPA shell" behavior, which
// is exactly the thing `github-callback-production-routing.feature` exists
// to catch a regression in.
//
// Modeled directly on `../e2e/mock-github-oauth-server.mjs`'s style: no
// framework, `PORT` from env, plain `if` route blocks.
import { createReadStream, existsSync, statSync } from 'node:fs'
import { createServer } from 'node:http'
import { extname, join, normalize } from 'node:path'

const PORT = Number(process.env.PORT ?? 4174)
const DIST_DIR = process.env.DIST_DIR ?? 'dist'
const BASE_PATH = process.env.BASE_PATH ?? '/jianpu-generator/'

const CONTENT_TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.json': 'application/json',
  '.wasm': 'application/wasm',
  '.woff2': 'font/woff2',
  '.png': 'image/png',
  '.ico': 'image/x-icon',
}

function contentTypeFor(filePath) {
  return CONTENT_TYPES[extname(filePath)] ?? 'application/octet-stream'
}

function serveFile(res, status, filePath) {
  res.writeHead(status, { 'Content-Type': contentTypeFor(filePath) })
  createReadStream(filePath).pipe(res)
}

const server = createServer((req, res) => {
  const url = new URL(req.url ?? '/', `http://localhost:${PORT}`)

  if (!url.pathname.startsWith(BASE_PATH)) {
    res.writeHead(404, { 'Content-Type': 'text/plain' })
    res.end('Not Found')
    return
  }

  const relativePath = url.pathname.slice(BASE_PATH.length) || 'index.html'
  // `normalize` collapses any `..` segments before joining, so a crafted
  // path can't escape `DIST_DIR` -- this only ever needs to serve this
  // repo's own build output, not act as a general-purpose file server.
  const candidate = join(DIST_DIR, normalize(relativePath))

  if (existsSync(candidate) && statSync(candidate).isFile()) {
    serveFile(res, 200, candidate)
    return
  }

  // The one behavior this server exists to reproduce: GitHub Pages serves
  // 404.html's *content* for any unmatched path, but with a real 404
  // *status* -- unlike Vite's dev server / `vite preview`, which have no
  // equivalent of this at all.
  const notFoundPage = join(DIST_DIR, '404.html')
  serveFile(res, 404, notFoundPage)
})

server.listen(PORT, () => {
  console.log(
    `Static GitHub-Pages-style server listening on http://localhost:${PORT}${BASE_PATH}`,
  )
})
