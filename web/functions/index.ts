// Cloudflare Pages Function for the root path. A shared-score link's `#synced=`
// hash never reaches the server (browsers don't send fragments in requests),
// so a link-preview crawler (WhatsApp, Slack, ...) -- which never executes JS
// -- would otherwise only ever see the static shell's generic "簡譜" title.
// `buildSyncedShareUrl` (`src/syncedShareUrl.ts`) mirrors the share id into a
// `?s=` query param for exactly this reason; this function reads it, fetches
// the share's current filename from `live-share-worker`, and rewrites the
// HTML before it leaves the edge. Ordinary browser navigation is unaffected:
// the client still drives everything from the hash, same as before.
//
// Only wired up for Cloudflare Pages -- GitHub Pages is plain static hosting
// with no way to run this, so share links opened there keep the generic
// preview.

const SHARE_QUERY_PARAM = 's'

// Must match `SHARE_ID_LENGTH` in `../src/syncedShareUrl.ts` and
// `crates/live-share-worker/src/share_id.rs`. Duplicated rather than
// imported so this edge function has no dependency on the SPA's bundle.
const SHARE_ID_LENGTH = 11
const SHARE_ID_PATTERN = new RegExp(`^[0-9A-Za-z_-]{${SHARE_ID_LENGTH}}$`)

// Matches `VITE_SYNCED_SHARE_HOST` in `.github/workflows/pages.yml`. Not
// read from an env binding: Pages Functions env vars would need their own
// dashboard/`wrangler.toml` setup that doesn't exist yet, and this worker
// host is already hardcoded at that same build-time location.
const SYNCED_SHARE_HOST = 'jianpu-live-share-worker-rs.hou32hou.workers.dev'

interface SyncedDocSummary {
  filename: string
  ended: boolean
}

export const onRequestGet: PagesFunction = async (context) => {
  const response = await context.next()

  const url = new URL(context.request.url)
  const payload = url.searchParams.get(SHARE_QUERY_PARAM)
  if (!payload) return response

  const shareId = payload.slice(0, SHARE_ID_LENGTH)
  if (!SHARE_ID_PATTERN.test(shareId)) return response

  let doc: SyncedDocSummary
  try {
    const docResponse = await fetch(
      `https://${SYNCED_SHARE_HOST}/shares/${shareId}`,
      { cache: 'no-store' },
    )
    if (!docResponse.ok) return response
    doc = await docResponse.json()
  } catch {
    return response
  }
  if (doc.ended || !doc.filename) return response

  const title = `${doc.filename.replace(/\.jianpu$/, '')} - 簡譜`

  // Rewrites the static shell's existing tags in place rather than
  // appending new ones: per the Open Graph spec, a parser takes the first
  // occurrence of a given property, so an appended `og:title` would be
  // shadowed by (not override) `index.html`'s default one.
  return new HTMLRewriter()
    .on('title', {
      element(element) {
        element.setInnerContent(title)
      },
    })
    .on('meta[property="og:title"]', {
      element(element) {
        element.setAttribute('content', title)
      },
    })
    .on('meta[property="og:description"]', {
      element(element) {
        element.setAttribute(
          'content',
          `${doc.filename} — a jianpu (numbered musical notation) score`,
        )
      },
    })
    .transform(response)
}
