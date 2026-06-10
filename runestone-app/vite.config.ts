import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import path from 'path'

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (!id.includes('node_modules')) return
          if (
            id.includes('@tiptap') ||
            id.includes('prosemirror') ||
            id.includes('lowlight') ||
            id.includes('highlight.js')
          ) {
            return 'editor'
          }
          if (id.includes('cytoscape')) return 'graph'
          if (id.includes('katex')) return 'math'
          if (id.includes('mermaid')) return 'mermaid'
        },
      },
    },
  },
  clearScreen: false,
  server: {
    host: '0.0.0.0',
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
})
