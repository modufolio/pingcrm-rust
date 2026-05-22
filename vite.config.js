import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'node:path'
import fs from 'node:fs'

// Writes `public/build/.vite-dev` containing the dev server URL while
// `vite` is running, and removes it on shutdown. The Rust server reads
// this file to decide whether to render dev or production asset tags.
const writeDevMarker = () => {
  const marker = path.resolve(__dirname, 'public/build/.vite-dev')
  const cleanup = () => {
    try { fs.unlinkSync(marker) } catch { /* ignore */ }
  }
  return {
    name: 'write-dev-marker',
    configureServer(server) {
      server.httpServer?.once('listening', () => {
        const addr = server.httpServer.address()
        const host = typeof addr === 'object' && addr ? addr.address : 'localhost'
        const port = typeof addr === 'object' && addr ? addr.port : 5173
        const url = `http://${host === '::' || host === '0.0.0.0' ? 'localhost' : host}:${port}`
        fs.mkdirSync(path.dirname(marker), { recursive: true })
        fs.writeFileSync(marker, url)
      })
      const onExit = () => { cleanup(); process.exit(0) }
      process.once('SIGINT', onExit)
      process.once('SIGTERM', onExit)
      process.once('exit', cleanup)
    },
    closeBundle() { cleanup() },
  }
}

export default defineConfig({
  plugins: [
    vue({
      template: {
        transformAssetUrls: {
          base: null,
          includeAbsolute: false,
        },
      },
    }),
    writeDevMarker(),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'resources/js'),
    },
  },
  publicDir: false,
  build: {
    outDir: 'public/build',
    emptyOutDir: true,
    manifest: true,
    rollupOptions: {
      input: path.resolve(__dirname, 'resources/js/app.js'),
    },
  },
  server: {
    port: 5173,
    strictPort: true,
    origin: 'http://localhost:5173',
  },
})
