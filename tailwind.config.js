/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}', './index.html'],
  theme: {
    extend: {
      colors: {
        bg: {
          primary: '#0a0a0a',
          secondary: '#111111',
          tertiary: '#1a1a1a',
          card: '#141414',
        },
        border: {
          subtle: '#222222',
        },
        accent: {
          primary: '#00d4aa',
          warning: '#f59e0b',
          error: '#ef4444',
          success: '#22c55e',
        },
        text: {
          primary: '#ffffff',
          secondary: '#888888',
          muted: '#555555',
        },
      },
    },
  },
  plugins: [],
};
