import { defineWranglerConfig } from 'wrangler/experimental-config'

export default defineWranglerConfig({
  build: {
    // `worker-build` compiles this `workers-rs` crate to the wasm bundle in
    // `build/`. Pinned: an unpinned install upgrades to whatever crates.io has
    // newest, and 0.8.7 requires a newer rustc (1.91) than the local default
    // toolchain, which broke the dev server for e2e.
    command: 'cargo install worker-build --version 0.8.6 && worker-build --release',
  },
  types: { generate: false },
})
