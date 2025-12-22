/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}', './index.html'],
  darkMode: 'class', // Enable class-based dark mode
  theme: {
    extend: {
      fontFamily: {
        sans: ['Inter', 'system-ui', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
        mono: ['Hack', 'Menlo', 'Monaco', 'Courier New', 'monospace'],
      },
      colors: {
        bg: {
          primary: 'rgb(var(--color-bg-primary) / <alpha-value>)',
          secondary: 'rgb(var(--color-bg-secondary) / <alpha-value>)',
          tertiary: 'rgb(var(--color-bg-tertiary) / <alpha-value>)',
          card: 'rgb(var(--color-bg-card) / <alpha-value>)'
        },
        border: {
          subtle: 'rgb(var(--color-border-subtle) / <alpha-value>)'
        },
        accent: {
          primary: '#00d4aa',
          warning: '#f59e0b',
          error: '#ef4444',
		  success: '#22c55e',   
		},
        text: {
          primary: 'rgb(var(--color-text-primary) / <alpha-value>)',
          secondary: 'rgb(var(--color-text-secondary) / <alpha-value>)',
          muted: 'rgb(var(--color-text-muted) / <alpha-value>)'
        },
      },
    },
  },
  plugins: [],
};
