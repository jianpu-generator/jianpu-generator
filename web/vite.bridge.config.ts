import path from 'node:path'
import { defineConfig } from 'vite'
import { workerDeployConfigPlugin } from './vite.config'

// Builds the bridge page (`bridge/index.html`, `bridge/entry.ts` -> `src/bridge/main.ts`) the
// app's old origins serve after the move to `poentaro.pages.dev` -- see
// "Origin migration" in `ARCHITECTURE.md`. Deployed instead of the app to
// the old Cloudflare Pages project and GitHub Pages. Cloudflare Pages serves
// `index.html` for every path; GitHub Pages needs the `404.html` copy the
// deploy workflow makes.
export default defineConfig({
  root: path.resolve(__dirname, 'bridge'),
  base: process.env.VITE_BASE_PATH ?? '/',
  envDir: __dirname,
  plugins: [workerDeployConfigPlugin()],
  // `bridge/entry.ts` imports `src/bridge/`, outside the root.
  server: { fs: { allow: [__dirname] } },
  build: {
    outDir: path.resolve(__dirname, 'dist-bridge'),
    emptyOutDir: true,
  },
})
