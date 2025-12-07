import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { resolve } from 'path';

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        detail: resolve(__dirname, 'detail.html'),
      },
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
});
