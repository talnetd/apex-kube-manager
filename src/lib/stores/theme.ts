// src/lib/stores/theme.ts
import { writable, derived, type Readable } from "svelte/store";

// Dark and Light only; System follows the OS between the two (spec 0, 10.2).
export type Theme = "light" | "dark" | "system";
export type ResolvedTheme = "light" | "dark";

export const THEME_STORAGE_KEY = "apex-kube-manager-theme";

/**
 * Parse a stored theme value. Anything unknown falls back to "system";
 * a value left over from a retired theme lands on "dark".
 */
export function parseStoredTheme(stored: string | null): Theme {
  if (stored === "light" || stored === "dark" || stored === "system") {
    return stored;
  }
  if (stored) return "dark";
  return "system";
}

function getInitialTheme(): Theme {
  if (typeof globalThis.window === "undefined") return "system";
  return parseStoredTheme(localStorage.getItem(THEME_STORAGE_KEY));
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

// Subscribe to theme changes and persist to localStorage. Skipping identical
// writes also rewrites a retired stored value exactly once.
theme.subscribe((value: Theme) => {
  if (typeof globalThis.window !== "undefined") {
    if (localStorage.getItem(THEME_STORAGE_KEY) !== value) {
      localStorage.setItem(THEME_STORAGE_KEY, value);
    }
  }
});

// Subscribe to resolved theme changes and apply to document
resolvedTheme.subscribe((value: ResolvedTheme) => {
  if (
    typeof globalThis.window !== "undefined" &&
    typeof document !== "undefined"
  ) {
    document.documentElement.classList.remove("light", "dark");
    document.documentElement.classList.add(value);
  }
});

// Detail and terminal windows are separate webviews sharing localStorage:
// follow theme changes made in the main window live.
if (typeof globalThis.window !== "undefined") {
  globalThis.window.addEventListener("storage", (e: StorageEvent) => {
    if (e.key === THEME_STORAGE_KEY) theme.set(parseStoredTheme(e.newValue));
  });
}

// Helper function to set theme
export function setTheme(newTheme: Theme) {
  theme.set(newTheme);
}

