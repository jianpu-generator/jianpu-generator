import { spawn } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import react from '@vitejs/plugin-react'
import type { Plugin, ViteDevServer } from 'vite'
import { defineConfig } from 'vitest/config'

const WASM_PACK_ARGS = [
  'build',
  '../crates/jianpu-wasm',
  '--target',
  'web',
  '--out-dir',
  'pkg',
  '--no-opt',
  '--',
  '--features',
  'wav,pdf',
] as const

const WASM_PKG_JS = path.resolve(
  __dirname,
  '../crates/jianpu-wasm/pkg/jianpu_wasm.js',
)

function isRustSource(file: string): boolean {
  return (
    file.endsWith('.rs') ||
    file.endsWith('Cargo.toml') ||
    file.endsWith('Cargo.lock')
  )
}

function wasmDevPlugin(): Plugin {
  let server: ViteDevServer | undefined
  let building = false
  let queued = false
  let debounceTimer: ReturnType<typeof setTimeout> | undefined

  const repoRoot = path.resolve(__dirname, '..')

  function runWasmPack(): Promise<void> {
    const wasmPackBin = path.join(
      __dirname,
      'node_modules',
      '.bin',
      process.platform === 'win32' ? 'wasm-pack.cmd' : 'wasm-pack',
    )

    return new Promise((resolve, reject) => {
      const child = spawn(wasmPackBin, [...WASM_PACK_ARGS], {
        cwd: __dirname,
        stdio: 'inherit',
      })

      child.on('exit', (code) => {
        if (code === 0) {
          resolve()
          return
        }
        reject(new Error(`wasm-pack exited with code ${code ?? 'unknown'}`))
      })
      child.on('error', reject)
    })
  }

  async function rebuild() {
    if (building) {
      queued = true
      return
    }

    building = true
    try {
      console.log('[jianpu-wasm] Rebuilding...')
      await runWasmPack()
      console.log('[jianpu-wasm] Rebuild complete')

      const wasmModule = server?.moduleGraph.getModuleById(WASM_PKG_JS)
      if (wasmModule) {
        server?.moduleGraph.invalidateModule(wasmModule)
      }
      server?.ws.send({ type: 'full-reload' })
    } catch (error) {
      console.error('[jianpu-wasm] Rebuild failed:', error)
    } finally {
      building = false
      if (queued) {
        queued = false
        void rebuild()
      }
    }
  }

  function scheduleRebuild(file: string) {
    if (!isRustSource(file)) {
      return
    }

    clearTimeout(debounceTimer)
    debounceTimer = setTimeout(() => {
      void rebuild()
    }, 300)
  }

  return {
    name: 'jianpu-wasm-dev',
    apply: 'serve',
    configureServer(devServer) {
      server = devServer

      devServer.middlewares.use((req, res, next) => {
        if (req.url?.includes('.wasm')) {
          res.setHeader('Cache-Control', 'no-store')
        }
        next()
      })

      const watchPaths = [
        path.join(repoRoot, 'crates/jianpu-wasm/src'),
        path.join(repoRoot, 'src'),
        path.join(repoRoot, 'Cargo.toml'),
        path.join(repoRoot, 'crates/jianpu-wasm/Cargo.toml'),
      ]

      for (const watchPath of watchPaths) {
        devServer.watcher.add(watchPath)
      }

      devServer.watcher.on('change', scheduleRebuild)
      devServer.watcher.on('add', scheduleRebuild)
      devServer.watcher.on('unlink', scheduleRebuild)
    },
  }
}

const DEPLOYED_ASSET_FILES = [
  'SourceHanSansSC-Regular.otf',
  'SourceHanSansTC-Regular.otf',
  'NotoSansMono-Regular.ttf',
  'GeneralUser_GS.sf2',
] as const

const fontsDir = path.resolve(__dirname, '..', 'fonts')

// Cloudflare Pages rejects any single deployed file over 25 MiB, so assets
// above that (currently just the soundfont) are split into parts plus a
// manifest that `useAssetLoader` reassembles client-side.
const MAX_DEPLOYED_FILE_BYTES = 20 * 1024 * 1024

function splitLargeFile(srcPath: string, outDir: string, name: string): void {
  const data = fs.readFileSync(srcPath)
  const chunkCount = Math.ceil(data.byteLength / MAX_DEPLOYED_FILE_BYTES)
  const chunkSize = Math.ceil(data.byteLength / chunkCount)
  const parts: string[] = []
  const partBytes: number[] = []
  for (let i = 0; i < chunkCount; i++) {
    const partName = `${name}.part${i}`
    const chunk = data.subarray(i * chunkSize, (i + 1) * chunkSize)
    fs.writeFileSync(path.join(outDir, partName), chunk)
    parts.push(partName)
    partBytes.push(chunk.byteLength)
  }
  fs.writeFileSync(
    path.join(outDir, `${name}.manifest.json`),
    JSON.stringify({ totalBytes: data.byteLength, parts, partBytes }),
  )
}

function copyFontsPlugin(): Plugin {
  return {
    name: 'copy-fonts',
    apply: 'build',
    closeBundle() {
      const outFonts = path.resolve(__dirname, 'dist', 'fonts')
      fs.mkdirSync(outFonts, { recursive: true })
      for (const name of DEPLOYED_ASSET_FILES) {
        const srcPath = path.join(fontsDir, name)
        if (fs.statSync(srcPath).size > MAX_DEPLOYED_FILE_BYTES) {
          splitLargeFile(srcPath, outFonts, name)
        } else {
          fs.copyFileSync(srcPath, path.join(outFonts, name))
        }
      }
    },
  }
}

function serveFontsPlugin(): Plugin {
  return {
    name: 'serve-fonts',
    apply: 'serve',
    configureServer(server) {
      server.middlewares.use('/fonts', (req, res, next) => {
        const filePath = path.join(fontsDir, req.url ?? '')
        if (!filePath.startsWith(fontsDir + path.sep)) {
          next()
          return
        }
        if (!fs.existsSync(filePath)) {
          next()
          return
        }
        res.setHeader('Content-Type', 'application/octet-stream')
        res.setHeader('Cache-Control', 'public, max-age=3600')
        fs.createReadStream(filePath).pipe(res)
      })
    },
  }
}

export default defineConfig({
  base: process.env.VITE_BASE_PATH ?? '/',
  plugins: [react(), wasmDevPlugin(), serveFontsPlugin(), copyFontsPlugin()],
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
  test: {
    // Vitest's default include glob also matches `*.spec.ts`, which is the
    // extension Playwright's e2e suite (web/e2e/**) uses — restrict to the
    // unit tests under src/ so `vitest run` doesn't try to execute those.
    include: ['src/**/*.test.{ts,tsx}'],
  },
})
