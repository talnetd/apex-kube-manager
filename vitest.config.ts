import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Unit + component tests (jsdom). Kept separate from vite.config.ts so the
// app build doesn't pick up test settings.
export default defineConfig({
  plugins: [svelte()],
  resolve: {
    // Svelte's client runtime (mount, flushSync), not the SSR build.
    conditions: ['browser'],
  },
  test: {
    environment: 'jsdom',
    include: ['tests/**/*.test.ts'],
    setupFiles: ['tests/setup.ts'],
  },
});
