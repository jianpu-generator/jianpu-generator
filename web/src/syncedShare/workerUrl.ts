// Builds the origin for the Synced Share worker from `VITE_SYNCED_SHARE_HOST`
// (a bare host, no scheme -- see that env var's doc comments in
// `useSyncedShareOwner.ts`/`syncedShareGithubAuth.ts`).
//
// Local dev points this at `localhost:8787` (`wrangler dev`, see
// `dekit.yaml`), which serves plain HTTP -- `wrangler dev` only serves HTTPS
// when started with `--local-protocol https`, and that requires trusting its
// self-signed cert in every browser profile/device before `fetch()` will
// succeed (a one-time click-through per browser that's easy to forget and
// surfaces as an opaque "Failed to fetch"). Nothing in the worker relies on
// HTTPS specifically (no `Secure` cookies, no secure-context check), so
// local dev stays plain HTTP and only the deployed `*.workers.dev` host
// (always HTTPS, terminated by Cloudflare) gets the `https://` scheme.
const LOCAL_HOST_PATTERN = /^(localhost|127\.0\.0\.1)(:\d+)?$/

export function syncedShareWorkerOrigin(host: string): string {
  const scheme = LOCAL_HOST_PATTERN.test(host) ? 'http' : 'https'
  return `${scheme}://${host}`
}
