import path from 'node:path'
import react from '@vitejs/plugin-react'
import { defineConfig, type Plugin } from 'vite'

function wasmDevPlugin(): Plugin {
  return {
    name: 'jianpu-wasm-dev',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        if (req.url?.includes('.wasm')) {
          res.setHeader('Cache-Control', 'no-store')
        }
        next()
      })
    },
  }
}

export default defineConfig({
  plugins: [react(), wasmDevPlugin()],
  resolve: {
    alias: {
      'jianpu-wasm': path.resolve(
        __dirname,
        '../crates/jianpu-wasm/pkg/jianpu_wasm.js',
      ),
    },
  },
  worker: {
    format: 'es',
  },
  server: {
    fs: {
      allow: ['..'],
    },
  },
})
