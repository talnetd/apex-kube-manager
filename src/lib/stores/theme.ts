// src/lib/stores/theme.ts
import { writable, derived, type Readable } from "svelte/store";

export type Theme = "light" | "dark" | "system";
export type ResolvedTheme = "light" | "dark";

const THEME_STORAGE_KEY = "apex-kube-manager-theme";

// Get initial theme from localStorage or default to 'system'
function getInitialTheme(): Theme {
  if (typeof globalThis.window === "undefined") return "system";

  const stored = localStorage.getItem(THEME_STORAGE_KEY);

  if (stored === "light" || stored === "dark" || stored === "system") {
    return stored;
  }

  return "system";
}

// Detect system theme preference
function getSystemTheme(): ResolvedTheme {
  if (typeof globalThis.window === "undefined") return "dark";

  const isDark = globalThis.window
    .matchMedia("(prefers-color-scheme: dark)")
    .matches;

  return isDark ? "dark" : "light";
}

// Create the theme store
export const theme = writable<Theme>(getInitialTheme());

// Derived store for the resolved theme (what's actually applied)
export const resolvedTheme: Readable<ResolvedTheme> = derived<
  typeof theme,
  ResolvedTheme
>(
  theme,
  ($theme: Theme, set: (value: ResolvedTheme) => void) => {
    if ($theme === "system") {
      set(getSystemTheme());

      // Listen for system theme changes
      const mediaQuery = globalThis.window.matchMedia(
        "(prefers-color-scheme: dark)"
      );
      const handler = (e: MediaQueryListEvent) => {
        set(e.matches ? "dark" : "light");
      };

      mediaQuery.addEventListener("change", handler);
      return () => mediaQuery.removeEventListener("change", handler);
    } else {
      set($theme);
    }
  },
  getSystemTheme()
);

// Subscribe to theme changes and persist to localStorage
theme.subscribe((value: Theme) => {
  if (typeof globalThis.window !== "undefined") {
    localStorage.setItem(THEME_STORAGE_KEY, value);
  }
});

// Subscribe to resolved theme changes and apply to document
resolvedTheme.subscribe((value: ResolvedTheme) => {
  if (
    typeof globalThis.window !== "undefined" &&
    typeof document !== "undefined"
  ) {
    if (value === "dark") {
      document.documentElement.classList.remove("light");
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
      document.documentElement.classList.add("light");
    }
  }
});

// Helper function to set theme
export function setTheme(newTheme: Theme) {
  theme.set(newTheme);
}

// Helper function to toggle between light and dark (skips system)
export function toggleTheme() {
  theme.update((current: Theme) => {
    if (current === "light") return "dark";
    if (current === "dark") return "light";

    // If current is system, toggle to opposite of current system theme
    const systemTheme = getSystemTheme();
    return systemTheme === "dark" ? "light" : "dark";
  });
}
