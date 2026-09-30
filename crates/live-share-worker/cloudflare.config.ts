// cf config for the Rust Synced Share worker (`.github/workflows/
// live-share-worker.yml` deploys it on every push to master touching
// `crates/live-share-worker/**`; `wrangler.config.ts` holds the build step).
//
// `name` deliberately does NOT reuse the old TypeScript worker's name
// (`jianpu-live`): the two stay distinct Workers so deploying this one never
// silently overwrites the still-running old one. `web/.env`,
// `web/.env.local` and `.github/workflows/pages.yml`'s `VITE_SYNCED_SHARE_HOST`
// point at this worker's `*.workers.dev` host, built from this name.
import { bindings, defineConfig } from 'cf/config'
import deploy from './deploy.json' with { type: 'json' }

// Local-run overrides, `{ "<var name>": "<value>" }` as JSON in
// `WORKER_VAR_OVERRIDES` (cf has no `--var` flag): e2e's
// `web/playwright.config.ts` points the worker's GitHub calls at a mock
// server with it, and `dekit.yaml` supplies the real client secret. Never set
// on deploy, so production only ever sees the values declared below.
const overrides: Record<string, string> = JSON.parse(
  process.env.WORKER_VAR_OVERRIDES ?? '{}',
)

const CLIENT_SECRET = 'SYNCED_SHARE_GITHUB_CLIENT_SECRET'

export default defineConfig({
  worker: {
    name: 'jianpu-live-share-worker-rs',
    compatibilityDate: '2024-09-23',
    // `worker-build`'s default output layout: a `build/` dir here containing
    // a JS shim entry point plus the compiled `.wasm` module.
    entrypoint: 'build/worker/shim.mjs',
    env: {
      // D1 binding. The name ("DB") must match `D1_BINDING` in
      // `src/handlers/mod.rs` -- do not rename one without the other.
      // Migrations live at `../../live-share-worker/migrations` (see
      // `build.rs` for why); pass `--dir` to `cf d1 migrations apply`.
      DB: bindings.d1({
        name: 'jianpu-live-share',
        id: deploy.d1DatabaseId,
      }),
      // Not secret: the browser already sends the same client id to build
      // the authorization URL (see `web/src/storage/syncedShareGithubAuth.ts`).
      // It is the dedicated Synced Share sign-in OAuth App, separate from
      // `githubAuth.ts`'s storage-backend app, so this identity-only sign-in
      // never inherits the storage app's `repo` grant.
      SYNCED_SHARE_GITHUB_CLIENT_ID: bindings.text(deploy.githubClientId),
      // The worker never reads it; it is declared so this crate is the
      // single source of deployment config -- `tests/export_openapi.rs`
      // publishes `deploy.json` to
      // `web/src/generated/live-share-worker/deployConfig.json`.
      SYNCED_SHARE_PUBLIC_HOST: bindings.text(deploy.workerHost),
      // NEVER committed. In production set it once via
      // `npx wrangler secret put SYNCED_SHARE_GITHUB_CLIENT_SECRET` (cf has
      // no `secret put` yet); locally `.dev.vars` or an override supplies it.
      // The name must match `CLIENT_SECRET_BINDING` in `src/oauth.rs`.
      [CLIENT_SECRET]: bindings.secret(),
      ...Object.fromEntries(
        Object.entries(overrides).map(([name, value]) => [
          name,
          bindings.text(value),
        ]),
      ),
    },
  },
})
