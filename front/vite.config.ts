import yaml from '@rollup/plugin-yaml'
import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react-swc'
import path from 'path'
import { defineConfig } from 'vite'
import { version } from './package.json'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss(), yaml({ include: '**/src/locales/**/*.yaml' })],
  define: {
    __APP_VERSION__: JSON.stringify(version),
  },
  server: {
    port: 5555
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
})
