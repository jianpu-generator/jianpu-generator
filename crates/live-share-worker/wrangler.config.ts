import { defineWranglerConfig } from 'wrangler/experimental-config'

export default defineWranglerConfig({
  build: {
    // `worker-build` compiles this `workers-rs` crate to the wasm bundle in
    // `build/`. Pinned: an unpinned install upgrades to whatever crates.io has
    // newest, and 0.8.7 requires a newer rustc (1.91) than the local default
    // toolchain, which broke the dev server for e2e.
    //
    // CI's e2e shards download the `build/` that its `build-worker` job
    // already produced (`PREBUILT_WORKER=1`), so they skip recompiling it.
    command: process.env.PREBUILT_WORKER
      ? 'true'
      : 'cargo install worker-build --version 0.8.6 && worker-build --release',
  },
  types: { generate: false },
})
