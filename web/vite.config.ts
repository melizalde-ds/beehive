import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

// Dev: the app calls /api/*, which the proxy forwards to the Rust service.
// Prod: set VITE_API_URL to the Rust service URL instead.
export default defineConfig({
  plugins: [react()],
  envDir: '..',
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
    },
  },
})
